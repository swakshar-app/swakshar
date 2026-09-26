/**
 * The approval window: who asks, what is signed, which certificate, the PIN.
 * Nothing is signed unless the user presses Sign here.
 */
import { type FormEvent, type ReactElement, useCallback, useEffect, useState } from "react";

import logo from "../../src-tauri/icons/128x128.png";
import { approveApi, errorText } from "../api/commands";
import { formatCountdown, hostOf } from "../components/format";
import { Icon } from "../components/Icon";
import { Button, Notice } from "../components/ui";
import { CertificateChoice } from "./CertificateChoice";
import { type Message, messageFor } from "./messages";
import { PinField } from "./PinField";
import { RequestSummary } from "./RequestSummary";
import { useClock, useRequest } from "./useRequest";

/** Suffix every GST portal host ends with. */
const GST_SUFFIX = ".gst.gov.in";

/** Heading naming who asks: the GST portal, or the site's own host. */
function heading(origin: string): string {
  const host = hostOf(origin);
  return host === "gst.gov.in" || host.endsWith(GST_SUFFIX) ? "The GST portal wants your signature" : `${host} wants your signature`;
}

/** Shown after a successful signature, until the window hides. */
function Signed({ holder }: { readonly holder: string }): ReactElement {
  return (
    <div className="approve approve-center">
      <div className="done-mark"><Icon name="check" size={28} /></div>
      <h1>Signed</h1>
      <p className="muted">Signed as {holder}. Continue on the GST portal.</p>
    </div>
  );
}

/** Shown when no request is waiting. */
function Idle(): ReactElement {
  return (
    <div className="approve approve-center">
      <img src={logo} alt="" width={56} height={56} />
      <h1>No request waiting</h1>
      <p className="muted">When the GST portal asks for your DSC, the request appears here.</p>
    </div>
  );
}

/** The approval window. */
export function ApproveApp(): ReactElement {
  const [selected, setSelected] = useState(0);
  const [pin, setPin] = useState("");
  const [attempt, setAttempt] = useState(0);
  const [busy, setBusy] = useState(false);
  const [checking, setChecking] = useState(false);
  const [signedAs, setSignedAs] = useState<string | null>(null);
  const [message, setMessage] = useState<Message | null>(null);
  const reset = useCallback(() => {
    setSelected(0);
    setPin("");
    setBusy(false);
    setSignedAs(null);
    setMessage(null);
  }, []);
  const { pending, clear, replace } = useRequest(reset);
  const now = useClock(pending !== null && signedAs === null);
  const cancel = useCallback(() => {
    if (pending !== null && !busy) {
      void approveApi.cancel(pending.id);
      clear();
    }
  }, [pending, busy, clear]);
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
    return <Signed holder={signedAs} />;
  }
  if (pending === null) {
    return <Idle />;
  }
  const candidate = pending.candidates[selected];
  const refresh = (): void => {
    setMessage(null);
    setChecking(true);
    approveApi.refresh(pending.id).then(
      (view) => {
        setChecking(false);
        setSelected(0);
        replace(view);
      },
      (reason: unknown) => {
        setChecking(false);
        setMessage({ tone: "danger", text: errorText(reason) });
      },
    );
  };
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
        setAttempt((value) => value + 1);
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
  return (
    <form className="approve" onSubmit={submit}>
      <div className="approve-body">
        <header className="approve-header">
          <img src={logo} alt="" width={22} height={22} />
          <span>Signature request</span>
          <span className="countdown muted">Expires in {formatCountdown(pending.expiresAt - now)}</span>
        </header>
        <h1>{heading(pending.origin)}</h1>
        <RequestSummary pending={pending} />
        {pending.candidates.length === 0 ? (
          <Notice tone="danger">No suitable certificate is on the connected tokens. Plug in the token for this PAN, then check again.</Notice>
        ) : (
          <CertificateChoice candidates={pending.candidates} selected={selected} onSelect={setSelected} />
        )}
        <div className="row">
          <Button variant="quiet" icon="refresh" onClick={refresh} disabled={busy || checking}>
            {checking ? "Checking tokens" : "Check tokens again"}
          </Button>
        </div>
        {candidate === undefined || candidate.pinPad ? null : (
          <PinField key={`${String(pending.id)}-${String(attempt)}`} value={pin} onChange={setPin} finalTry={candidate.pinFinalTry} />
        )}
        {candidate?.pinPad === true ? <Notice tone="neutral">Enter the PIN on your token's keypad after you press Sign.</Notice> : null}
        {message === null ? null : <Notice tone={message.tone}>{message.text}</Notice>}
      </div>
      <footer className="approve-footer">
        <Button onClick={cancel} disabled={busy}>Cancel</Button>
        <Button variant="primary" type="submit" disabled={busy || checking || candidate === undefined || candidate.pinLocked}>
          {busy ? "Signing on your token" : "Sign"}
        </Button>
      </footer>
    </form>
  );
}
