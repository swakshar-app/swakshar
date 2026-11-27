/** PDF signing with local preview, certificate selection and a fresh output file. */
import { type ReactElement, useEffect, useState } from "react";

import { signingApi, type PdfBox, type PdfDocument } from "../api/signing";
import { useAction } from "../components/hooks";
import { Button, Card, Notice } from "../components/ui";
import { PdfPreview } from "./PdfPreview";
import { PdfSigner } from "./PdfSigner";
import { usePdf } from "./usePdf";

/** File chooser stays usable even when no token is connected. */
export function PdfSigningView(): ReactElement {
  const [source, setSource] = useState<PdfDocument | null>(null);
  const [signing, setSigning] = useState(false);
  const action = useAction();
  return <div className="page pdf-page">
    <header className="page-header"><h1>Sign a PDF</h1><p className="muted">Choose a page, draw a signature box and sign with your DSC token.</p></header>
    <Card title={source?.name ?? "Choose a document"} actions={<Button disabled={action.busy || signing} onClick={() => {
      void action.run(async () => { const picked = await signingApi.choosePdf(); if (picked !== null) { setSource(picked); } });
    }}>{action.busy ? "Opening PDF" : source === null ? "Choose PDF" : "Choose another PDF"}</Button>}>
      <p className="muted">{source === null ? "Your document stays on this computer. Signing saves a new copy." : `${(source.size / (1024 * 1024)).toFixed(1)} MB. The original stays unchanged.`}</p>
      {action.busy ? <p className="muted" role="status">Choose a PDF in the file picker. Large files may take a moment to read.</p> : null}
      {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
    </Card>
    {source === null ? null : <PdfEditor key={source.id} source={source} onBusy={setSigning} />}
  </div>;
}

/** Keeps page and placement tied to one immutable input selection. */
function PdfEditor({ source, onBusy }: { readonly source: PdfDocument; readonly onBusy: (busy: boolean) => void }): ReactElement {
  const { pdf, error, stage } = usePdf(source);
  const [page, setPage] = useState(1);
  const [box, setBox] = useState<PdfBox | null>(null);
  const [busy, setBusy] = useState(false);
  const [signer, setSigner] = useState("Certificate holder");
  useEffect(() => () => { void signingApi.cancelPdf(source.id).catch(() => undefined); }, [source.id]);

  /** Changing page invalidates the old placement rather than signing off-page. */
  function changePage(next: number): void {
    if (Number.isInteger(next) && next >= 1 && next <= (pdf?.numPages ?? 1)) {
      setPage(next); setBox(null);
    }
  }

  return <>
    {error === null ? null : <Notice tone="danger">{error}</Notice>}
    <div className="pdf-workspace">
    {pdf === null ? error === null ? <div className="pdf-loading" role="status">{stage}.</div> : null : <Card title="Place your signature">
      <div className="row pdf-toolbar">
        <Button disabled={busy || page <= 1} onClick={() => changePage(page - 1)}>Previous page</Button>
        <label className="pdf-page-choice">Page <input type="number" min={1} max={pdf.numPages} value={page} disabled={busy}
          onChange={(event) => changePage(event.target.valueAsNumber)} /> of {pdf.numPages}</label>
        <Button disabled={busy || page >= pdf.numPages} onClick={() => changePage(page + 1)}>Next page</Button>
      </div>
      <PdfPreview key={page} pdf={pdf} page={page} box={box} disabled={busy} signer={signer} onBox={setBox} />
    </Card>}
    <PdfSigner documentId={source.id} page={page} box={box} ready={pdf !== null && error === null} onSigner={setSigner}
      onBusy={(next) => { setBusy(next); onBusy(next); }} />
    </div>
  </>;
}
