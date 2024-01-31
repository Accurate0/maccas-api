"use client";

import { parseISO } from "date-fns";
import { useState, useEffect, CSSProperties, useMemo } from "react";

export const Time = ({ datetime }: { datetime: string }) => {
  // TODO: FIXME
  const [currentTime, setCurrentTime] = useState(
    parseISO(datetime + "Z").toLocaleString("en-AU")
  );

  useEffect(() => {
    setCurrentTime(parseISO(datetime + "Z").toLocaleString("en-AU"));
  }, [datetime]);

  const styleMemo = useMemo(
    () =>
      ({
        visibility: typeof window === "undefined" ? "hidden" : "visible",
      } as CSSProperties),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [currentTime]
  );

  return (
    <time
      style={{ ...styleMemo }}
      dateTime={currentTime}
      suppressHydrationWarning
    >
      {currentTime}
    </time>
  );
};
