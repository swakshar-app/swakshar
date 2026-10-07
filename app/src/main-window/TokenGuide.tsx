/**
 * Guidance for tokens plugged in over USB that no driver reaches yet: which
 * driver to install, with the maker's download page when it is known, or
 * what went wrong with the one that is installed.
 */
import type { ReactElement } from "react";

import { mainApi } from "../api/commands";
import type { InventoryView } from "../api/types";
import { useAction } from "../components/hooks";
import { Icon } from "../components/Icon";
import { Button, Notice } from "../components/ui";
import { advice, tokenName } from "./tokenGuideText";

/** One guide card per token that needs something, or a prompt to plug one in. */
export function TokenGuide(props: { readonly tokens: InventoryView | null; readonly onChange: () => void }): ReactElement | null {
  const { tokens, onChange } = props;
  const action = useAction();
  if (tokens === null) {
    return null;
  }
  const waiting = tokens.attached.filter((token) => token.state !== "ready");
  if (waiting.length === 0) {
    return tokens.tokens.length > 0 || tokens.attached.length > 0 ? null : (
      <div className="guide guide-idle">
        <Icon name="token" size={20} />
        <div className="guide-body">
          <p className="guide-title">Plug in your DSC token</p>
          <p className="muted">Swakshar recognises it as soon as it is connected, and says here if it needs a driver.</p>
        </div>
      </div>
    );
  }
  const choose = (): void => {
    void action.run(async () => {
      await mainApi.addDriver();
      onChange();
    });
  };
  const download = (vendorId: number): void => {
    void action.run(() => mainApi.openDriverPage(vendorId));
  };
  return (
    <div className="guides">
      {waiting.map((token, index) => (
        <div key={`${tokenName(token)}-${String(index)}`} className="guide">
          <Icon name="token" size={20} />
          <div className="guide-body">
            <p className="guide-title">{tokenName(token)} is plugged in</p>
            <p>{advice(token)}</p>
            <div className="row">
              {token.state === "missing" && token.driverPage !== null ? (
                <Button icon="external" onClick={() => download(token.vendorId)} disabled={action.busy}>Get the driver</Button>
              ) : null}
              <Button icon="refresh" onClick={onChange} disabled={action.busy}>Check again</Button>
              <Button variant="quiet" icon="plus" onClick={choose} disabled={action.busy}>Driver installed elsewhere? Choose the file</Button>
            </div>
          </div>
        </div>
      ))}
      {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
    </div>
  );
}
