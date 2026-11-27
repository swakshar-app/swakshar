/**
 * Hooks for talking to the backend: polling views and one-off actions.
 */
import { useCallback, useEffect, useRef, useState } from "react";

import { errorText } from "../api/commands";
import { useWindowVisible } from "./visibility";

/** Latest polled value. */
export interface Polled<T> {
  readonly data: T | null;
  readonly error: string | null;
  readonly refresh: () => void;
}

/**
 * Calls `load` now and every `intervalMs` while the window is on screen, and
 * again as soon as it comes back; `load` must be a stable function. A hidden
 * window stops polling, so a closed Swakshar does not keep reading the token.
 */
export function usePolling<T>(load: () => Promise<T>, intervalMs: number, enabled = true): Polled<T> {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);
  const runNow = useRef<() => void>(() => undefined);
  const inFlight = useRef<{ readonly load: () => Promise<T>; readonly promise: Promise<T> } | null>(null);
  const visible = useWindowVisible();
  useEffect(() => {
    if (!visible || !enabled) {
      return undefined;
    }
    let active = true;
    let pending = false;
    let timer: number | undefined;
    const run = (): void => {
      if (!active || pending) { return; }
      pending = true;
      window.clearTimeout(timer);
      const request = inFlight.current?.load === load ? inFlight.current.promise : load();
      inFlight.current = { load, promise: request };
      request.then(
        (value) => {
          if (active) {
            setData(value);
            setError(null);
          }
        },
        (reason: unknown) => {
          if (active) {
            setError(errorText(reason));
          }
        },
      ).finally(() => {
        if (inFlight.current?.promise === request) { inFlight.current = null; }
        pending = false;
        if (active) { timer = window.setTimeout(run, intervalMs); }
      });
    };
    runNow.current = run;
    run();
    return () => {
      active = false;
      window.clearTimeout(timer);
      runNow.current = () => undefined;
    };
  }, [load, intervalMs, visible, enabled]);
  const refresh = useCallback(() => runNow.current(), []);
  return { data, error, refresh };
}

/** A one-off action with a busy flag and an error message. */
export interface Action {
  readonly busy: boolean;
  readonly error: string | null;
  readonly run: (action: () => Promise<unknown>) => Promise<void>;
}

/** Runs actions one at a time and keeps the last error. */
export function useAction(): Action {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const pending = useRef(false);
  const run = useCallback(async (action: () => Promise<unknown>): Promise<void> => {
    if (pending.current) { return; }
    pending.current = true;
    setBusy(true);
    setError(null);
    try {
      await action();
    } catch (reason: unknown) {
      setError(errorText(reason));
    } finally {
      pending.current = false;
      setBusy(false);
    }
  }, []);
  return { busy, error, run };
}
