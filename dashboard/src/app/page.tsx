import { getSession } from "@/auth";
import { redirect } from "next/navigation";

const Page = async () => {
  const session = await getSession();
  if (!session) {
    redirect("/login");
  }

  redirect("/event");
};

export default Page;
