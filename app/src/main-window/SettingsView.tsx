/**
 * Settings: start at login, port, greeting version, extra sites and drivers.
 */
import { type ReactElement, useEffect, useState } from "react";

import { errorText, mainApi } from "../api/commands";
import type { Settings } from "../api/types";
import { useAction } from "../components/hooks";
import { Button, Card, Notice } from "../components/ui";
import { LocalCertificateCard } from "./LocalCertificateCard";

/** Ports the GST portal tries. */
const PORTS: readonly number[] = [1585, 2095, 2568, 2868, 4587];

/** Splits a textarea into trimmed, non-empty lines. */
function lines(text: string): string[] {
  return text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

/** The Settings section. */
export function SettingsView(): ReactElement {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [origins, setOrigins] = useState("");
  const [modules, setModules] = useState("");
  const [loadError, setLoadError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const action = useAction();
  useEffect(() => {
    mainApi.settings().then(
      (value) => {
        setSettings(value);
        setOrigins(value.extraOrigins.join("\n"));
        setModules(value.modules.join("\n"));
      },
      (reason: unknown) => setLoadError(errorText(reason)),
    );
  }, []);
  if (settings === null) {
    return <div className="page">{loadError === null ? <p className="muted">Loading.</p> : <Notice tone="danger">{loadError}</Notice>}</div>;
  }
  const update = (patch: Partial<Settings>): void => {
    setSaved(false);
    setSettings({ ...settings, ...patch });
  };
  const save = (): void => {
    void action.run(async () => {
      const next = await mainApi.saveSettings({ ...settings, extraOrigins: lines(origins), modules: lines(modules) });
      setSettings(next);
      setSaved(true);
    });
  };
  return (
    <div className="page">
      <Card title="General">
        <label className="field field-inline">
          <input type="checkbox" checked={settings.startAtLogin} onChange={(event) => update({ startAtLogin: event.target.checked })} />
          <span>Start Swakshar when you log in</span>
        </label>
      </Card>
      <Card title="Connection">
        <label className="field">
          <span>Signer port</span>
          <select value={settings.preferredPort ?? ""} onChange={(event) => update({ preferredPort: event.target.value === "" ? null : Number(event.target.value) })}>
            <option value="">Automatic (first free of the portal's five)</option>
            {PORTS.map((port) => (
              <option key={port} value={port}>{port}</option>
            ))}
          </select>
        </label>
        <label className="field">
          <span>Greeting version</span>
          <input value={settings.greetingVersion} onChange={(event) => update({ greetingVersion: event.target.value })} />
          <small className="muted">Change only if the portal says "Please install and use correct version of emSigner". It must match the portal's expected version.</small>
        </label>
        <label className="field">
          <span>Extra allowed sites</span>
          <textarea rows={3} value={origins} placeholder="https://example.gov.in" onChange={(event) => { setSaved(false); setOrigins(event.target.value); }} />
          <small className="muted">GST portal sites are always allowed. Add others only if you know why, one per line.</small>
        </label>
      </Card>
      <Card title="Token drivers">
        <label className="field">
          <span>Extra driver files</span>
          <textarea rows={3} value={modules} placeholder="/usr/local/lib/your-token-pkcs11.dylib" onChange={(event) => { setSaved(false); setModules(event.target.value); }} />
          <small className="muted">Full paths to PKCS#11 driver files, one per line. Common drivers are found automatically.</small>
        </label>
      </Card>
      <LocalCertificateCard />
      <div className="row">
        <Button variant="primary" onClick={save} disabled={action.busy}>Save settings</Button>
        {saved ? <span className="muted">Saved.</span> : null}
      </div>
      {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
    </div>
  );
}
