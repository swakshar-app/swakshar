/**
 * Display formatting.
 */

/** Milliseconds per second. */
const MS_PER_SECOND = 1000;

/** A Unix time as a short local date and time. */
export function formatTime(unixSeconds: number): string {
  return new Date(unixSeconds * MS_PER_SECOND).toLocaleString(undefined, {
    day: "numeric",
    month: "short",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** The host of an origin, for display. */
export function hostOf(origin: string): string {
  return URL.canParse(origin) ? new URL(origin).host : origin;
}

/** Seconds as `m:ss`. */
export function formatCountdown(seconds: number): string {
  const safe = Math.max(0, Math.floor(seconds));
  return `${Math.floor(safe / 60)}:${String(safe % 60).padStart(2, "0")}`;
}
