/**
 * Inline stroke icons on a 24px grid, drawn with currentColor. No icon font
 * and no network requests.
 */
import type { ReactElement } from "react";

/** Available icons. */
export type IconName =
  | "home"
  | "clock"
  | "sliders"
  | "book"
  | "lock"
  | "check"
  | "alert"
  | "token"
  | "shield"
  | "pause"
  | "play"
  | "external"
  | "close"
  | "refresh";

/** Path data per icon. */
const PATHS: Record<IconName, readonly string[]> = {
  home: ["M3 11 12 4l9 7", "M5 10v10h14V10"],
  clock: ["M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z", "M12 7v5l3 2"],
  sliders: ["M4 7h9", "M17 7h3", "M4 17h3", "M11 17h9", "M13 5v4", "M7 15v4"],
  book: ["M5 4h11a3 3 0 0 1 3 3v13H8a3 3 0 0 1-3-3V4Z", "M5 17a3 3 0 0 1 3-3h11"],
  lock: ["M6 11h12v9H6z", "M8 11V8a4 4 0 0 1 8 0v3"],
  check: ["M5 12.5 10 17l9-10"],
  alert: ["M12 4 2.5 20h19L12 4Z", "M12 10v4", "M12 16.8v0.4"],
  token: ["M8 3h8v6H8z", "M6 9h12v8a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V9Z", "M10 5v2", "M14 5v2"],
  shield: ["M12 3 5 6v5c0 4.5 3 8 7 10 4-2 7-5.5 7-10V6l-7-3Z"],
  pause: ["M9 5v14", "M15 5v14"],
  play: ["M8 5l11 7-11 7V5Z"],
  external: ["M14 4h6v6", "M20 4l-9 9", "M18 14v6H4V6h6"],
  close: ["M6 6l12 12", "M18 6 6 18"],
  refresh: ["M20 11a8 8 0 1 0-2.3 5.7", "M20 4v7h-7"],
};

/** One icon. Decorative: screen readers skip it; pair it with text. */
export function Icon({ name, size = 18 }: { readonly name: IconName; readonly size?: number }): ReactElement {
  return (
    <svg
      className="icon"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.8}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {PATHS[name].map((path) => (
        <path key={path} d={path} />
      ))}
    </svg>
  );
}
