/** Vector preview of the approved CLI signature appearance, with shared metrics. */
import { type ReactElement, useState } from "react";

import { stampDate, stampLayout, stampStyle } from "./stampLayout";

/** Shows the signer and timestamp at the exact PDF-space placement dimensions. */
export function SignatureStamp({ width, height, signer }: {
  readonly width: number; readonly height: number; readonly signer: string;
}): ReactElement | null {
  const [date] = useState(() => stampDate(new Date()));
  if (width <= 0 || height <= 0) { return null; }
  const { scale, lines, top } = stampLayout(width, height, signer, date);
  const tick = stampStyle.tick.map(([x = 0, y = 0]) => `${String(width * (0.4 + x * 0.3))},${String(height * (0.9 - y * 0.72))}`).join(" ");
  return <svg viewBox={`0 0 ${String(width)} ${String(height)}`} role="img" aria-label="Signature appearance preview" preserveAspectRatio="none">
    <polygon points={tick} fill="#1a8c26" stroke="black" strokeWidth={1.4} strokeLinejoin="round" />
    <g fill="black" fontFamily="Helvetica, Arial, sans-serif">
      <text x={stampStyle.padding * scale} y={top} fontSize={stampStyle.headSize * scale}>{stampStyle.heading}</text>
      {lines.map((line, index) => <text key={String(index)} x={stampStyle.padding * scale}
        y={top + (stampStyle.headSize * 0.35 + (index + 1) * stampStyle.leading) * scale}
        fontSize={stampStyle.bodySize * scale}>{line}</text>)}
    </g>
  </svg>;
}
