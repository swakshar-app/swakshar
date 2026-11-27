/** Local page canvas with a draggable signature rectangle. */
import type { PDFDocumentProxy } from "pdfjs-dist";
import { type PointerEvent, type ReactElement, useRef } from "react";

import type { PdfBox } from "../api/signing";
import { Notice } from "../components/ui";
import { boxFromPoints } from "./placement";
import { usePageCanvas } from "./usePageCanvas";
import { SignatureStamp } from "./SignatureStamp";

/** Renders one page and maps pointer coordinates through its rotation and crop box. */
export function PdfPreview(props: {
  readonly pdf: PDFDocumentProxy;
  readonly page: number;
  readonly box: PdfBox | null;
  readonly disabled: boolean;
  readonly signer: string;
  readonly onBox: (box: PdfBox | null) => void;
}): ReactElement {
  const { pdf, page, box, disabled, signer, onBox } = props;
  const canvas = useRef<HTMLCanvasElement>(null);
  const container = useRef<HTMLDivElement>(null);
  const start = useRef<readonly number[] | null>(null);
  const { viewport, error, rendering } = usePageCanvas(pdf, page, canvas, container);

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

  const overlay = viewport === null || box === null ? null : [
    ...viewport.convertToViewportPoint(box[0], box[1]),
    ...viewport.convertToViewportPoint(box[2], box[3]),
  ];
  const [x1 = 0, y1 = 0, x2 = 0, y2 = 0] = overlay ?? [];
  return <>
    {error === null ? null : <Notice tone="danger">{error}</Notice>}
    {rendering ? <p className="muted" role="status">Rendering page {page}.</p> : null}
    <div className="pdf-preview-scroll" ref={container} aria-busy={rendering}>
      <div className="pdf-preview" onPointerDown={begin} onPointerMove={move} onPointerUp={finish}
        onPointerCancel={() => { start.current = null; onBox(null); }}>
        <canvas ref={canvas} aria-label={`PDF page ${String(page)}`} />
        {overlay === null || viewport === null ? null : <div className="pdf-signature-box" style={{
          left: `${String(Math.min(x1, x2) / viewport.width * 100)}%`,
          top: `${String(Math.min(y1, y2) / viewport.height * 100)}%`,
          width: `${String(Math.abs(x2 - x1) / viewport.width * 100)}%`,
          height: `${String(Math.abs(y2 - y1) / viewport.height * 100)}%`,
        }}><SignatureStamp width={box === null ? 0 : box[2] - box[0]} height={box === null ? 0 : box[3] - box[1]} signer={signer} /></div>}
      </div>
    </div>
    <p className="muted pdf-hint">Drag on the page to place your signature. Draw again to adjust its size or position.</p>
  </>;
}
