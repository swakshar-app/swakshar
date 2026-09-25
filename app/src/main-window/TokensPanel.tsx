/**
 * Connected tokens and their certificates, read without a PIN.
 */
import type { ReactElement } from "react";

import type { CertView, InventoryView, ModuleView, TokenView } from "../api/types";
import { Icon } from "../components/Icon";
import { Card, Notice, Tag } from "../components/ui";

/** Tags describing a certificate's state. */
function CertTags({ cert }: { readonly cert: CertView }): ReactElement {
  return (
    <div className="tags">
      <Tag tone="neutral">{cert.classLabel}</Tag>
      {cert.valid ? null : <Tag tone="danger">Expired or not yet valid</Tag>}
      {cert.expiresSoon ? <Tag tone="warn">Expires soon</Tag> : null}
      {cert.signing ? null : <Tag tone="neutral">Not for signing</Tag>}
    </div>
  );
}

/** One token with its certificates. */
function TokenCard({ token }: { readonly token: TokenView }): ReactElement {
  return (
    <div className="token">
      <div className="token-head">
        <Icon name="token" />
        <strong>{token.name}</strong>
        <span className="muted">Serial {token.serial}</span>
        {token.pinLocked ? <Tag tone="danger">PIN locked</Tag> : null}
        {token.pinFinalTry ? <Tag tone="warn">One PIN try left</Tag> : null}
      </div>
      {token.certificates.length === 0 ? <p className="muted">No certificates on this token.</p> : null}
      <ul className="certs">
        {token.certificates.map((cert) => (
          <li key={`${cert.holder}-${cert.validUntil}-${cert.issuer}`} className="cert">
            <div>
              <p className="cert-holder">{cert.holder}</p>
              <p className="muted">Issued by {cert.issuer}, valid until {cert.validUntil}</p>
            </div>
            <CertTags cert={cert} />
          </li>
        ))}
      </ul>
    </div>
  );
}

/** Driver list, for troubleshooting. */
function Drivers({ modules }: { readonly modules: readonly ModuleView[] }): ReactElement {
  return (
    <details className="details">
      <summary>Token drivers ({modules.length})</summary>
      <ul className="plain">
        {modules.map((module) => (
          <li key={module.path}>
            <strong>{module.family}</strong> <code>{module.path}</code>{" "}
            {module.loaded ? <Tag tone="success">Loaded</Tag> : null}
            {module.needsRosetta ? <Tag tone="warn">Built for Intel; open Swakshar using Rosetta</Tag> : null}
            {module.error !== null && !module.needsRosetta ? <Tag tone="danger">{module.error}</Tag> : null}
            {module.found ? null : <Tag tone="danger">File not found</Tag>}
          </li>
        ))}
      </ul>
    </details>
  );
}

/** The tokens card. */
export function TokensPanel({ tokens, error }: { readonly tokens: InventoryView | null; readonly error: string | null }): ReactElement {
  return (
    <Card title="Your tokens">
      {error === null ? null : <Notice tone="danger">{error}</Notice>}
      {tokens === null ? <p className="muted">Looking for tokens.</p> : null}
      {tokens !== null && tokens.tokens.length === 0 ? <p className="muted">No token connected. Plug in your DSC token.</p> : null}
      {tokens?.tokens.map((token) => <TokenCard key={`${token.name}-${token.serial}`} token={token} />)}
      {tokens === null ? null : <Drivers modules={tokens.modules} />}
    </Card>
  );
}
