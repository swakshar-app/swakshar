/**
 * The token PIN field. The value lives here only until Sign is pressed.
 */
import { type KeyboardEvent, type ReactElement, useState } from "react";

/** PIN input with a show toggle, a Caps Lock hint and lockout warnings. */
export function PinField(props: {
  readonly value: string;
  readonly onChange: (value: string) => void;
  readonly warning: string | null;
}): ReactElement {
  const { value, onChange, warning } = props;
  const [visible, setVisible] = useState(false);
  const [capsLock, setCapsLock] = useState(false);
  const track = (event: KeyboardEvent<HTMLInputElement>): void => setCapsLock(event.getModifierState("CapsLock"));
  return (
    <div className="field">
      <label htmlFor="pin">Token PIN</label>
      <div className="pin">
        <input
          id="pin"
          type={visible ? "text" : "password"}
          value={value}
          onChange={(event) => onChange(event.target.value)}
          onKeyUp={track}
          onKeyDown={track}
          autoComplete="off"
          autoCorrect="off"
          autoCapitalize="off"
          spellCheck={false}
          autoFocus
        />
        <button type="button" className="button button-quiet" onClick={() => setVisible(!visible)}>
          {visible ? "Hide" : "Show"}
        </button>
      </div>
      {capsLock ? <small className="warn-text">Caps Lock is on.</small> : null}
      {warning === null ? null : <small className="warn-text">{warning}</small>}
    </div>
  );
}
