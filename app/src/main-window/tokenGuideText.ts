/**
 * The token guide's words: a name for a plugged-in token, and what to do
 * next for each driver state.
 */
import type { AttachedView } from "../api/types";

/** A readable name: maker and product, whichever the token reports. */
export function tokenName(token: AttachedView): string {
  const { maker, product } = token;
  if (maker !== null && product !== null) {
    return product.toLowerCase().includes(maker.toLowerCase()) ? product : `${maker} ${product}`;
  }
  return product ?? maker ?? "A DSC token";
}

/** What to do next for a token that is not ready. */
export function advice(token: AttachedView): string {
  const driver = token.family === null ? "the driver that came with it" : `the ${token.family} driver`;
  switch (token.state) {
    case "missing":
      return `Install ${driver} for macOS. Your Certifying Authority or the shop that sold the token provides it. Swakshar picks it up within a few seconds; there is nothing else to set.`;
    case "other-architecture":
      return `The installed driver is built for a different processor than this Mac's. Get the right version of ${driver} from your token's supplier.`;
    case "failed":
      return `The driver is installed but did not start (${token.detail ?? "no reason given"}). Unplug the token and plug it back in.`;
    case "not-seen":
      return `A driver is installed but does not see this token yet. Unplug it and plug it back in. If it still does not appear, install ${driver}.`;
    case "ready":
      return "";
  }
}
