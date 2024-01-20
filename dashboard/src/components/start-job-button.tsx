"use client";

import { runJob } from "@/app/action";
import { Button } from "@tremor/react";

export const StartJobButton = ({ name }: { name: string }) => (
  <Button onClick={(e) => runJob({ name: name })}>Start new</Button>
);
