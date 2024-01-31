"use client";

import { CSSProperties, useEffect, useState } from "react";

export const TimeSecondsInFuture = ({
  secondsInFuture,
}: {
  secondsInFuture: number;
}) => {
  const [currentTime, setCurrentTime] = useState(
    new Date(Date.now() + secondsInFuture * 1000).toLocaleString()
  );

  useEffect(() => {
    setCurrentTime(
      new Date(Date.now() + secondsInFuture * 1000).toLocaleString()
    );
  }, [secondsInFuture]);

  const style: CSSProperties = {
    visibility: typeof window === "undefined" ? "hidden" : "visible",
  };

  return (
    <time style={style} dateTime={currentTime} suppressHydrationWarning>
      {currentTime}
    </time>
  );
};
