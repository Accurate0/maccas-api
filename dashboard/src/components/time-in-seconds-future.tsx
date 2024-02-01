"use client";

import { CSSProperties, useEffect, useState } from "react";

export const TimeSecondsInFuture = ({
  secondsInFuture,
}: {
  secondsInFuture: number;
}) => {
  const date = new Date(Date.now() + secondsInFuture * 1000);
  const adjustedDateTime = new Intl.DateTimeFormat("en-AU", {
    timeZone: "Australia/Perth",
    timeStyle: "medium",
    dateStyle: "long",
  }).format(date);

  return (
    <time dateTime={adjustedDateTime} suppressHydrationWarning>
      {adjustedDateTime}
    </time>
  );
};
