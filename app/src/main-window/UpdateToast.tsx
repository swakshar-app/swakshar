/**
 * The update card at the bottom right of the main window, on every page:
 * See changes, then Download, then Restart when idle or Restart. Dismissing
 * it hides it until the update reaches its next step.
 */
import { type ReactElement, useState } from "react";

import { mainApi } from "../api/commands";
import type { UpdateView } from "../api/types";
import { useAction } from "../components/hooks";
import { Icon } from "../components/Icon";
import { Button, Notice } from "../components/ui";
import { noteBlocks } from "./releaseNotes";

/** States the card appears in. */
const SHOWN: ReadonlySet<UpdateView["state"]> = new Set(["available", "downloading", "ready", "failed"]);

/** Title and explanation for the current state. */
function text(update: UpdateView): { readonly title: string; readonly detail: string } {
  const version = update.version ?? "";
  switch (update.state) {
    case "downloading":
      return { title: `Downloading Swakshar ${version}`, detail: update.progress === null ? "Downloading." : `${String(update.progress)}% downloaded.` };
    case "ready":
      return {
        title: `Swakshar ${version} is ready`,
        detail: update.restartWhenIdle ? "Swakshar restarts as soon as no signing is in progress." : "Restart now, or when no signing is in progress.",
      };
    case "failed":
      return { title: "The update could not be downloaded", detail: update.error ?? "Try again in a moment." };
    default:
      return { title: `Swakshar ${version} is available`, detail: "Download it now; you choose when to restart." };
  }
}

/** Release notes as paragraphs and list items. */
function Notes(props: { readonly text: string }): ReactElement {
  return (
    <div className="notes">
      {noteBlocks(props.text).map((block, index) =>
        block.kind === "paragraph" ? (
          <p key={index}>{block.text}</p>
        ) : (
          <ul key={index}>
            {block.items.map((item, itemIndex) => (
              <li key={itemIndex}>{item}</li>
            ))}
          </ul>
        ),
      )}
    </div>
  );
}

/** The card, or nothing. */
export function UpdateToast(props: { readonly update: UpdateView; readonly onChange: () => void }): ReactElement | null {
  const { update, onChange } = props;
  const action = useAction();
  const [dismissed, setDismissed] = useState<string | null>(null);
  const [notesOpen, setNotesOpen] = useState(false);
  const step = `${update.state}-${update.version ?? ""}`;
  if (!SHOWN.has(update.state) || dismissed === step) {
    return null;
  }
  const run = (work: () => Promise<unknown>): void => {
    void action.run(async () => {
      await work();
      onChange();
    });
  };
  const { title, detail } = text(update);
  const notes = update.notes === null || update.notes.trim() === "" ? null : update.notes;
  return (
    <aside className="toast" aria-live="polite">
      <button type="button" className="toast-close" aria-label="Dismiss" onClick={() => setDismissed(step)}>
        <Icon name="close" size={14} />
      </button>
      <p className="toast-title">{title}</p>
      <p className="muted">{detail}</p>
      {update.state === "downloading" && update.progress !== null ? (
        <div className="progress" role="progressbar" aria-valuenow={update.progress} aria-valuemin={0} aria-valuemax={100}>
          <div className="progress-fill" style={{ width: `${String(update.progress)}%` }} />
        </div>
      ) : null}
      {notesOpen && notes !== null ? <Notes text={notes} /> : null}
      <div className="toast-actions">
        {notes === null ? null : <Button onClick={() => setNotesOpen(!notesOpen)}>{notesOpen ? "Hide changes" : "See changes"}</Button>}
        {update.state === "available" || update.state === "failed" ? (
          <Button variant="primary" icon="download" onClick={() => run(mainApi.downloadUpdate)} disabled={action.busy}>
            {update.state === "failed" ? "Try again" : "Download"}
          </Button>
        ) : null}
        {update.state === "ready" ? (
          <>
            <Button onClick={() => run(() => mainApi.restartToUpdate(true))} disabled={action.busy || update.restartWhenIdle}>Restart when idle</Button>
            <Button variant="primary" onClick={() => run(() => mainApi.restartToUpdate(false))} disabled={action.busy}>Restart</Button>
          </>
        ) : null}
      </div>
      {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
    </aside>
  );
}
