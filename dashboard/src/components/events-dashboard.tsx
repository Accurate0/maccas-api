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
import { formatRelative } from "date-fns";

export const EventsDashboard = async () => {
  const response = await fetch(`${env.EVENT_API_BASE}/event`, {
    cache: "no-store",
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
                      {new Date(item.created_at).toLocaleString("en-AU")}
                    </TableCell>
                    <TableCell>
                      {new Date(item.should_be_completed_at).toLocaleString(
                        "en-AU"
                      )}
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

                const text = item.error
                  ? "failed"
                  : item.is_completed
                  ? "success"
                  : "not completed";

                return (
                  <TableRow key={item.event_id}>
                    <TableCell>{item.name}</TableCell>
                    <TableCell>
                      <pre>{item.event_id}</pre>
                    </TableCell>
                    <TableCell>
                      {new Date(item.created_at).toLocaleString("en-AU")}
                    </TableCell>
                    <TableCell>
                      {item.completed_at
                        ? new Date(item.completed_at).toLocaleString("en-AU")
                        : "Not finished"}
                    </TableCell>
                    <TableCell>{item.attempts}</TableCell>
                    <TableCell className="whitespace-pre-line">
                      {item.error_message ?? "Completed"}
                    </TableCell>
                    <TableCell className="w-24">
                      <Badge className="w-24" color={colour} size="xl">
                        {text}
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
