/**
 * Which system the app runs on, for choices that exist on one system only.
 */

/** Marker the macOS web view puts in its user agent. */
const MAC_MARKER = "Macintosh";

/** True inside the macOS app, where the Dock exists. */
export function onMac(userAgent: string = navigator.userAgent): boolean {
  return userAgent.includes(MAC_MARKER);
}
