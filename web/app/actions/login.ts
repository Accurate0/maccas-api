"use server";

import { redirect } from "next/navigation";
import { z } from "zod";
import { randomBytes } from "crypto";
import bcrypt from "bcrypt";
import { Role } from "@prisma/client";
import { cookies } from "next/headers";
import { SessionId } from "@/lib/constants";
import { prisma } from "@/lib/prisma";

const schema = z.object({
  username: z.string().min(1),
  password: z.string().min(1),
});

export type LoginState = {
  error: string | null;
};

const legacyLoginResponseSchema = z.object({
  token: z.string().min(1),
  refreshToken: z.string().min(1),
  role: z.union([
    z.literal("admin"),
    z.literal("privileged"),
    z.literal("none"),
  ]),
});

const createSession = async (userId: string) => {
  const sessionId = randomBytes(30).toString("base64");
  const sevenDaysInMs = 604800000;
  const expires = new Date(Date.now() + sevenDaysInMs);

  await prisma.session.create({
    data: {
      userId,
      id: sessionId,
      expires,
    },
  });

  cookies().set(SessionId, sessionId, { path: "/", httpOnly: true, expires });
};

export async function login(_currentState: LoginState, formData: FormData) {
  const validatedFields = schema.safeParse({
    username: formData.get("username"),
    password: formData.get("password"),
  });

  // Return early if the form data is invalid
  if (!validatedFields.success) {
    return {
      error: "Invalid details",
    };
  }

  const { username, password } = validatedFields.data;
  const existingUser = await prisma.user.findUnique({ where: { username } });
  if (existingUser) {
    const isPasswordCorrect = await bcrypt.compare(
      password,
      existingUser.passwordHash.toString()
    );

    if (!isPasswordCorrect) {
      return {
        error: "Invalid details",
      };
    }

    await createSession(existingUser.id);
  } else {
    const response = await fetch("https://api.maccas.one/v1/auth/login", {
      method: "POST",
      body: formData,
    });

    if (!response.ok) {
      return {
        error: "Invalid details",
      };
    }

    const result = await legacyLoginResponseSchema.safeParseAsync(
      await response.json()
    );

    if (!result.success) {
      return {
        error: "Invalid details",
      };
    }

    const { role, token } = result.data;

    const existingUserId = JSON.parse(atob(token.split(".")[1] ?? ""))[
      "oid"
    ] as string;

    const passwordHash = await bcrypt.hash(password, 10);
    const newUser = await prisma.user.create({
      data: {
        id: existingUserId,
        username: username,
        passwordHash: Buffer.from(passwordHash),
        // the prisma one is just uppercase, this should be fine
        role: role.toUpperCase() as Role,
      },
    });

    await createSession(existingUserId);

    console.log(
      `new user created for ${newUser.username} with user id: ${existingUserId}`
    );
  }

  redirect("/offers");
}
