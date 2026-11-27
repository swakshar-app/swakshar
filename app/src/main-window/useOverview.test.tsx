import { cleanup, renderHook } from "@testing-library/react";
import { type ReactElement, type ReactNode } from "react";
import { afterEach, expect, it, vi } from "vitest";

import { overview } from "../testing/fixtures";
import { OverviewContext, useOverview } from "./useOverview";

const api = vi.hoisted(() => ({ overview: vi.fn() }));
vi.mock("../api/commands", () => ({ mainApi: api, errorText: String }));
vi.mock("../components/visibility", () => ({ useWindowVisible: () => true }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it("shares shell status without starting another backend probe", () => {
  const shared = { data: overview(), error: null, refresh: vi.fn() };
  /** Supplies the main shell's already loaded overview. */
  function wrapper({ children }: { readonly children: ReactNode }): ReactElement {
    return <OverviewContext.Provider value={shared}>{children}</OverviewContext.Provider>;
  }
  const { result } = renderHook(() => useOverview(), { wrapper });
  expect(result.current).toBe(shared);
  result.current.refresh();
  expect(shared.refresh).toHaveBeenCalledTimes(1);
  expect(api.overview).not.toHaveBeenCalled();
});
