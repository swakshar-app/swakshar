/**
 * Small building blocks shared by both windows: buttons, cards, tags and
 * notices. Status is always text (with an optional icon), never a dot.
 */
import type { ReactElement, ReactNode } from "react";

import { Icon, type IconName } from "./Icon";

/** Tone of a tag or notice. */
export type Tone = "success" | "warn" | "danger" | "neutral";

/** Icons that accompany each tone. */
const TONE_ICONS: Record<Tone, IconName | null> = {
  success: "check",
  warn: "alert",
  danger: "alert",
  neutral: null,
};

/** A button. `primary` is used once per view, for the main action. */
export function Button(props: {
  readonly children: ReactNode;
  readonly onClick?: () => void;
  readonly variant?: "primary" | "secondary" | "danger" | "quiet";
  readonly disabled?: boolean;
  readonly type?: "button" | "submit";
  readonly icon?: IconName;
}): ReactElement {
  const { children, onClick, variant = "secondary", disabled = false, type = "button", icon } = props;
  return (
    <button className={`button button-${variant}`} type={type} onClick={onClick} disabled={disabled}>
      {icon === undefined ? null : <Icon name={icon} size={16} />}
      <span>{children}</span>
    </button>
  );
}

/** A titled surface. */
export function Card(props: { readonly title?: string; readonly actions?: ReactNode; readonly children: ReactNode }): ReactElement {
  const { title, actions, children } = props;
  return (
    <section className="card">
      {title === undefined && actions === undefined ? null : (
        <header className="card-header">
          {title === undefined ? <span /> : <h2>{title}</h2>}
          {actions ?? null}
        </header>
      )}
      <div className="card-body">{children}</div>
    </section>
  );
}

/** A short status label. */
export function Tag({ tone, children }: { readonly tone: Tone; readonly children: ReactNode }): ReactElement {
  const icon = TONE_ICONS[tone];
  return (
    <span className={`tag tag-${tone}`}>
      {icon === null ? null : <Icon name={icon} size={13} />}
      {children}
    </span>
  );
}

/** A message box. */
export function Notice({ tone, children }: { readonly tone: Tone; readonly children: ReactNode }): ReactElement {
  const icon = TONE_ICONS[tone];
  return (
    <div className={`notice notice-${tone}`} role={tone === "danger" ? "alert" : "status"}>
      {icon === null ? null : <Icon name={icon} size={16} />}
      <div>{children}</div>
    </div>
  );
}
