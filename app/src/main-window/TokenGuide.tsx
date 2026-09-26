/**
 * Guidance for tokens plugged in over USB that no driver reaches yet: which
 * driver to install, or what went wrong with the one that is installed.
 */
import type { ReactElement } from "react";

import { mainApi } from "../api/commands";
import type { AttachedView, InventoryView } from "../api/types";
import { useAction } from "../components/hooks";
import { Icon } from "../components/Icon";
import { Button, Notice } from "../components/ui";

/** A readable name: maker and product, whichever the token reports. */
function tokenName(token: AttachedView): string {
  const { maker, product } = token;
  if (maker !== null && product !== null) {
    return product.toLowerCase().includes(maker.toLowerCase()) ? product : `${maker} ${product}`;
  }
  return product ?? maker ?? "A DSC token";
}

/** What to do next for a token that is not ready. */
function advice(token: AttachedView): string {
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
  return (
    <div className="guides">
      {waiting.map((token, index) => (
        <div key={`${tokenName(token)}-${String(index)}`} className="guide">
          <Icon name="token" size={20} />
          <div className="guide-body">
            <p className="guide-title">{tokenName(token)} is plugged in</p>
            <p>{advice(token)}</p>
            <div className="row">
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
