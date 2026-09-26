/**
 * Help's Updates card: the running version, what the last check found, and
 * Check now. Works even when automatic checks are off.
 */
import type { ReactElement } from "react";

import { mainApi } from "../api/commands";
import type { UpdateView } from "../api/types";
import { formatTime } from "../components/format";
import { useAction, usePolling } from "../components/hooks";
import { Button, Card, Notice } from "../components/ui";

/** How often the card refreshes. */
const UPDATES_INTERVAL_MS = 3000;

/** One sentence about the update state. */
function summary(update: UpdateView): string {
  const checked = update.lastChecked === null ? "" : ` Last checked ${formatTime(update.lastChecked)}.`;
  switch (update.state) {
    case "unavailable":
      return "Updates are not set up in this build.";
    case "checking":
      return "Checking for updates.";
    case "available":
    case "downloading":
    case "ready":
    case "failed":
      return `Swakshar ${update.version ?? ""} is available. Home has the details.`;
    case "idle":
      return update.lastChecked === null ? "Not checked yet." : `You have the latest version.${checked}`;
  }
}

/** The Updates card. */
export function UpdatesCard(): ReactElement {
  const overview = usePolling(mainApi.overview, UPDATES_INTERVAL_MS);
  const action = useAction();
  const update = overview.data?.update ?? null;
  const check = (): void => {
    void action.run(async () => {
      await mainApi.checkForUpdate();
      overview.refresh();
    });
  };
  return (
    <Card title="Updates">
      <p>
        Version {overview.data?.appVersion ?? ""}. {update === null ? "" : summary(update)}
      </p>
      {update?.state === "idle" && update.error !== null ? <Notice tone="warn">The last check failed: {update.error}</Notice> : null}
      <div className="row">
        <Button icon="refresh" onClick={check} disabled={action.busy || update === null || update.state === "unavailable" || update.state === "checking" || update.state === "downloading"}>
          Check now
        </Button>
      </div>
      {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
    </Card>
  );
}
