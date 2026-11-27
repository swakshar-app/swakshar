/**
 * The main window: a sidebar with six sections.
 */
import { type ReactElement, useState } from "react";

import logo from "../../src-tauri/icons/128x128.png";
import { mainApi } from "../api/commands";
import { usePolling } from "../components/hooks";
import { Icon, type IconName } from "../components/Icon";
import { ActivityView } from "./ActivityView";
import { HelpView } from "./HelpView";
import { HomeView } from "./HomeView";
import { SettingsView } from "./SettingsView";
import { UpdateToast } from "./UpdateToast";
import { PortalView } from "./PortalView";
import { PdfSigningView } from "../pdf/PdfSigningView";

/** How often the shell refreshes the version and update state. */
const SHELL_INTERVAL_MS = 3000;

/** Sections of the main window. */
type Section = "home" | "portal" | "pdf" | "activity" | "settings" | "help";

/** Sidebar entries. */
const SECTIONS: ReadonlyArray<{ readonly id: Section; readonly label: string; readonly icon: IconName }> = [
  { id: "home", label: "Home", icon: "home" },
  { id: "portal", label: "Income Tax", icon: "shield" },
  { id: "pdf", label: "Sign PDF", icon: "document" },
  { id: "activity", label: "Activity", icon: "clock" },
  { id: "settings", label: "Settings", icon: "sliders" },
  { id: "help", label: "Help", icon: "book" },
];

/** The view for a section. */
function SectionView({ section }: { readonly section: Section }): ReactElement {
  switch (section) {
    case "home":
      return <HomeView />;
    case "portal":
      return <PortalView />;
    case "pdf":
      return <PdfSigningView />;
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
  const overview = usePolling(mainApi.overview, SHELL_INTERVAL_MS);
  const version = overview.data?.appVersion ?? null;
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
      {overview.data === null ? null : <UpdateToast update={overview.data.update} onChange={overview.refresh} />}
    </div>
  );
}
