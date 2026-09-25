/**
 * Entry point. One bundle serves both windows; the window label picks the view.
 */
import "./styles/base.css";
import "./styles/components.css";
import "./styles/layout.css";
import "./styles/approve.css";

import { getCurrentWindow } from "@tauri-apps/api/window";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { ApproveApp } from "./approve/ApproveApp";
import { MainApp } from "./main-window/MainApp";

/** Label of the approval window, as in tauri.conf.json. */
const APPROVE_WINDOW = "approve";

/** Mounts the view for this window. */
function mount(): void {
  const container = document.getElementById("root");
  if (container === null) {
    return;
  }
  const isApproval = getCurrentWindow().label === APPROVE_WINDOW;
  createRoot(container).render(<StrictMode>{isApproval ? <ApproveApp /> : <MainApp />}</StrictMode>);
}

mount();
