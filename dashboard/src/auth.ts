import { cookies } from "next/headers";
import { db } from "./db";

export const SessionId = "dashboard-session-id";

export const getSession = async () => {
  const cookie = (await cookies()).get(SessionId);
  if (cookie) {
    const session = await db.session.findUnique({
      where: {
        id: cookie.value,
      },
    });

    if (!session) {
      return null;
    }

    if (session?.expires > new Date()) {
      return session;
    }

    return null;
  }

  return null;
};
