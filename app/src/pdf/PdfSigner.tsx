/** Certificate and PIN controls for one immutable PDF selection. */
import { type ReactElement, useEffect, useState } from "react";

import { errorText } from "../api/commands";
import { signingApi, type PdfBox } from "../api/signing";
import { PinField } from "../approve/PinField";
import { Button, Card, Notice } from "../components/ui";
import { isUsableBox } from "./placement";
import { useCertificates } from "./useCertificates";

/** Keeps sensitive input and fresh PIN flags separate from preview rendering. */
export function PdfSigner({ documentId, page, box, ready, onBusy, onSigner }: {
  readonly documentId: number; readonly page: number; readonly box: PdfBox | null;
  readonly ready: boolean; readonly onBusy: (busy: boolean) => void; readonly onSigner: (holder: string) => void;
}): ReactElement {
  const [fingerprint, setFingerprint] = useState("");
  const [pin, setPin] = useState("");
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);
  const [saved, setSaved] = useState<string | null>(null);
  const [needsRefresh, setNeedsRefresh] = useState(false);
  const [allowLastTry, setAllowLastTry] = useState(false);
  const status = useCertificates();
  const choices = status.data?.certificates ?? [];
  const selected = choices.find((certificate) => certificate.fingerprint === fingerprint) ?? null;
  useEffect(() => { onSigner(selected?.holder ?? "Certificate holder"); }, [selected?.holder, onSigner]);
  const canSign = !busy && !needsRefresh && ready && box !== null && isUsableBox(box)
    && selected !== null && !selected.locked && status.data?.available === true
    && (selected.protectedPath || pin.length > 0) && (!selected.finalTry || allowLastTry);

  /** Clears the PIN immediately and never retries after a failed token operation. */
  async function sign(): Promise<void> {
    if (!canSign || box === null || selected === null) { return; }
    const submittedPin = selected.protectedPath ? null : pin;
    setPin(""); setBusy(true); onBusy(true); setFailure(null); setSaved(null);
    try {
      setSaved(await signingApi.signPdf({ documentId, fingerprint, finalTryAcknowledged: selected.finalTry && allowLastTry, placement: { page, rect: box } }, submittedPin));
    } catch (reason: unknown) { setFailure(errorText(reason)); setNeedsRefresh(true); }
    finally { setBusy(false); onBusy(false); setAllowLastTry(false); }
  }

  return <Card title="Signing certificate" actions={<Button disabled={busy || status.loading} onClick={() => {
    setPin(""); setAllowLastTry(false); void status.refresh().then(() => setNeedsRefresh(false));
  }}>{status.loading ? "Reading token" : "Refresh"}</Button>}>
    {status.error === null ? null : <Notice tone="warn">{status.error}</Notice>}
    {status.data?.available === false ? <Notice tone="danger">The bundled PDF signing engine is missing from this build. Reinstall Swakshar.</Notice> : null}
    {status.loading ? <p className="muted" role="status">Reading certificates from your token.</p> : choices.length === 0 ? <p>Connect a DSC token with a valid signing certificate, then refresh.</p> : <label className="field">
      <span>Certificate</span><select disabled={busy} value={fingerprint} onChange={(event) => {
        setFingerprint(event.target.value); setPin(""); setSaved(null); setAllowLastTry(false);
      }}><option value="">Choose a certificate</option>{choices.map((certificate) => <option key={certificate.fingerprint} value={certificate.fingerprint}>
        {certificate.holder} | {certificate.token}
      </option>)}</select>
    </label>}
    {selected?.finalTry === true && !selected.locked ? <Notice tone="warn">
      <p>One more wrong PIN will lock this token.</p>
      <label className="field-inline"><input type="checkbox" checked={allowLastTry} disabled={busy}
        onChange={(event) => setAllowLastTry(event.target.checked)} /> I understand this is the last PIN attempt</label>
    </Notice> : null}
    {selected === null ? null : selected.locked ? <Notice tone="danger">This token PIN is locked. Contact the token supplier.</Notice> : selected.protectedPath ? <p>Enter the PIN on your token's PIN pad when prompted.</p> : <fieldset disabled={busy} className="pdf-pin">
      <PinField value={pin} onChange={setPin} finalTry={false} />
    </fieldset>}
    {needsRefresh ? <Notice tone="warn">Refresh certificates to read the latest PIN lockout flags before another attempt.</Notice> : null}
    {failure === null ? null : <Notice tone="danger">{failure}</Notice>}
    {saved === null ? null : <Notice tone="success">Signed PDF saved to {saved}</Notice>}
    <div className="row">
      <Button variant="primary" disabled={!canSign} onClick={() => { void sign(); }}>{busy ? "Signing PDF" : "Sign and save a copy"}</Button>
      {!busy ? null : <Button onClick={() => { void signingApi.cancelPdf(documentId).catch((reason: unknown) => setFailure(errorText(reason))); }}>Cancel signing</Button>}
    </div>
    {!ready || box === null ? <p className="muted">Place a signature on the page to continue.</p> : !isUsableBox(box) ? <p className="muted">Draw a larger signature box.</p> : null}
    <p className="muted pdf-hint">The signature date is recorded in UTC. Timestamping and long-term validation data are not added.</p>
  </Card>;
}
