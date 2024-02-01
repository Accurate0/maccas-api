"use client";

import { parseISO } from "date-fns";
import { useState, useEffect, CSSProperties, useMemo } from "react";

export const Time = ({ datetime }: { datetime: string }) => {
  // TODO: FIXME
  const date = parseISO(datetime + "Z");
  const adjustedDateTime = new Intl.DateTimeFormat("en-AU", {
    timeZone: "Australia/Perth",
    timeStyle: "medium",
    dateStyle: "medium",
  }).format(date);

  return (
    <time dateTime={adjustedDateTime} suppressHydrationWarning>
      {adjustedDateTime}
    </time>
  );
};
