/** One overview read feeds the shell and its active section. */
import { createContext, useContext } from "react";

import { mainApi } from "../api/commands";
import type { Overview } from "../api/types";
import { type Polled, usePolling } from "../components/hooks";

/** Standalone views keep working in tests without mounting the entire shell. */
const OVERVIEW_INTERVAL_MS = 3000;
/** The shell owns polling while its child sections consume this snapshot. */
export const OverviewContext = createContext<Polled<Overview> | null>(null);

/** Uses the shell's snapshot, with a standalone fallback for isolated views. */
export function useOverview(): Polled<Overview> {
  const shared = useContext(OverviewContext);
  const standalone = usePolling(mainApi.overview, OVERVIEW_INTERVAL_MS, shared === null);
  return shared ?? standalone;
}
