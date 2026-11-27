/** Loads local PDF bytes without allowing PDF scripts or network resources. */
import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from "pdfjs-dist/legacy/build/pdf.mjs";
import { useEffect, useState } from "react";

import { errorText } from "../api/commands";
import type { PdfDocument } from "../api/signing";

GlobalWorkerOptions.workerSrc = new URL("pdfjs-dist/legacy/build/pdf.worker.min.mjs", import.meta.url).href;

/** Destroys the previous parser and worker when the user chooses another PDF. */
export function usePdf(source: PdfDocument): { readonly pdf: PDFDocumentProxy | null; readonly error: string | null } {
  const [pdf, setPdf] = useState<PDFDocumentProxy | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    let passwordProtected = false;
    const task = getDocument({
      data: Uint8Array.from(source.bytes),
      cMapUrl: "/pdfjs/cmaps/", cMapPacked: true, standardFontDataUrl: "/pdfjs/standard_fonts/",
      iccUrl: "/pdfjs/iccs/",
      wasmUrl: "/pdfjs/wasm/",
    });
    task.onPassword = (): void => {
      passwordProtected = true;
      if (active) { setError("Password-protected PDFs cannot be signed here. Choose an unlocked copy."); }
      void task.destroy();
    };
    void task.promise.then((document) => { if (active) { setPdf(document); } }, (reason: unknown) => {
      if (active && !passwordProtected) { setError(errorText(reason)); }
    });
    return () => { active = false; void task.destroy(); };
  }, [source]);
  return { pdf, error };
}
