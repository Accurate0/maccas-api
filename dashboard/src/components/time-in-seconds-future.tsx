"use client";

import { Suspense } from "react";
import { useHydration } from "./use-hydration";

export const TimeSecondsInFuture = ({
  secondsInFuture,
}: {
  secondsInFuture: number;
}) => {
  const dateTime = new Date(Date.now() + secondsInFuture * 1000);
  const hydrated = useHydration();

  return (
    <Suspense key={hydrated ? "local" : "utc"}>
      <time dateTime={dateTime.toISOString()}>
        {dateTime.toLocaleString("en-AU")}
      </time>
    </Suspense>
  );
};
