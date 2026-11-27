/** Income Tax signing through the installed emBridge companion. */
import type { ReactElement } from "react";

import { signingApi } from "../api/signing";
import { useAction, usePolling } from "../components/hooks";
import { Button, Card, Notice, Tag } from "../components/ui";

/** Poll gently because a TLS check can spawn a system tool. */
const REFRESH_MS = 8000;

/** Explicit service controls and the portal's setup requirements. */
export function PortalView(): ReactElement {
  const status = usePolling(signingApi.bridgeStatus, REFRESH_MS);
  const action = useAction();
  const bridge = status.data;
  const ready = bridge?.loaded === true && bridge.tlsReady && bridge.hostsReady;
  return (
    <div className="page">
      <header className="page-header"><h1>Income Tax portal</h1><p className="muted">Manage the local DSC service used by the portal.</p></header>
      {status.error === null ? null : <Notice tone="danger">{status.error}</Notice>}
      {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
      <Card title="emBridge" actions={<Button disabled={action.busy} onClick={status.refresh}>Refresh</Button>}>
        {bridge === null ? <p>Reading service status.</p> : <>
          <Tag tone={ready ? "success" : "warn"}>{ready ? "Ready for portal signing" : "Setup needed"}</Tag>
          {!bridge.supported ? <p>Service controls are currently available on macOS. Use the vendor utility on other systems.</p> : !bridge.installed ? <>
            <p>Install the bundled emBridge companion once. macOS guides you through the installation, then Swakshar manages its service here.</p>
            <Button disabled={!bridge.installerAvailable} onClick={() => { void action.run(signingApi.bridgeInstall); }}>Install emBridge</Button>
            {bridge.installerAvailable ? null : <Notice tone="warn">The offline installer is missing from this build. Reinstall Swakshar or use the official download.</Notice>}
            <Button onClick={() => { void action.run(signingApi.bridgeDownload); }}>Official download page</Button>
          </> : <>
            <dl className="portal-checks">
              <dt>Service</dt><dd>{bridge.loaded ? "Loaded" : "Stopped"}</dd>
              <dt>Port 26769</dt><dd>{bridge.listening ? "Listening" : "Not listening"}</dd>
              <dt>HTTPS certificate</dt><dd>{bridge.tlsReady ? "Trusted" : "Not ready. Finish setup in emBridge."}</dd>
              <dt>Local hostname</dt><dd>{bridge.hostsReady ? "Mapped to this Mac" : "Missing. Finish setup in emBridge."}</dd>
            </dl>
            {bridge.listening && !bridge.loaded ? <Notice tone="warn">Another service may be using port 26769. Close it before starting emBridge.</Notice> : null}
            <div className="actions">
              <Button variant="primary" disabled={action.busy || (bridge.listening && !bridge.loaded)} onClick={() => {
                void action.run(async () => { await signingApi.bridgeControl("start"); status.refresh(); });
              }}>Start emBridge</Button>
              <Button disabled={action.busy || !bridge.loaded} onClick={() => {
                void action.run(async () => { await signingApi.bridgeControl("stop"); status.refresh(); });
              }}>Stop emBridge</Button>
            </div>
            <p className="muted">macOS asks for administrator approval. Stop also disables the service at the next reboot.</p>
          </>}
        </>}
      </Card>
      <Card title="Sign on the portal">
        <ol><li>Connect your DSC token and start emBridge.</li><li>On the Income Tax portal, allow local network access when your browser asks.</li><li>Select your certificate and approve the signature in emBridge.</li></ol>
        <p className="muted">emBridge handles these portal signatures and its own PIN prompt. Its service stays running when Swakshar closes.</p>
      </Card>
    </div>
  );
}
