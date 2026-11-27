/** Loads a binary snapshot into the local PDF worker without document scripts. */
import { getDocument, GlobalWorkerOptions, type PDFDocumentLoadingTask, type PDFDocumentProxy } from "pdfjs-dist/legacy/build/pdf.mjs";
import { useEffect, useState } from "react";

import { errorText } from "../api/commands";
import { signingApi, type PdfDocument } from "../api/signing";

GlobalWorkerOptions.workerSrc = new URL("pdfjs-dist/legacy/build/pdf.worker.min.mjs", import.meta.url).href;

/** A parser result belongs only to the snapshot that produced it. */
interface Loaded {
  readonly id: number; readonly pdf: PDFDocumentProxy | null; readonly error: string | null; readonly stage: string;
}

/** Destroys the previous parser and worker when the user chooses another PDF. */
export function usePdf(source: PdfDocument): { readonly pdf: PDFDocumentProxy | null; readonly error: string | null; readonly stage: string } {
  const [loaded, setLoaded] = useState<Loaded | null>(null);
  useEffect(() => {
    let active = true;
    let passwordProtected = false;
    let task: PDFDocumentLoadingTask | null = null;
    /** Shares exactly the backend's immutable selection with the PDF worker. */
    async function load(): Promise<void> {
      const bytes = await signingApi.pdfBytes(source.id);
      if (!active) { return; }
      setLoaded({ id: source.id, pdf: null, error: null, stage: "Preparing PDF pages" });
      task = getDocument({
        data: new Uint8Array(bytes), enableXfa: false,
        cMapUrl: "/pdfjs/cmaps/", cMapPacked: true, standardFontDataUrl: "/pdfjs/standard_fonts/",
        iccUrl: "/pdfjs/iccs/", wasmUrl: "/pdfjs/wasm/",
      });
      task.onPassword = (): void => {
        passwordProtected = true;
        if (active) { setLoaded({ id: source.id, pdf: null, error: "Password-protected PDFs cannot be signed here. Choose an unlocked copy.", stage: "" }); }
        void task?.destroy();
      };
      const document = await task.promise;
      if (active) { setLoaded({ id: source.id, pdf: document, error: null, stage: "" }); }
    }
    void load().catch((reason: unknown) => {
      if (active && !passwordProtected) { setLoaded({ id: source.id, pdf: null, error: errorText(reason), stage: "" }); }
    });
    return () => { active = false; void task?.destroy(); };
  }, [source.id]);
  return loaded?.id === source.id ? loaded : { pdf: null, error: null, stage: "Reading PDF" };
}
