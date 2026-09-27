/**
 * The headline: ready, paused, waiting, or what to fix.
 */
import type { ReactElement } from "react";

import { mainApi } from "../api/commands";
import type { Overview } from "../api/types";
import { useAction } from "../components/hooks";
import { Button, Notice, Tag } from "../components/ui";
import { statusText } from "./statusText";

/** The status card at the top of Home. */
export function StatusHeader({ overview, onChange }: { readonly overview: Overview; readonly onChange: () => void }): ReactElement {
  const action = useAction();
  const { title, detail, tone } = statusText(overview);
  const off = overview.server.state === "paused" || overview.server.state === "failed";
  const needsTrust = overview.onboardingComplete && overview.server.state === "running" && overview.trust.status === "not-trusted";
  const run = (work: () => Promise<unknown>): void => {
    void action.run(async () => {
      await work();
      onChange();
    });
  };
  return (
    <section className={`status status-${tone}`}>
      <div className="status-text">
        <Tag tone={tone}>{overview.server.state === "running" ? "Signing on" : "Signing off"}</Tag>
        <h1>{title}</h1>
        <p>{detail}</p>
      </div>
      <div className="status-actions">
        {needsTrust ? (
          <Button variant="primary" icon="shield" onClick={() => run(mainApi.installTrust)} disabled={action.busy}>
            Install certificate
          </Button>
        ) : null}
        <Button
          variant={off ? "primary" : "secondary"}
          icon={off ? "play" : "pause"}
          onClick={() => run(() => mainApi.setPaused(!off))}
          disabled={action.busy || overview.server.state === "starting"}
        >
          {off ? "Turn on signing" : "Turn off signing"}
        </Button>
      </div>
      {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
    </section>
  );
}
