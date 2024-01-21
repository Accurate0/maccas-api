export const dynamic = "force-dynamic";

import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { Title, TabGroup, TabList, Text, Tab } from "@tremor/react";
import Link from "next/link";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { TabsClient } from "@/components/tabs-client";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Maccas Dashboard",
};

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <main className="p-12">
          <Title>
            <Link href="/">Maccas Dashboard</Link>
          </Title>
          <Text>Lorem ipsum dolor sit amet, consetetur sadipscing elitr.</Text>
          <TabsClient />
          {children}
        </main>
      </body>
    </html>
  );
}
