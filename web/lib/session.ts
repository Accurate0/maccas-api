"use server";

import { cookies } from "next/headers";
import { SessionId } from "./constants";
import { redirect } from "next/navigation";
import { prisma } from "./prisma";

export const userSession = async () => {
  const cookieJar = cookies();
  const sessionId = cookieJar.get(SessionId);
  if (!sessionId) {
    return redirect("/auth/login");
  }

  const existingSession = await prisma.session.findUnique({
    where: { id: sessionId.value },
  });

  if (!existingSession) {
    return redirect("/auth/login");
  }

  return existingSession;
};
