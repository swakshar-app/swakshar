/** Explicit certificate refreshes discard stale PIN flags before allowing another attempt. */
import { useCallback, useEffect, useRef, useState } from "react";

import { errorText } from "../api/commands";
import { signingApi, type PdfStatus } from "../api/signing";

/** Reads certificates on entry and on demand; never polls during a signature. */
export function useCertificates(): {
  readonly data: PdfStatus | null;
  readonly error: string | null;
  readonly refresh: () => Promise<void>;
} {
  const [data, setData] = useState<PdfStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const generation = useRef(0);
  const refresh = useCallback(async (): Promise<void> => {
    const current = ++generation.current;
    setData(null); setError(null);
    try {
      const next = await signingApi.pdfStatus();
      if (current === generation.current) { setData(next); }
    } catch (reason: unknown) {
      if (current === generation.current) { setError(errorText(reason)); }
    }
  }, []);
  useEffect(() => {
    void refresh();
    return () => { generation.current += 1; };
  }, [refresh]);
  return { data, error, refresh };
}
