/**
 * What to tell the user after an attempt that leaves the request open.
 */
import type { ApproveResult } from "../api/types";
import type { Tone } from "../components/ui";

/** A message under the form. */
export interface Message {
  readonly tone: Tone;
  readonly text: string;
}

/** Turns a result that keeps the request open into a message. */
export function messageFor(result: ApproveResult): Message | null {
  switch (result.status) {
    case "pin-incorrect":
      if (result.locked) {
        return { tone: "danger", text: "Wrong PIN. The token PIN is now locked; unlock it with your token vendor's tool." };
      }
      return { tone: "danger", text: result.finalTry ? "Wrong PIN. One more wrong PIN will lock this token." : "Wrong PIN. The token counts wrong entries, so check it carefully." };
    case "pin-locked":
      return { tone: "danger", text: "This token's PIN is locked. Unlock it with your token vendor's tool, or choose another certificate." };
    case "pin-invalid":
      return { tone: "danger", text: "That PIN has the wrong length or characters." };
    case "pin-required":
      return { tone: "warn", text: "Enter the token PIN." };
    case "failed":
      return { tone: "danger", text: result.message };
    case "expired":
      return { tone: "neutral", text: "This request has ended." };
    case "signed":
      return null;
  }
}
