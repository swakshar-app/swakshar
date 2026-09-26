/**
 * The waiting request, kept in step with the backend's events.
 */
import { useCallback, useEffect, useState } from "react";

import { approveApi } from "../api/commands";
import type { PendingView } from "../api/types";

/** The current request, and ways to clear or replace it locally. */
export interface RequestState {
  readonly pending: PendingView | null;
  readonly clear: () => void;
  readonly replace: (view: PendingView | null) => void;
}

/** Loads the waiting request and follows `sign-request` / `sign-finished`. */
export function useRequest(onNewRequest: () => void): RequestState {
  const [pending, setPending] = useState<PendingView | null>(null);
  useEffect(() => {
    let active = true;
    approveApi.pending().then(
      (view) => {
        if (active) {
          setPending(view);
        }
      },
      () => undefined,
    );
    const unlistenRequest = approveApi.onRequest((view) => {
      onNewRequest();
      setPending(view);
    });
    const unlistenFinished = approveApi.onFinished((finished) => {
      if (finished.outcome !== "signed") {
        setPending((current) => (current !== null && current.id === finished.id ? null : current));
      }
    });
    return () => {
      active = false;
      void unlistenRequest.then((unlisten) => unlisten());
      void unlistenFinished.then((unlisten) => unlisten());
    };
  }, [onNewRequest]);
  const clear = useCallback(() => setPending(null), []);
  return { pending, clear, replace: setPending };
}

/** Current Unix time, ticking every second while `running`. */
export function useClock(running: boolean): number {
  const [now, setNow] = useState(() => Date.now() / 1000);
  useEffect(() => {
    if (!running) {
      return undefined;
    }
    const timer = window.setInterval(() => setNow(Date.now() / 1000), 1000);
    return () => window.clearInterval(timer);
  }, [running]);
  return now;
}
