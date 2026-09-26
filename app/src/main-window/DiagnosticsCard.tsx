/**
 * Help's Report a problem card: a diagnostic report the user reads, copies
 * and pastes into an issue. Swakshar sends nothing itself.
 */
import { type ReactElement, useRef, useState } from "react";

import { mainApi } from "../api/commands";
import { useAction } from "../components/hooks";
import { Button, Card, Notice } from "../components/ui";

/** The Report a problem card. */
export function DiagnosticsCard(): ReactElement {
  const [report, setReport] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const field = useRef<HTMLTextAreaElement>(null);
  const action = useAction();
  const create = (): void => {
    void action.run(async () => {
      setCopied(false);
      setReport(await mainApi.diagnosticReport());
    });
  };
  const copy = (): void => {
    if (report === null) {
      return;
    }
    navigator.clipboard.writeText(report).then(
      () => setCopied(true),
      () => field.current?.select(),
    );
  };
  return (
    <Card title="Report a problem">
      <p className="muted">
        The report lists versions, the checklist, drivers and recent log lines. It leaves out names, PANs and token serials, and nothing is sent: read it, then paste it into an issue yourself.
      </p>
      <div className="row">
        <Button onClick={create} disabled={action.busy}>{report === null ? "Create diagnostic report" : "Create again"}</Button>
        {report === null ? null : <Button onClick={copy}>{copied ? "Copied" : "Copy report"}</Button>}
        <Button variant="quiet" icon="external" onClick={() => void action.run(mainApi.openIssuePage)}>Report on GitHub</Button>
      </div>
      {report === null ? null : <textarea ref={field} className="report" readOnly value={report} rows={12} />}
      {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
    </Card>
  );
}
