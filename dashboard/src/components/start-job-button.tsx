"use client";

import { runJob } from "@/app/action";
import { Button } from "@tremor/react";

export const StartJobButton = ({
  name,
  disabled,
}: {
  name: string;
  disabled: boolean;
}) => (
  <Button
    disabled={disabled}
    aria-disabled={disabled}
    onClick={() => runJob({ name: name })}
  >
    Start new
  </Button>
);
