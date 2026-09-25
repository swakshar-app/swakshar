/**
 * Hooks for talking to the backend: polling views and one-off actions.
 */
import { useCallback, useEffect, useRef, useState } from "react";

import { errorText } from "../api/commands";

/** Latest polled value. */
export interface Polled<T> {
  readonly data: T | null;
  readonly error: string | null;
  readonly refresh: () => void;
}

/** Calls `load` now and every `intervalMs`; `load` must be a stable function. */
export function usePolling<T>(load: () => Promise<T>, intervalMs: number): Polled<T> {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);
  const runNow = useRef<() => void>(() => undefined);
  useEffect(() => {
    let active = true;
    const run = (): void => {
      load().then(
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
      );
    };
    runNow.current = run;
    run();
    const timer = window.setInterval(run, intervalMs);
    return () => {
      active = false;
      window.clearInterval(timer);
      runNow.current = () => undefined;
    };
  }, [load, intervalMs]);
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
  const run = useCallback(async (action: () => Promise<unknown>): Promise<void> => {
    setBusy(true);
    setError(null);
    try {
      await action();
    } catch (reason: unknown) {
      setError(errorText(reason));
    } finally {
      setBusy(false);
    }
  }, []);
  return { busy, error, run };
}
