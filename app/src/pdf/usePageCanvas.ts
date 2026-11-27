/** Bounded, cancellable canvas rendering for the currently visible PDF page. */
import type { PDFDocumentProxy, PageViewport } from "pdfjs-dist";
import { type RefObject, useEffect, useState } from "react";

import { errorText } from "../api/commands";

/** Keep high-density previews sharp without allocating unbounded canvas memory. */
const MAX_PIXELS = 2_000_000;
const MAX_WIDTH = 850;

/** A completion is usable only for the document, page and size that produced it. */
interface Rendered {
  readonly pdf: PDFDocumentProxy; readonly page: number; readonly width: number;
  readonly viewport: PageViewport | null; readonly error: string | null;
}

/** Fits the canvas to its container and ignores obsolete page or resize work. */
export function usePageCanvas(pdf: PDFDocumentProxy, page: number, canvas: RefObject<HTMLCanvasElement | null>, container: RefObject<HTMLDivElement | null>): {
  readonly viewport: PageViewport | null;
  readonly error: string | null;
  readonly rendering: boolean;
} {
  const [width, setWidth] = useState(650);
  const [rendered, setRendered] = useState<Rendered | null>(null);
  useEffect(() => {
    const element = container.current;
    if (element === null) { return undefined; }
    const observer = new ResizeObserver(([entry]) => {
      if (entry !== undefined && entry.contentRect.width > 0) {
        setWidth(Math.min(MAX_WIDTH, Math.floor(entry.contentRect.width)));
      }
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, [container]);
  useEffect(() => {
    let active = true;
    let cancel: (() => void) | undefined;
    void pdf.getPage(page).then(async (pdfPage) => {
      const element = canvas.current;
      if (!active || element === null) { return; }
      const original = pdfPage.getViewport({ scale: 1 });
      const view = pdfPage.getViewport({ scale: Math.min(1.5, width / original.width) });
      const density = Math.min(window.devicePixelRatio || 1, 2, Math.sqrt(MAX_PIXELS / (view.width * view.height)));
      element.width = Math.ceil(view.width * density);
      element.height = Math.ceil(view.height * density);
      element.style.width = `${String(view.width)}px`;
      element.style.height = `${String(view.height)}px`;
      const render = pdfPage.render({ canvas: element, viewport: view, transform: [density, 0, 0, density, 0, 0] });
      cancel = (): void => render.cancel();
      await render.promise;
      if (active) { setRendered({ pdf, page, width, viewport: view, error: null }); }
    }).catch((reason: unknown) => {
      if (active) { setRendered({ pdf, page, width, viewport: null, error: errorText(reason) }); }
    });
    return () => { active = false; cancel?.(); };
  }, [pdf, page, width, canvas]);
  const current = rendered?.pdf === pdf && rendered.page === page && rendered.width === width ? rendered : null;
  return { viewport: current?.viewport ?? null, error: current?.error ?? null, rendering: current === null };
}
