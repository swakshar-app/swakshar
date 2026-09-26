/**
 * The main window: a sidebar with four sections.
 */
import { type ReactElement, useEffect, useState } from "react";

import logo from "../../src-tauri/icons/128x128.png";
import { mainApi } from "../api/commands";
import { Icon, type IconName } from "../components/Icon";
import { ActivityView } from "./ActivityView";
import { HelpView } from "./HelpView";
import { HomeView } from "./HomeView";
import { SettingsView } from "./SettingsView";

/** Sections of the main window. */
type Section = "home" | "activity" | "settings" | "help";

/** Sidebar entries. */
const SECTIONS: ReadonlyArray<{ readonly id: Section; readonly label: string; readonly icon: IconName }> = [
  { id: "home", label: "Home", icon: "home" },
  { id: "activity", label: "Activity", icon: "clock" },
  { id: "settings", label: "Settings", icon: "sliders" },
  { id: "help", label: "Help", icon: "book" },
];

/** The view for a section. */
function SectionView({ section }: { readonly section: Section }): ReactElement {
  switch (section) {
    case "home":
      return <HomeView />;
    case "activity":
      return <ActivityView />;
    case "settings":
      return <SettingsView />;
    case "help":
      return <HelpView />;
  }
}

/** Sidebar plus the selected section. */
export function MainApp(): ReactElement {
  const [section, setSection] = useState<Section>("home");
  const [version, setVersion] = useState<string | null>(null);
  useEffect(() => {
    mainApi.overview().then(
      (overview) => setVersion(overview.appVersion),
      () => undefined,
    );
  }, []);
  return (
    <div className="shell">
      <nav className="sidebar" aria-label="Sections">
        <div className="brand">
          <img src={logo} alt="" width={28} height={28} />
          <span>Swakshar</span>
        </div>
        {SECTIONS.map((entry) => (
          <button
            key={entry.id}
            type="button"
            className={entry.id === section ? "nav-item nav-item-active" : "nav-item"}
            aria-current={entry.id === section ? "page" : undefined}
            onClick={() => setSection(entry.id)}
          >
            <Icon name={entry.icon} />
            <span>{entry.label}</span>
          </button>
        ))}
        {version === null ? null : <p className="sidebar-footer muted">Version {version}</p>}
      </nav>
      <main className="content">
        <SectionView section={section} />
      </main>
    </div>
  );
}
