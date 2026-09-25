/**
 * The headline: ready, paused, waiting, or what to fix.
 */
import type { ReactElement } from "react";

import { mainApi } from "../api/commands";
import type { Overview } from "../api/types";
import { useAction } from "../components/hooks";
import { Button, Notice, Tag, type Tone } from "../components/ui";

/** Headline, explanation and tone for the current state. */
function describe(overview: Overview): { readonly title: string; readonly detail: string; readonly tone: Tone } {
  const { server, trust, waiting } = overview;
  if (waiting) {
    return { title: "Waiting for your approval", detail: "Answer the request in the signature window.", tone: "warn" };
  }
  switch (server.state) {
    case "failed":
      return { title: "Swakshar is not running", detail: server.error ?? "The signer could not start.", tone: "danger" };
    case "paused":
      return { title: "Signing is paused", detail: "The GST portal cannot reach Swakshar until you resume.", tone: "neutral" };
    case "starting":
      return { title: "Starting", detail: "Preparing the local certificate and a port.", tone: "neutral" };
    case "running":
      return trust.status === "trusted"
        ? { title: "Ready for the GST portal", detail: `Listening on 127.0.0.1:${String(server.port ?? "")}. Sign in to the portal and use your DSC as usual.`, tone: "success" }
        : { title: "Finish setup", detail: "Your browser must trust Swakshar's local certificate before the portal can connect.", tone: "warn" };
  }
}

/** The status card at the top of Home. */
export function StatusHeader({ overview, onChange }: { readonly overview: Overview; readonly onChange: () => void }): ReactElement {
  const action = useAction();
  const { title, detail, tone } = describe(overview);
  const paused = overview.server.state === "paused" || overview.server.state === "failed";
  const toggle = (): void => {
    void action.run(async () => {
      await mainApi.setPaused(!paused);
      onChange();
    });
  };
  return (
    <section className={`status status-${tone}`}>
      <div className="status-text">
        <Tag tone={tone}>{overview.server.state === "running" ? "Signer on" : "Signer off"}</Tag>
        <h1>{title}</h1>
        <p>{detail}</p>
      </div>
      <div className="status-actions">
        <Button icon={paused ? "play" : "pause"} onClick={toggle} disabled={action.busy || overview.server.state === "starting"}>
          {paused ? "Resume signing" : "Pause signing"}
        </Button>
      </div>
      {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
    </section>
  );
}
