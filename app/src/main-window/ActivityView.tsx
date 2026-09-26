/**
 * Activity: every request this Mac answered, stored only on this Mac.
 */
import { type ReactElement, useEffect, useState } from "react";

import { mainApi } from "../api/commands";
import type { ActivityEntry } from "../api/types";
import { formatTime, hostOf } from "../components/format";
import { useAction, usePolling } from "../components/hooks";
import { Button, Card, Notice, Tag, type Tone } from "../components/ui";

/** How often the list refreshes. */
const ACTIVITY_INTERVAL_MS = 4000;
/** How long "Press again to clear" waits for the second press. */
const CONFIRM_WINDOW_MS = 4000;

/** Label and tone per outcome. */
const OUTCOMES: Record<string, { readonly label: string; readonly tone: Tone }> = {
  signed: { label: "Signed", tone: "success" },
  canceled: { label: "Declined", tone: "neutral" },
  "timed-out": { label: "Timed out", tone: "warn" },
  abandoned: { label: "Portal closed", tone: "neutral" },
  refused: { label: "Refused", tone: "danger" },
};

/** One row. */
function Row({ entry }: { readonly entry: ActivityEntry }): ReactElement {
  const outcome = OUTCOMES[entry.outcome] ?? { label: entry.outcome, tone: "neutral" as const };
  const what = entry.kind === "registration" ? `DSC registration${entry.panMasked === null ? "" : ` for ${entry.panMasked}`}` : "Document";
  return (
    <tr>
      <td>{formatTime(entry.at)}</td>
      <td>{hostOf(entry.origin)}</td>
      <td>{what}</td>
      <td>{entry.holder === null ? "" : `${entry.holder} (${entry.token ?? ""})`}</td>
      <td>
        <Tag tone={outcome.tone}>{outcome.label}</Tag>
      </td>
    </tr>
  );
}

/** The Activity section. */
export function ActivityView(): ReactElement {
  const activity = usePolling(mainApi.activity, ACTIVITY_INTERVAL_MS);
  const action = useAction();
  const [confirming, setConfirming] = useState(false);
  useEffect(() => {
    if (!confirming) {
      return undefined;
    }
    const timer = window.setTimeout(() => setConfirming(false), CONFIRM_WINDOW_MS);
    return () => window.clearTimeout(timer);
  }, [confirming]);
  const clear = (): void => {
    if (!confirming) {
      setConfirming(true);
      return;
    }
    setConfirming(false);
    void action.run(async () => {
      await mainApi.clearActivity();
      activity.refresh();
    });
  };
  const entries = activity.data ?? [];
  return (
    <div className="page">
      <Card
        title="Activity"
        actions={
          <Button variant={confirming ? "danger" : "quiet"} onClick={clear} disabled={action.busy || entries.length === 0}>
            {confirming ? "Press again to clear" : "Clear history"}
          </Button>
        }
      >
        <p className="muted">Kept only on this Mac. No PINs, signatures or full PANs are stored.</p>
        {activity.error === null ? null : <Notice tone="danger">{activity.error}</Notice>}
        {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
        {entries.length === 0 ? (
          <p className="muted">No requests yet.</p>
        ) : (
          <table className="table">
            <thead>
              <tr>
                <th>When</th>
                <th>Site</th>
                <th>What</th>
                <th>Certificate</th>
                <th>Result</th>
              </tr>
            </thead>
            <tbody>
              {entries.map((entry, index) => (
                <Row key={`${String(entry.at)}-${String(index)}`} entry={entry} />
              ))}
            </tbody>
          </table>
        )}
      </Card>
    </div>
  );
}
