/**
 * The approval window's heading: who is asking for the signature.
 */
import { hostOf } from "../components/format";

/** The GST portal's registrable domain. */
const GST_DOMAIN = "gst.gov.in";
/** Suffix every GST portal subdomain ends with. */
const GST_SUFFIX = ".gst.gov.in";

/** "The GST portal wants your signature" for GST hosts, else the host's own name. */
export function heading(origin: string): string {
  const host = hostOf(origin);
  return host === GST_DOMAIN || host.endsWith(GST_SUFFIX) ? "The GST portal wants your signature" : `${host} wants your signature`;
}
