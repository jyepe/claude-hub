import { useCallback, useEffect, useRef, useState } from "react";

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

  const cancelledRef = useRef(false);

  useEffect(() => {
    cancelledRef.current = false;
    let timerId: number | undefined;

    const poll = async () => {
      await refresh();
      if (cancelledRef.current) return;
      timerId = window.setTimeout(poll, intervalMs);
    };

    poll();

    return () => {
      cancelledRef.current = true;
      if (timerId !== undefined) window.clearTimeout(timerId);
    };
  }, [refresh, intervalMs]);

  return { data, error, lastRefresh, refresh };
}
