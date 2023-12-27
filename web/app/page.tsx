import { userSession } from "@/lib/session";
import { redirect } from "next/navigation";

export default async function Home() {
  const session = await userSession();
  if (session) {
    redirect("/offers");
  }

  return null;
}
