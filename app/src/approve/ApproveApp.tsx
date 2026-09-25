/**
 * The approval window: what is asked, which certificate, the PIN, and Sign.
 * Nothing is signed unless the user presses Sign here.
 */
import { type FormEvent, type ReactElement, useCallback, useEffect, useState } from "react";

import logo from "../../src-tauri/icons/128x128.png";
import { approveApi, errorText } from "../api/commands";
import type { ApproveResult } from "../api/types";
import { formatCountdown } from "../components/format";
import { Icon } from "../components/Icon";
import { Button, Notice, type Tone } from "../components/ui";
import { CertificateChoice } from "./CertificateChoice";
import { PinField } from "./PinField";
import { RequestSummary } from "./RequestSummary";
import { useClock, useRequest } from "./useRequest";

/** A message under the form. */
interface Message {
  readonly tone: Tone;
  readonly text: string;
}

/** Turns a result that keeps the request open into a message. */
function messageFor(result: ApproveResult): Message | null {
  switch (result.status) {
    case "pin-incorrect":
      if (result.locked) {
        return { tone: "danger", text: "Wrong PIN. The token PIN is now locked; unlock it with your token vendor's tool." };
      }
      return { tone: "danger", text: result.finalTry ? "Wrong PIN. One more wrong PIN will lock this token." : "Wrong PIN. Wrong entries are counted by the token." };
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

/** The approval window. */
export function ApproveApp(): ReactElement {
  const [selected, setSelected] = useState(0);
  const [pin, setPin] = useState("");
  const [busy, setBusy] = useState(false);
  const [signedAs, setSignedAs] = useState<string | null>(null);
  const [message, setMessage] = useState<Message | null>(null);
  const reset = useCallback(() => {
    setSelected(0);
    setPin("");
    setBusy(false);
    setSignedAs(null);
    setMessage(null);
  }, []);
  const { pending, clear } = useRequest(reset);
  const now = useClock(pending !== null && signedAs === null);
  const cancel = useCallback(() => {
    if (pending !== null) {
      void approveApi.cancel(pending.id);
      clear();
    }
  }, [pending, clear]);
  useEffect(() => {
    const onKey = (event: globalThis.KeyboardEvent): void => {
      if (event.key === "Escape") {
        cancel();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [cancel]);
  if (signedAs !== null) {
    return (
      <div className="approve approve-center">
        <div className="done-mark"><Icon name="check" size={28} /></div>
        <h1>Signed</h1>
        <p className="muted">Signed as {signedAs}. Continue on the GST portal.</p>
      </div>
    );
  }
  if (pending === null) {
    return (
      <div className="approve approve-center">
        <img src={logo} alt="" width={56} height={56} />
        <h1>No request waiting</h1>
        <p className="muted">When the GST portal asks for your DSC, the request appears here.</p>
      </div>
    );
  }
  const candidate = pending.candidates[selected];
  const submit = (event: FormEvent): void => {
    event.preventDefault();
    if (candidate === undefined || busy) {
      return;
    }
    const secret = candidate.pinPad ? null : pin;
    setPin("");
    setBusy(true);
    setMessage(null);
    approveApi.approve(pending.id, candidate.index, secret).then(
      (result) => {
        setBusy(false);
        if (result.status === "signed") {
          setSignedAs(result.holder);
        } else if (result.status === "expired") {
          clear();
        } else {
          setMessage(messageFor(result));
        }
      },
      (reason: unknown) => {
        setBusy(false);
        setMessage({ tone: "danger", text: errorText(reason) });
      },
    );
  };
  const warning = candidate?.pinFinalTry === true ? "One more wrong PIN will lock this token." : null;
  return (
    <form className="approve" onSubmit={submit}>
      <header className="approve-header">
        <img src={logo} alt="" width={24} height={24} />
        <span>Signature request</span>
        <span className="countdown muted">Expires in {formatCountdown(pending.expiresAt - now)}</span>
      </header>
      <h1>The GST portal wants your signature</h1>
      <RequestSummary pending={pending} />
      {pending.candidates.length === 0 ? (
        <Notice tone="danger">No suitable certificate is on the connected tokens. Plug in the right token, cancel, and try again on the portal.</Notice>
      ) : (
        <CertificateChoice candidates={pending.candidates} selected={selected} onSelect={setSelected} />
      )}
      {candidate === undefined || candidate.pinPad ? null : <PinField value={pin} onChange={setPin} warning={warning} />}
      {candidate?.pinPad === true ? <Notice tone="neutral">Enter the PIN on your token's keypad after you press Sign.</Notice> : null}
      {message === null ? null : <Notice tone={message.tone}>{message.text}</Notice>}
      <div className="approve-actions">
        <Button onClick={cancel} disabled={busy}>Cancel</Button>
        <Button variant="primary" type="submit" disabled={busy || candidate === undefined || candidate.pinLocked}>
          {busy ? "Signing on your token" : "Sign"}
        </Button>
      </div>
    </form>
  );
}
