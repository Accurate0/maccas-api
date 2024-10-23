"use server";

import { env } from "@/env";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import jwt from "jsonwebtoken";
import { randomBytes } from "crypto";
import { db } from "@/db";
import bcrypt from "bcrypt";
import { SessionId, getSession } from "@/auth";

export const runJob = async ({ name }: { name: string }) => {
  const session = await getSession();

  await fetch(`${env.BATCH_API_BASE}/job/${name}`, {
    method: "POST",
    headers: { Authorization: `Bearer ${session?.accessToken}` },
  });
};

export const login = async (_: any, formData: FormData) => {
  const username = formData.get("username");
  const password = formData.get("password");
  // basic check
  if (
    typeof username !== "string" ||
    username.length < 1 ||
    username.length > 31
  ) {
    return {
      error: "Invalid username",
    };
  }
  if (
    typeof password !== "string" ||
    password.length < 1 ||
    password.length > 255
  ) {
    return {
      error: "Invalid password",
    };
  }

  try {
    const user = await db.user.findUnique({ where: { username } });
    if (!user) {
      return {
        error: "Incorrect username or password",
      };
    }

    const isPasswordCorrect = await bcrypt.compare(
      password,
      user.passwordHash.toString(),
    );

    if (!isPasswordCorrect) {
      return {
        error: "Incorrect username or password",
      };
    }

    const sessionId = randomBytes(64).toString("base64");
    const sevenDaysInMs = 604800000;

    const accessToken = jwt.sign(
      { userId: user.id, sessionId, role: user.role },
      env.AUTH_SECRET,
      {
        expiresIn: sevenDaysInMs / 1000,
        issuer: "Maccas Dashboard",
        audience: "Maccas API",
        subject: "Maccas API",
      },
    );

    const expires = new Date(Date.now() + sevenDaysInMs);
    await db.session.create({
      data: {
        userId: user.id,
        id: sessionId,
        expires,
        accessToken,
      },
    });

    (await cookies()).set(SessionId, sessionId, {
      path: "/",
      httpOnly: true,
      expires,
      sameSite: "strict",
    });
  } catch (e) {
    return {
      error: "An unknown error occurred",
    };
  }

  redirect("/");
};
