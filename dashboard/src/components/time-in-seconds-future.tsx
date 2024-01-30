"use client";

export const TimeSecondsInFuture = ({
  secondsInFuture,
}: {
  secondsInFuture: number;
}) => {
  const dateTime = new Date(Date.now() + secondsInFuture * 1000);

  return (
    <time dateTime={dateTime.toISOString()}>
      {dateTime.toLocaleString("en-AU")}
    </time>
  );
};
