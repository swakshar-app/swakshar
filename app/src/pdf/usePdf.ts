/** Loads a binary snapshot into the local PDF worker without document scripts. */
import { getDocument, GlobalWorkerOptions, type PDFDocumentLoadingTask, type PDFDocumentProxy } from "pdfjs-dist/legacy/build/pdf.mjs";
import { useEffect, useState } from "react";

import { errorText } from "../api/commands";
import { signingApi, type PdfDocument } from "../api/signing";

GlobalWorkerOptions.workerSrc = new URL("pdfjs-dist/legacy/build/pdf.worker.min.mjs", import.meta.url).href;

/** Destroys the previous parser and worker when the user chooses another PDF. */
export function usePdf(source: PdfDocument): { readonly pdf: PDFDocumentProxy | null; readonly error: string | null; readonly stage: string } {
  const [pdf, setPdf] = useState<PDFDocumentProxy | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [stage, setStage] = useState("Reading PDF");
  useEffect(() => {
    let active = true;
    let passwordProtected = false;
    let task: PDFDocumentLoadingTask | null = null;
    setPdf(null); setError(null); setStage("Reading PDF");
    /** Shares exactly the backend's immutable selection with the PDF worker. */
    async function load(): Promise<void> {
      const bytes = await signingApi.pdfBytes(source.id);
      if (!active) { return; }
      setStage("Preparing PDF pages");
      task = getDocument({
        data: new Uint8Array(bytes), enableXfa: false,
        cMapUrl: "/pdfjs/cmaps/", cMapPacked: true, standardFontDataUrl: "/pdfjs/standard_fonts/",
        iccUrl: "/pdfjs/iccs/", wasmUrl: "/pdfjs/wasm/",
      });
      task.onPassword = (): void => {
        passwordProtected = true;
        if (active) { setError("Password-protected PDFs cannot be signed here. Choose an unlocked copy."); }
        void task?.destroy();
      };
      const document = await task.promise;
      if (active) { setPdf(document); }
    }
    void load().catch((reason: unknown) => {
      if (active && !passwordProtected) { setError(errorText(reason)); }
    });
    return () => { active = false; void task?.destroy(); };
  }, [source.id]);
  return { pdf, error, stage };
}
