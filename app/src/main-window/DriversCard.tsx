/**
 * Settings card for driver files outside the usual locations. Changes save
 * at once; the file picker is native and opened by the app, not the page.
 */
import type { ReactElement } from "react";

import { mainApi } from "../api/commands";
import { useAction } from "../components/hooks";
import { Button, Card, Notice } from "../components/ui";

/** The Token drivers card. */
export function DriversCard(props: { readonly modules: readonly string[]; readonly onModules: (modules: readonly string[]) => void }): ReactElement {
  const { modules, onModules } = props;
  const action = useAction();
  const add = (): void => {
    void action.run(async () => {
      const next = await mainApi.addDriver();
      if (next !== null) {
        onModules(next.modules);
      }
    });
  };
  const remove = (path: string): void => {
    void action.run(async () => {
      onModules((await mainApi.removeDriver(path)).modules);
    });
  };
  return (
    <Card title="Token drivers">
      <p className="muted">Swakshar finds the usual token drivers on its own. Add a driver file here only if yours is installed somewhere else.</p>
      {modules.length === 0 ? null : (
        <ul className="plain">
          {modules.map((path) => (
            <li key={path} className="driver-row">
              <code>{path}</code>
              <Button variant="quiet" onClick={() => remove(path)} disabled={action.busy}>Remove</Button>
            </li>
          ))}
        </ul>
      )}
      <div className="row">
        <Button icon="plus" onClick={add} disabled={action.busy}>Add driver file</Button>
      </div>
      {action.error === null ? null : <Notice tone="danger">{action.error}</Notice>}
    </Card>
  );
}
