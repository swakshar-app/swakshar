/**
 * Settings card for the local certificate: whether this Mac trusts it, and
 * the buttons to install or remove it.
 */
import type { ReactElement } from "react";

import { mainApi } from "../api/commands";
import type { TrustView } from "../api/types";
import { useAction, usePolling } from "../components/hooks";
import { Button, Card, Notice, Tag, type Tone } from "../components/ui";

/** How often the card refreshes. */
const TRUST_INTERVAL_MS = 5000;

/** Label and tone per trust state. */
const STATES: Record<TrustView["status"], { readonly label: string; readonly tone: Tone }> = {
  trusted: { label: "Trusted by this Mac", tone: "success" },
  "not-trusted": { label: "Not trusted yet", tone: "warn" },
  missing: { label: "Not created yet", tone: "neutral" },
  unsupported: { label: "Install manually on this system", tone: "neutral" },
};

/** The Local certificate card. */
export function LocalCertificateCard(): ReactElement {
  const overview = usePolling(mainApi.overview, TRUST_INTERVAL_MS);
  const action = useAction();
  const trust = overview.data?.trust ?? null;
  const run = (work: () => Promise<unknown>): void => {
    void action.run(async () => {
      await work();
      overview.refresh();
    });
  };
  const state = trust === null ? null : STATES[trust.status];
  return (
    <Card title="Local certificate">
      <p className="muted">
        Lets your browser reach Swakshar over a secure local connection. It works only for this computer (127.0.0.1) and cannot vouch for any website; its signing key was never saved.
      </p>
      <div className="row">
        {state === null ? null : <Tag tone={state.tone}>{state.label}</Tag>}
        {trust !== null && trust.validUntil !== null ? <span className="muted">Valid until {trust.validUntil}</span> : null}
      </div>
      <div className="row">
        {trust?.status === "trusted" ? (
          <Button variant="danger" onClick={() => run(mainApi.removeTrust)} disabled={action.busy}>Remove from this Mac</Button>
        ) : (
          <Button variant="primary" icon="shield" onClick={() => run(mainApi.installTrust)} disabled={action.busy || trust?.status === "unsupported"}>Install certificate</Button>
        )}
      </div>
      {trust !== null && trust.command !== null ? (
        <details className="details">
          <summary>What installing runs</summary>
          <code>{trust.command}</code>
        </details>
      ) : null}
      {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
    </Card>
  );
}
