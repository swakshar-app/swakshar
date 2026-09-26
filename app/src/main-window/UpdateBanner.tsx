/**
 * The update banner on Home: what is new, the download, then Restart now or
 * Restart when idle. Hidden when there is nothing to offer.
 */
import type { ReactElement } from "react";

import { mainApi } from "../api/commands";
import type { UpdateView } from "../api/types";
import { useAction } from "../components/hooks";
import { Icon } from "../components/Icon";
import { Button, Notice } from "../components/ui";

/** Headline and explanation for each state the banner shows. */
function describe(update: UpdateView): { readonly title: string; readonly detail: string } | null {
  const version = update.version ?? "";
  switch (update.state) {
    case "available":
      return { title: `Swakshar ${version} is available`, detail: "Download it now; you choose when to restart." };
    case "downloading":
      return { title: `Downloading Swakshar ${version}`, detail: update.progress === null ? "Downloading." : `${String(update.progress)}% downloaded.` };
    case "ready":
      return {
        title: `Swakshar ${version} is ready`,
        detail: update.restartWhenIdle
          ? "Swakshar will restart as soon as no signing is in progress."
          : "Restart now, or let Swakshar restart when no signing is in progress.",
      };
    case "failed":
      return { title: "The update could not be downloaded", detail: update.error ?? "Try again in a moment." };
    default:
      return null;
  }
}

/** The banner, or nothing. */
export function UpdateBanner(props: { readonly update: UpdateView; readonly onChange: () => void }): ReactElement | null {
  const { update, onChange } = props;
  const action = useAction();
  const text = describe(update);
  if (text === null) {
    return null;
  }
  const run = (work: () => Promise<unknown>): void => {
    void action.run(async () => {
      await work();
      onChange();
    });
  };
  return (
    <section className="update">
      <span className="update-mark"><Icon name="download" size={18} /></span>
      <div className="update-text">
        <h2>{text.title}</h2>
        <p className="muted">{text.detail}</p>
        {update.state === "downloading" && update.progress !== null ? (
          <div className="progress" role="progressbar" aria-valuenow={update.progress} aria-valuemin={0} aria-valuemax={100}>
            <div className="progress-fill" style={{ width: `${String(update.progress)}%` }} />
          </div>
        ) : null}
        {update.notes === null || update.notes.trim() === "" ? null : (
          <details className="details">
            <summary>What's new</summary>
            <p className="notes">{update.notes}</p>
          </details>
        )}
        {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
      </div>
      <div className="update-actions">
        {update.state === "available" || update.state === "failed" ? (
          <Button variant="primary" icon="download" onClick={() => run(mainApi.downloadUpdate)} disabled={action.busy}>
            {update.state === "failed" ? "Try again" : "Download update"}
          </Button>
        ) : null}
        {update.state === "ready" ? (
          <>
            <Button variant="primary" onClick={() => run(() => mainApi.restartToUpdate(false))} disabled={action.busy}>Restart now</Button>
            <Button onClick={() => run(() => mainApi.restartToUpdate(true))} disabled={action.busy || update.restartWhenIdle}>Restart when idle</Button>
          </>
        ) : null}
      </div>
    </section>
  );
}
