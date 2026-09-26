/**
 * Home: whether Swakshar is ready, guided setup until it is, and the tokens.
 */
import type { ReactElement } from "react";

import { mainApi } from "../api/commands";
import { usePolling } from "../components/hooks";
import { Notice } from "../components/ui";
import { SetupSteps } from "./SetupSteps";
import { StatusHeader } from "./StatusHeader";
import { TokensPanel } from "./TokensPanel";

/** How often the status refreshes. */
const OVERVIEW_INTERVAL_MS = 3000;
/** How often tokens are re-read (plugging and unplugging). */
const TOKENS_INTERVAL_MS = 4000;

/** The Home section. */
export function HomeView(): ReactElement {
  const overview = usePolling(mainApi.overview, OVERVIEW_INTERVAL_MS);
  const tokens = usePolling(mainApi.tokens, TOKENS_INTERVAL_MS);
  if (overview.data === null) {
    return <div className="page">{overview.error === null ? <p className="muted">Loading.</p> : <Notice tone="danger">{overview.error}</Notice>}</div>;
  }
  return (
    <div className="page">
      <StatusHeader overview={overview.data} onChange={overview.refresh} />
      {overview.data.onboardingComplete ? null : (
        <SetupSteps overview={overview.data} tokens={tokens.data} onChange={overview.refresh} onTokensChange={tokens.refresh} />
      )}
      <TokensPanel tokens={tokens.data} error={tokens.error} showGuide={overview.data.onboardingComplete} onChange={tokens.refresh} />
    </div>
  );
}
