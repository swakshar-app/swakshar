/**
 * Help: a live checklist, browser guidance, what portal errors mean, about.
 */
import type { ReactElement } from "react";

import { mainApi } from "../api/commands";
import type { DoctorCheck } from "../api/types";
import { useAction, usePolling } from "../components/hooks";
import { Icon, type IconName } from "../components/Icon";
import { Button, Card, Notice, Tag, type Tone } from "../components/ui";
import { DiagnosticsCard } from "./DiagnosticsCard";
import { UpdatesCard } from "./UpdatesCard";

/** How often the checklist refreshes. */
const DOCTOR_INTERVAL_MS = 6000;

/** Icon per check status. */
const CHECK_ICONS: Record<DoctorCheck["status"], IconName> = { pass: "check", warn: "alert", fail: "close" };

/** Spoken label per check status. */
const CHECK_LABELS: Record<DoctorCheck["status"], string> = { pass: "Passed", warn: "Needs a look", fail: "Needs fixing" };

/** How many checks pass, as a header tag. */
function Summary({ checks }: { readonly checks: readonly DoctorCheck[] }): ReactElement | null {
  if (checks.length === 0) {
    return null;
  }
  const passed = checks.filter((check) => check.status === "pass").length;
  const tone: Tone = passed === checks.length ? "success" : "warn";
  return <Tag tone={tone}>{`${String(passed)} of ${String(checks.length)} pass`}</Tag>;
}

/** Portal messages and what they usually mean. */
const PORTAL_ERRORS: ReadonlyArray<readonly [string, string]> = [
  ["Failed to establish connection to the server. Kindly restart the Emsigner", "Swakshar is paused, the browser does not trust the local certificate yet, or the browser blocked access to apps on this device. Check the list above."],
  ["Please install and use correct version of emSigner", "The portal expects a different greeting version. Set it in Settings."],
  ["Signing Cancelled", "You declined, the request timed out, or no suitable certificate was found."],
  ["registration failed during signature verification", "The certificate does not belong to the PAN on the portal. Choose the matching certificate."],
];

/** The Help section. */
export function HelpView(): ReactElement {
  const doctor = usePolling(mainApi.doctor, DOCTOR_INTERVAL_MS);
  const action = useAction();
  return (
    <div className="page">
      <Card title="Checklist" actions={<Summary checks={doctor.data ?? []} />}>
        {doctor.error === null ? null : <Notice tone="danger">{doctor.error}</Notice>}
        {doctor.data === null && doctor.error === null ? <p className="muted">Loading.</p> : null}
        <ul className="checks">
          {(doctor.data ?? []).map((check) => (
            <li key={check.label} className={`check check-${check.status}`}>
              <span className="check-mark" role="img" aria-label={CHECK_LABELS[check.status]}>
                <Icon name={CHECK_ICONS[check.status]} size={16} />
              </span>
              <div className="check-text">
                <p className="check-label">{check.label}</p>
                <p className="muted">{check.detail}</p>
              </div>
            </li>
          ))}
        </ul>
        <div className="row">
          <Button icon="external" onClick={() => void action.run(mainApi.openStatusPage)} disabled={action.busy}>Open status page in browser</Button>
        </div>
        <p className="muted">If the status page opens without a warning, this browser trusts Swakshar.</p>
        {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
      </Card>
      <Card title="Browsers">
        <p><strong>Chrome, Edge, Brave:</strong> the first time the GST portal reaches Swakshar, the browser asks whether the site may connect to apps on this device. Choose Allow. If you dismissed it, open the site settings from the icon left of the address and allow it there.</p>
        <p><strong>Firefox:</strong> no extra step. Firefox trusts certificates you install in macOS.</p>
        <p><strong>Offices:</strong> administrators can allow the portal for everyone with Chrome's LoopbackNetworkAllowedForUrls policy set to https://[*.]gst.gov.in.</p>
      </Card>
      <Card title="What portal messages mean">
        <dl className="glossary">
          {PORTAL_ERRORS.map(([message, meaning]) => (
            <div key={message}>
              <dt>{message}</dt>
              <dd>{meaning}</dd>
            </div>
          ))}
        </dl>
      </Card>
      <UpdatesCard />
      <DiagnosticsCard />
      <Card title="About">
        <p>Swakshar is open-source software under the Apache License 2.0. It signs only with your own DSC, only after you approve each request, and sends nothing anywhere else.</p>
        <p className="muted">Not affiliated with, endorsed by, or supported by GSTN, the GST Council, Infosys, eMudhra, or any Certifying Authority.</p>
      </Card>
    </div>
  );
}
