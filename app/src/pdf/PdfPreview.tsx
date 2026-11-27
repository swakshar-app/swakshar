/** Local page canvas with a draggable signature rectangle. */
import type { PDFDocumentProxy, PageViewport } from "pdfjs-dist";
import { type PointerEvent, type ReactElement, useEffect, useRef, useState } from "react";

import { errorText } from "../api/commands";
import type { PdfBox } from "../api/signing";
import { Notice } from "../components/ui";
import { boxFromPoints } from "./placement";

/** Limit preview width while preserving real PDF-space coordinates. */
const MAX_PREVIEW_WIDTH = 650;

/** Renders one page and maps pointer coordinates through its rotation and crop box. */
export function PdfPreview(props: {
  readonly pdf: PDFDocumentProxy;
  readonly page: number;
  readonly box: PdfBox | null;
  readonly disabled: boolean;
  readonly onBox: (box: PdfBox | null) => void;
}): ReactElement {
  const { pdf, page, box, disabled, onBox } = props;
  const canvas = useRef<HTMLCanvasElement>(null);
  const start = useRef<readonly number[] | null>(null);
  const [viewport, setViewport] = useState<PageViewport | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    let cancel: (() => void) | null = null;
    setViewport(null);
    setError(null);
    void pdf.getPage(page).then(async (pdfPage) => {
      const element = canvas.current;
      if (!active || element === null) { return; }
      const original = pdfPage.getViewport({ scale: 1 });
      const view = pdfPage.getViewport({ scale: Math.min(1, MAX_PREVIEW_WIDTH / original.width) });
      element.width = Math.ceil(view.width);
      element.height = Math.ceil(view.height);
      const render = pdfPage.render({ canvas: element, viewport: view });
      cancel = (): void => render.cancel();
      await render.promise;
      if (active) { setViewport(view); }
    }).catch((reason: unknown) => { if (active) { setError(errorText(reason)); } });
    return () => { active = false; cancel?.(); };
  }, [pdf, page]);

  /** Converts a captured pointer to PDF space, clamped to the displayed page. */
  function point(event: PointerEvent<HTMLDivElement>): readonly number[] | null {
    if (viewport === null) { return null; }
    const bounds = event.currentTarget.getBoundingClientRect();
    const x = Math.max(0, Math.min(viewport.width, (event.clientX - bounds.left) * viewport.width / bounds.width));
    const y = Math.max(0, Math.min(viewport.height, (event.clientY - bounds.top) * viewport.height / bounds.height));
    return viewport.convertToPdfPoint(x, y);
  }

  /** Begins a new rectangle only when the current page has finished rendering. */
  function begin(event: PointerEvent<HTMLDivElement>): void {
    if (disabled || viewport === null || event.button !== 0) { return; }
    start.current = point(event);
    event.currentTarget.setPointerCapture(event.pointerId);
    onBox(null);
  }

  /** Updates the PDF-space rectangle regardless of drag direction. */
  function move(event: PointerEvent<HTMLDivElement>): void {
    const end = point(event);
    if (!disabled && start.current !== null && end !== null) { onBox(boxFromPoints(start.current, end)); }
  }

  /** Finishes placement; subsequent pointer motion cannot change the rectangle. */
  function finish(event: PointerEvent<HTMLDivElement>): void {
    move(event);
    start.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) { event.currentTarget.releasePointerCapture(event.pointerId); }
  }

  const overlay = viewport === null || box === null ? null : viewport.convertToViewportRectangle([...box]);
  const [x1 = 0, y1 = 0, x2 = 0, y2 = 0] = overlay ?? [];
  return <>
    {error === null ? null : <Notice tone="danger">{error}</Notice>}
    <div className="pdf-preview-scroll">
      <div className="pdf-preview" onPointerDown={begin} onPointerMove={move} onPointerUp={finish}
        onPointerCancel={() => { start.current = null; onBox(null); }}>
        <canvas ref={canvas} aria-label={`PDF page ${String(page)}`} />
        {overlay === null || viewport === null ? null : <div className="pdf-signature-box" style={{
          left: `${String(Math.min(x1, x2) / viewport.width * 100)}%`,
          top: `${String(Math.min(y1, y2) / viewport.height * 100)}%`,
          width: `${String(Math.abs(x2 - x1) / viewport.width * 100)}%`,
          height: `${String(Math.abs(y2 - y1) / viewport.height * 100)}%`,
        }}>Signature</div>}
      </div>
    </div>
    <p className="muted">Drag on the page to place and resize the signature. Drawing another box replaces this placement.</p>
  </>;
}
