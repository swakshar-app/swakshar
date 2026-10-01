/**
 * Whether this window is on screen. The app announces every show and hide,
 * the window asks once when first used (a start at login never shows it), and
 * the page's own visibility covers minimising. Views poll only while visible.
 */
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSyncExternalStore } from "react";

/** Event the app sends when it shows or hides this window. */
const EVENT_VISIBILITY = "window-visibility";

/** Global the Tauri shell injects; missing in tests and plain browsers. */
const TAURI_INTERNALS = "__TAURI_INTERNALS__";

/** Last show or hide the app announced; unknown until the first answer. */
let shown = false;
/** Listening has started. */
let started = false;
/** Components waiting for changes. */
const listeners = new Set<() => void>();

/** Wakes every subscriber. */
function notify(): void {
  for (const listener of listeners) {
    listener();
  }
}

/**
 * Starts listening on first use, not on import. Outside the app shell there
 * is no window to ask, so the page's own visibility decides. Only events sent
 * to this window count: a plain `listen` hears every window's, and the
 * approval window hiding after a signature would stop the main window too.
 */
function start(): void {
  if (started) {
    return;
  }
  started = true;
  document.addEventListener("visibilitychange", notify);
  if (!(TAURI_INTERNALS in window)) {
    shown = true;
    return;
  }
  const current = getCurrentWindow();
  current.isVisible().then(
    (visible) => {
      shown = visible;
      notify();
    },
    () => undefined,
  );
  void listen<boolean>(
    EVENT_VISIBILITY,
    (event) => {
      shown = event.payload;
      notify();
    },
    { target: current.label },
  );
}

/** Registers a subscriber. */
function subscribe(listener: () => void): () => void {
  start();
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
