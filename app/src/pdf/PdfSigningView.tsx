/** PDF signing with local preview, certificate selection and a fresh output file. */
import { type ReactElement, useEffect, useState } from "react";

import { errorText } from "../api/commands";
import { signingApi, type PdfBox, type PdfDocument } from "../api/signing";
import { PinField } from "../approve/PinField";
import { useAction } from "../components/hooks";
import { Button, Card, Notice } from "../components/ui";
import { PdfPreview } from "./PdfPreview";
import { isUsableBox } from "./placement";
import { usePdf } from "./usePdf";
import { useCertificates } from "./useCertificates";

/** File chooser stays usable even when no token is connected. */
export function PdfSigningView(): ReactElement {
  const [source, setSource] = useState<PdfDocument | null>(null);
  const [signing, setSigning] = useState(false);
  const action = useAction();
  return <div className="page">
    <header className="page-header"><h1>Sign a PDF</h1><p className="muted">Choose a page, draw a signature box and sign with your DSC token.</p></header>
    <Card title={source?.name ?? "Choose a document"} actions={<Button disabled={action.busy || signing} onClick={() => {
      void action.run(async () => { const picked = await signingApi.choosePdf(); if (picked !== null) { setSource(picked); } });
    }}>{source === null ? "Choose PDF" : "Choose another PDF"}</Button>}>
      <p className="muted">Your document stays on this computer. Signing saves a new PDF alongside the original or in a folder you choose.</p>
      {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
    </Card>
    {source === null ? null : <PdfEditor key={source.id} source={source} onBusy={setSigning} />}
  </div>;
}

/** Keeps page and placement tied to one immutable input selection. */
function PdfEditor({ source, onBusy }: { readonly source: PdfDocument; readonly onBusy: (busy: boolean) => void }): ReactElement {
  const { pdf, error } = usePdf(source);
  const [page, setPage] = useState(1);
  const [box, setBox] = useState<PdfBox | null>(null);
  const [fingerprint, setFingerprint] = useState("");
  const [pin, setPin] = useState("");
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);
  const [saved, setSaved] = useState<string | null>(null);
  const [needsRefresh, setNeedsRefresh] = useState(false);
  const [allowLastTry, setAllowLastTry] = useState(false);
  const status = useCertificates();
  useEffect(() => () => { void signingApi.cancelPdf(source.id).catch(() => undefined); }, [source.id]);
  const choices = status.data?.certificates ?? [];
  const selected = choices.find((certificate) => certificate.fingerprint === fingerprint) ?? null;
  const canSign = !busy && !needsRefresh && pdf !== null && box !== null && isUsableBox(box)
    && selected !== null && !selected.locked && status.data?.available === true
    && (selected.protectedPath || pin.length > 0) && (!selected.finalTry || allowLastTry);

  /** Clears the PIN immediately and never retries after a failed token operation. */
  async function sign(): Promise<void> {
    if (!canSign || box === null || selected === null) { return; }
    const submittedPin = selected.protectedPath ? null : pin;
    setPin(""); setBusy(true); onBusy(true); setFailure(null); setSaved(null);
    try {
      const output = await signingApi.signPdf({ documentId: source.id, fingerprint, finalTryAcknowledged: selected.finalTry && allowLastTry, placement: { page, rect: box } }, submittedPin);
      setSaved(output);
    } catch (reason: unknown) {
      setFailure(errorText(reason));
      setNeedsRefresh(true);
    } finally { setBusy(false); onBusy(false); setAllowLastTry(false); }
  }

  /** Changing page invalidates the old placement rather than signing off-page. */
  function changePage(next: number): void {
    if (Number.isInteger(next) && next >= 1 && next <= (pdf?.numPages ?? 1)) {
      setPage(next); setBox(null); setSaved(null);
    }
  }

  return <>
    {error === null ? null : <Notice tone="danger">{error}</Notice>}
    {pdf === null ? <p className="muted">Loading PDF preview.</p> : <Card title="Place your signature">
      <div className="actions">
        <Button disabled={busy || page <= 1} onClick={() => changePage(page - 1)}>Previous page</Button>
        <label className="pdf-page-choice">Page <input type="number" min={1} max={pdf.numPages} value={page} disabled={busy}
          onChange={(event) => changePage(event.target.valueAsNumber)} /> of {pdf.numPages}</label>
        <Button disabled={busy || page >= pdf.numPages} onClick={() => changePage(page + 1)}>Next page</Button>
      </div>
      <PdfPreview key={page} pdf={pdf} page={page} box={box} disabled={busy} onBox={setBox} />
      {box === null ? null : <p className="muted">PDF coordinates: {box.join(", ")}. {isUsableBox(box) ? "Placement ready." : "Draw a larger box."}</p>}
    </Card>}
    <Card title="Signing certificate" actions={<Button disabled={busy} onClick={() => {
      setPin(""); setAllowLastTry(false); void status.refresh().then(() => setNeedsRefresh(false));
    }}>Refresh certificates</Button>}>
      {status.error === null ? null : <Notice tone="warn">{status.error}</Notice>}
      {status.data?.available === false ? <Notice tone="danger">The bundled PDF signing engine is missing from this build. Reinstall Swakshar.</Notice> : null}
      {choices.length === 0 ? <p>Connect a DSC token with a valid signing certificate, then refresh.</p> : <label className="field">
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
      <div className="actions">
        <Button variant="primary" disabled={!canSign} onClick={() => { void sign(); }}>{busy ? "Signing PDF" : "Sign and save a copy"}</Button>
        {!busy ? null : <Button onClick={() => { void signingApi.cancelPdf(source.id).catch((reason: unknown) => setFailure(errorText(reason))); }}>Cancel signing</Button>}
      </div>
      <p className="muted">Creates an offline digital signature with your certificate. Timestamping and long-term validation data are not added in this version.</p>
    </Card>
  </>;
}
