/**
 * Home's headline for the signer's state: ready, off, waiting, or what to fix.
 */
import type { Overview } from "../api/types";
import type { Tone } from "../components/ui";

/** Headline, explanation and tone for the current state. */
export function statusText(overview: Overview): { readonly title: string; readonly detail: string; readonly tone: Tone } {
  const { server, trust, waiting } = overview;
  if (waiting) {
    return { title: "Waiting for your approval", detail: "Answer the request in the signature window.", tone: "warn" };
  }
  switch (server.state) {
    case "failed":
      return { title: "Signing could not start", detail: server.error ?? "The signer could not start.", tone: "danger" };
    case "paused":
      return { title: "Signing is off", detail: "Turn it on when you are ready to sign on the GST portal. Swakshar remembers your choice.", tone: "neutral" };
    case "starting":
      return { title: "Starting", detail: "Preparing the local certificate and a port.", tone: "neutral" };
    case "running":
      return trust.status === "trusted"
        ? { title: "Ready for the GST portal", detail: `Listening on 127.0.0.1:${String(server.port ?? "")}. Sign in to the portal and use your DSC as usual.`, tone: "success" }
        : { title: "Finish setup", detail: "Your browser must trust Swakshar's local certificate before the portal can connect.", tone: "warn" };
  }
}
