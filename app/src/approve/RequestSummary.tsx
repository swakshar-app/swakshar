/**
 * What is being signed, and for which site.
 */
import type { ReactElement } from "react";

import type { PendingView } from "../api/types";
import { hostOf } from "../components/format";
import { Icon } from "../components/Icon";

/** Site, purpose and, for documents, the exact fingerprint signed. */
export function RequestSummary({ pending }: { readonly pending: PendingView }): ReactElement {
  const registration = pending.kind === "registration";
  return (
    <dl className="summary">
      <div>
        <dt>Site</dt>
        <dd className="site">
          <Icon name="lock" size={15} />
          {hostOf(pending.origin)}
        </dd>
      </div>
      <div>
        <dt>Purpose</dt>
        <dd>{registration ? `DSC registration for PAN ${pending.content}` : "A GST return or form"}</dd>
      </div>
      {registration ? null : (
        <div>
          <dt>Document fingerprint</dt>
          <dd>
            <code className="hash">{pending.content}</code>
          </dd>
        </div>
      )}
      {registration || pending.panMasked === null ? null : (
        <div>
          <dt>PAN</dt>
          <dd>{pending.panMasked}</dd>
        </div>
      )}
    </dl>
  );
}
