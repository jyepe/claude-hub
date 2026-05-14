import { useCallback, useEffect, useState } from "react";

export function usePoll<T>(fetcher: () => Promise<T>, intervalMs = 30_000) {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [lastRefresh, setLastRefresh] = useState<Date | null>(null);

  const refresh = useCallback(async () => {
    try {
      const next = await fetcher();
      setData(next);
      setError(null);
      setLastRefresh(new Date());
    } catch (e) {
      setError(e);
    }
  }, [fetcher]);

  useEffect(() => {
    refresh();
    const id = window.setInterval(refresh, intervalMs);
    return () => window.clearInterval(id);
  }, [refresh, intervalMs]);

  return { data, error, lastRefresh, refresh };
}
