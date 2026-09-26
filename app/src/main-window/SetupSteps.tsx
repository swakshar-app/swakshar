/**
 * Guided setup: token, certificate, local certificate trust, portal access.
 */
import type { ReactElement, ReactNode } from "react";

import { mainApi } from "../api/commands";
import type { InventoryView, Overview } from "../api/types";
import { useAction } from "../components/hooks";
import { Icon } from "../components/Icon";
import { Button, Card, Notice } from "../components/ui";
import { TokenGuide } from "./TokenGuide";

/** Number of setup steps. */
const STEP_COUNT = 4;

/** One step's row. */
function Step(props: { readonly index: number; readonly title: string; readonly done: boolean; readonly current: boolean; readonly children: ReactNode }): ReactElement {
  const { index, title, done, current, children } = props;
  return (
    <li className={current ? "step step-current" : "step"}>
      <div className={done ? "step-marker step-marker-done" : "step-marker"}>{done ? <Icon name="check" size={16} /> : String(index)}</div>
      <div className="step-body">
        <p className="step-count">Step {index} of {STEP_COUNT}</p>
        <h3>{title}</h3>
        {current || !done ? children : null}
      </div>
    </li>
  );
}

/** The four setup steps with their actions. */
export function SetupSteps(props: {
  readonly overview: Overview;
  readonly tokens: InventoryView | null;
  readonly onChange: () => void;
  readonly onTokensChange: () => void;
}): ReactElement {
  const { overview, tokens, onChange, onTokensChange } = props;
  const action = useAction();
  const hasToken = (tokens?.tokens.length ?? 0) > 0;
  const hasCertificate = tokens?.tokens.some((token) => token.certificates.some((cert) => cert.valid && cert.signing)) ?? false;
  const trusted = overview.trust.status === "trusted";
  const running = overview.server.state === "running";
  const firstOpen = [hasToken, hasCertificate, trusted].findIndex((done) => !done) + 1 || STEP_COUNT;
  const run = (work: () => Promise<unknown>): void => {
    void action.run(async () => {
      await work();
      onChange();
    });
  };
  return (
    <Card title="Set up Swakshar">
      <ol className="steps">
        <Step index={1} title="Connect your DSC token" done={hasToken} current={firstOpen === 1}>
          <p>Plug in your token. Swakshar recognises common Indian tokens (ePass2003, HYP2003, SafeNet, ProxKey, TrustKey, mToken) and says what, if anything, it still needs.</p>
          <TokenGuide tokens={tokens} onChange={onTokensChange} />
        </Step>
        <Step index={2} title="Check your certificate" done={hasCertificate} current={firstOpen === 2}>
          <p>A valid signing certificate must be on the token. No PIN is needed to read it; it is listed below.</p>
        </Step>
        <Step index={3} title="Let your browser reach Swakshar" done={trusted} current={firstOpen === 3}>
          <p>The portal connects to Swakshar over a secure local connection, so your Mac must trust Swakshar's local certificate. It works only for this computer (127.0.0.1) and cannot be used for any website. macOS will ask for your password.</p>
          <div className="row">
            <Button variant="primary" icon="shield" onClick={() => run(mainApi.installTrust)} disabled={action.busy}>Install certificate</Button>
            {trusted && running ? <Button icon="external" onClick={() => run(mainApi.openStatusPage)}>Test in browser</Button> : null}
          </div>
          {trusted && !running ? <p className="muted">Turn on signing above to test the connection in a browser.</p> : null}
          {overview.trust.command === null ? null : (
            <details className="details">
              <summary>What this runs</summary>
              <code>{overview.trust.command}</code>
            </details>
          )}
        </Step>
        <Step index={4} title="Allow Swakshar on the GST portal" done={false} current={firstOpen === STEP_COUNT}>
          <p>The first time the portal asks for your DSC, Chrome, Edge and Brave ask whether the site may connect to apps on this device. Choose Allow, then press the portal's sign button again. Firefox needs nothing more. Quit and reopen the browser once after installing the certificate.</p>
          <Button variant={trusted ? "primary" : "secondary"} onClick={() => run(mainApi.completeOnboarding)} disabled={action.busy || !trusted}>Finish setup</Button>
        </Step>
      </ol>
      {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
    </Card>
  );
}
