import { getSession } from "@/auth";
import { BatchDashboard } from "@/components/batch-dashboard";
import { EventsDashboard } from "@/components/events-dashboard";
import {
  Tab,
  TabGroup,
  TabList,
  TabPanel,
  TabPanels,
  Text,
  Title,
} from "@tremor/react";
import { redirect } from "next/navigation";

export default async function DashboardExample() {
  const session = await getSession();
  if (!session) {
    redirect("/login");
  }

  return (
    <main className="p-12">
      <Title>Maccas Dashboard</Title>
      <Text>Lorem ipsum dolor sit amet, consetetur sadipscing elitr.</Text>

      <TabGroup className="mt-6">
        <TabList>
          <Tab>Events</Tab>
          <Tab>Batch</Tab>
        </TabList>
        <TabPanels>
          <TabPanel>
            <EventsDashboard />
          </TabPanel>
          <TabPanel>
            <BatchDashboard />
          </TabPanel>
        </TabPanels>
      </TabGroup>
    </main>
  );
}
