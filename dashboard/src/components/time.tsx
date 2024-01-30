"use client";

import { parseISO } from "date-fns";
import { useHydration } from "./use-hydration";
import { Suspense } from "react";

export const Time = ({ datetime }: { datetime: string }) => {
  // TODO: FIXME
  const dateTime = parseISO(datetime + "Z");
  const hydrated = useHydration();

  return (
    <Suspense key={hydrated ? "local" : "utc"}>
      <time dateTime={dateTime.toISOString()}>
        {dateTime.toLocaleString("en-AU")}
      </time>
    </Suspense>
  );
};
