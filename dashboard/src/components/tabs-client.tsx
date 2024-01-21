"use client";

import Link from "next/link";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { usePathname } from "next/navigation";

export const TabsClient = () => {
  const pathname = usePathname();

  if (!["/event", "/batch"].includes(pathname)) {
    return null;
  }

  return (
    <Tabs defaultValue={pathname.replace("/", "")} className="mt-6">
      <TabsList>
        <TabsTrigger value="event">
          <Link href="/event">Event</Link>
        </TabsTrigger>
        <TabsTrigger value="batch">
          <Link href="/batch">Batch</Link>
        </TabsTrigger>
      </TabsList>
    </Tabs>
  );
};
