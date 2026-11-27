/** Bounded, cancellable canvas rendering for the currently visible PDF page. */
import type { PDFDocumentProxy, PageViewport } from "pdfjs-dist";
import { type RefObject, useEffect, useState } from "react";

import { errorText } from "../api/commands";

/** Keep high-density previews sharp without allocating unbounded canvas memory. */
const MAX_PIXELS = 2_000_000;
const MAX_WIDTH = 850;

/** Fits the canvas to its container and ignores obsolete page or resize work. */
export function usePageCanvas(pdf: PDFDocumentProxy, page: number, canvas: RefObject<HTMLCanvasElement | null>, container: RefObject<HTMLDivElement | null>): {
  readonly viewport: PageViewport | null;
  readonly error: string | null;
  readonly rendering: boolean;
} {
  const [width, setWidth] = useState(650);
  const [viewport, setViewport] = useState<PageViewport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [rendering, setRendering] = useState(true);
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
    setViewport(null); setError(null); setRendering(true);
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
      if (active) { setViewport(view); setRendering(false); }
    }).catch((reason: unknown) => {
      if (active) { setError(errorText(reason)); setRendering(false); }
    });
    return () => { active = false; cancel?.(); };
  }, [pdf, page, width, canvas]);
  return { viewport, error, rendering };
}
