/**
 * The one way this app subscribes to backend events. Tauri's own `listen`
 * hears an event sent to any window, so the main window once took the
 * approval window's "hidden" as its own and stopped refreshing. Every
 * subscription made here is addressed to the current window only.
 */
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type { UnlistenFn };

/** Subscribes to `event` as sent to this window, handing over its payload. */
export function listenHere<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(event, (received) => handler(received.payload), { target: getCurrentWindow().label });
}
