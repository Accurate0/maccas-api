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
import { StartJobButton } from "../../components/start-job-button";
import { getSession } from "@/auth";
import { Time } from "../../components/time";
import { TimeSecondsInFuture } from "../../components/time-in-seconds-future";
import { env } from "@/env";
import { GetJobsResponse } from "@/types/batch";

export const BatchDashboard = async () => {
  const session = await getSession();

  const response = await fetch(`${env.BATCH_API_BASE}/job`, {
    cache: "no-store",
    headers: {
      Authorization: `Bearer ${session?.accessToken}`,
    },
  }).then((r) => r.json() as Promise<GetJobsResponse>);

  return (
    <Grid className="gap-6 mt-6">
      <Card>
        <Flex justifyContent="start" className="space-x-2">
          <Title>Upcoming</Title>
          <Badge color="gray">{response.task_queue.length}</Badge>
        </Flex>
        <Table className="mt-6 table-fixed">
          <TableHead>
            <TableRow>
              <TableHeaderCell>Name</TableHeaderCell>
              <TableHeaderCell>Run At</TableHeaderCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {response.task_queue.map((item) => {
              return (
                <TableRow key={item.name}>
                  <TableCell>{item.name}</TableCell>
                  <TableCell>
                    <TimeSecondsInFuture
                      secondsInFuture={item.seconds_until_next}
                    />
                  </TableCell>
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
      </Card>

      <Card>
        <Flex justifyContent="start" className="space-x-2">
          <Title>Current</Title>
          <Badge color="gray">{response.current_jobs.length}</Badge>
        </Flex>
        <Table className="mt-6 table-fixed">
          <TableHead>
            <TableRow>
              <TableHeaderCell>Name</TableHeaderCell>
              <TableHeaderCell>State</TableHeaderCell>
              <TableHeaderCell className="text-right"></TableHeaderCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {response.current_jobs.map((item) => {
              const colour = item.state === "Stopped" ? "gray" : "emerald";
              const text = item.state;

              return (
                <TableRow key={item.name}>
                  <TableCell>{item.name}</TableCell>
                  <TableCell>
                    <Badge className="w-24" color={colour} size="xl">
                      {text}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-right">
                    <StartJobButton name={item.name} />
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
          <Badge color="gray">{response.history.length}</Badge>
        </Flex>
        <Table className="mt-6 table-fixed">
          <TableHead>
            <TableRow>
              <TableHeaderCell>Job ID</TableHeaderCell>
              <TableHeaderCell>Created</TableHeaderCell>
              <TableHeaderCell>Completed</TableHeaderCell>
              <TableHeaderCell>Message</TableHeaderCell>
              <TableHeaderCell>Status</TableHeaderCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {response.history.map((item) => {
              const colour = item.error
                ? "rose"
                : item.completed_at
                ? "emerald"
                : "blue";

              const text = item.error
                ? "failed"
                : item.completed_at
                ? "success"
                : "running";

              return (
                <TableRow key={item.completed_at}>
                  <TableCell>{item.job_name}</TableCell>
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
  );
};

export default BatchDashboard;
