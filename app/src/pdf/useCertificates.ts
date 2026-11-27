/** Explicit certificate refreshes discard stale PIN flags before allowing another attempt. */
import { useCallback, useEffect, useRef, useState } from "react";

import { errorText } from "../api/commands";
import { signingApi, type PdfStatus } from "../api/signing";

/** Reads certificates on entry and on demand; never polls during a signature. */
export function useCertificates(): {
  readonly data: PdfStatus | null;
  readonly error: string | null;
  readonly loading: boolean;
  readonly refresh: () => Promise<void>;
} {
  const [data, setData] = useState<PdfStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const generation = useRef(0);
  const pending = useRef(false);
  const load = useCallback(async (current: number): Promise<void> => {
    try {
      const next = await signingApi.pdfStatus();
      if (current === generation.current) { setData(next); }
    } catch (reason: unknown) {
      if (current === generation.current) { setError(errorText(reason)); }
    } finally { pending.current = false; if (current === generation.current) { setLoading(false); } }
  }, []);
  /** User refresh clears stale PIN flags before starting the new read. */
  const refresh = useCallback(async (): Promise<void> => {
    if (pending.current) { return; }
    pending.current = true;
    const current = ++generation.current;
    setData(null); setError(null); setLoading(true);
    await load(current);
  }, [load]);
  useEffect(() => {
    pending.current = true;
    void load(++generation.current);
    return () => { generation.current += 1; };
  }, [load]);
  return { data, error, loading, refresh };
}
