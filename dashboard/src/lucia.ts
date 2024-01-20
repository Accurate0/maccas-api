import { lucia } from "lucia";
import { prisma } from "@lucia-auth/adapter-prisma";
import { PrismaClient } from "@prisma/client";
import { nextjs_future } from "lucia/middleware";
import { env } from "./env";

const client = new PrismaClient();

export const auth = lucia({
  adapter: prisma(client),
  env: env.NODE_ENV === "development" ? "DEV" : "PROD",
  middleware: nextjs_future(),
  sessionCookie: {
    expires: false,
  },
});
