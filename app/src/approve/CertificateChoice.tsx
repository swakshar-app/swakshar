/**
 * Choosing the certificate. The one matching the PAN comes first.
 */
import type { ReactElement } from "react";

import type { CandidateView } from "../api/types";
import { Tag } from "../components/ui";

/** Radio list of eligible certificates. */
export function CertificateChoice(props: {
  readonly candidates: readonly CandidateView[];
  readonly selected: number;
  readonly onSelect: (index: number) => void;
}): ReactElement {
  const { candidates, selected, onSelect } = props;
  return (
    <fieldset className="choices">
      <legend>Sign with</legend>
      {candidates.map((candidate) => (
        <label key={candidate.index} className={candidate.index === selected ? "choice choice-selected" : "choice"}>
          <input type="radio" name="certificate" checked={candidate.index === selected} onChange={() => onSelect(candidate.index)} />
          <div className="choice-text">
            <p className="choice-holder">{candidate.holder}</p>
            <p className="muted">
              {candidate.classLabel}, valid until {candidate.validUntil}, issued by {candidate.issuer}
            </p>
            <p className="muted">
              {candidate.tokenName}, serial {candidate.tokenSerial}
            </p>
            {candidate.panMatch === "match" ? <Tag tone="success">Matches the PAN</Tag> : null}
            {candidate.panMatch === "mismatch" ? <Tag tone="danger">Different PAN; the portal will reject it</Tag> : null}
            {candidate.pinLocked ? <Tag tone="danger">PIN locked</Tag> : null}
          </div>
        </label>
      ))}
    </fieldset>
  );
}
