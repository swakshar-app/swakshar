/**
 * The token PIN field. The value lives here only until Sign is pressed; the
 * field remounts for each attempt, so focus returns to it after a wrong PIN.
 */
import { type KeyboardEvent, type ReactElement, useState } from "react";

/** PIN input with a show toggle, a Caps Lock hint and a last-try warning. */
export function PinField(props: {
  readonly value: string;
  readonly onChange: (value: string) => void;
  readonly finalTry: boolean;
}): ReactElement {
  const { value, onChange, finalTry } = props;
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
      {finalTry ? <small className="warn-text">One more wrong PIN will lock this token.</small> : null}
    </div>
  );
}
