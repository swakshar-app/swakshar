/**
 * Whether this window is on screen. The app announces every show and hide,
 * the window asks once at start (a start at login never shows it), and the
 * page's own visibility covers minimising. Views poll only while visible.
 */
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSyncExternalStore } from "react";

/** Event the app sends when it shows or hides this window. */
const EVENT_VISIBILITY = "window-visibility";

/** Last show or hide the app announced; unknown until the first answer. */
let shown = false;
/** Components waiting for changes. */
const listeners = new Set<() => void>();

/** Wakes every subscriber. */
function notify(): void {
  for (const listener of listeners) {
    listener();
  }
}

getCurrentWindow()
  .isVisible()
  .then(
    (visible) => {
      shown = visible;
      notify();
    },
    () => undefined,
  );
void listen<boolean>(EVENT_VISIBILITY, (event) => {
  shown = event.payload;
  notify();
});
document.addEventListener("visibilitychange", notify);

/** Registers a subscriber. */
function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/** Current visibility. */
function snapshot(): boolean {
  return shown && document.visibilityState === "visible";
}

/** True while this window is on screen. */
export function useWindowVisible(): boolean {
  return useSyncExternalStore(subscribe, snapshot);
}
