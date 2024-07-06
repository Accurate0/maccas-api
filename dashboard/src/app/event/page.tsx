import { getSession } from "@/auth";
import { Time } from "@/components/time";
import { env } from "@/env";
import { GetEventsResponse } from "@/types/event";
import {
  Grid,
  Card,
  Badge,
  Flex,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeaderCell,
  TableRow,
  Title,
} from "@tremor/react";
import { redirect } from "next/navigation";

const Page = async () => {
  const session = await getSession();
  if (!session) {
    redirect("/login");
  }
  const response = await fetch(`${env.EVENT_API_BASE}/event`, {
    cache: "no-store",
    headers: {
      Authorization: `Bearer ${session?.accessToken}`,
    },
  }).then((r) => r.json() as Promise<GetEventsResponse>);

  return (
    <>
      <Grid className="gap-6 mt-6">
        <Card>
          <Flex justifyContent="start" className="space-x-2">
            <Title>Active</Title>
            <Badge color="gray">{response.active_events.length}</Badge>
          </Flex>
          <Table className="mt-6 table-fixed">
            <TableHead>
              <TableRow>
                <TableHeaderCell>Name</TableHeaderCell>
                <TableHeaderCell>Event ID</TableHeaderCell>
                <TableHeaderCell>Created</TableHeaderCell>
                <TableHeaderCell>Run At</TableHeaderCell>
                <TableHeaderCell>Data</TableHeaderCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {response.active_events.map((item) => {
                return (
                  <TableRow key={item.event_id}>
                    <TableCell>{item.name}</TableCell>
                    <TableCell>
                      <pre>{item.event_id}</pre>
                    </TableCell>
                    <TableCell>
                      <Time datetime={item.created_at} />
                    </TableCell>
                    <TableCell>
                      <Time datetime={item.should_be_completed_at} />
                    </TableCell>
                    <TableCell>
                      <pre className="whitespace-pre">
                        {JSON.stringify(item.data, null, 2)}
                      </pre>
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </Card>

        <Card>
          <Flex justifyContent="start" className="space-x-2">
            <Title>History</Title>
            <Badge color="gray">{response.historical_events.length}</Badge>
          </Flex>
          <Table className="mt-6 table-fixed">
            <TableHead>
              <TableRow>
                <TableHeaderCell>Name</TableHeaderCell>
                <TableHeaderCell>Event ID</TableHeaderCell>
                <TableHeaderCell>Created</TableHeaderCell>
                <TableHeaderCell>Completed</TableHeaderCell>
                <TableHeaderCell>Attempts</TableHeaderCell>
                <TableHeaderCell>Message</TableHeaderCell>
                <TableHeaderCell>Status</TableHeaderCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {response.historical_events.map((item) => {
                const colour = item.error
                  ? "rose"
                  : item.is_completed
                    ? "emerald"
                    : "gray";

                return (
                  <TableRow key={item.event_id}>
                    <TableCell>{item.name}</TableCell>
                    <TableCell>
                      <pre>{item.event_id}</pre>
                    </TableCell>
                    <TableCell>
                      <Time datetime={item.created_at} />
                    </TableCell>
                    <TableCell>
                      {item.completed_at ? (
                        <Time datetime={item.completed_at} />
                      ) : (
                        "Not finished"
                      )}
                    </TableCell>
                    <TableCell>{item.attempts}</TableCell>
                    <TableCell className="whitespace-pre-line">
                      {item.status === "Failed"
                        ? item.error_message
                        : item.status}
                    </TableCell>
                    <TableCell className="w-24">
                      <Badge className="w-24" color={colour} size="xl">
                        {item.status}
                      </Badge>
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </Card>
      </Grid>
    </>
  );
};

export default Page;
