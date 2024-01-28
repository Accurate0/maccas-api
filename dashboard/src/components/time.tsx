"use client";

import { parseISO } from "date-fns";

export const Time = ({ datetime }: { datetime: string | number }) => {
  // TODO: FIXME
  const dateTime = parseISO(datetime + "Z");

  return (
    <time dateTime={dateTime.toISOString()}>
      {dateTime.toLocaleString("en-AU")}
    </time>
  );
};
