"use server";

import { env } from "@/env";

export const runJob = async ({ name }: { name: string }) => {
  await fetch(`${env.BATCH_API_BASE}/job/${name}`, { method: "POST" });
};
