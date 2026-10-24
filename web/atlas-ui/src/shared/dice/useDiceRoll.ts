import { AtlasApiError } from "../../api/atlasApi";
import { useEffect, useRef, useState } from "react";
import { waitForRollReveal } from "./rollLifecycle";

/** Owns presentation timing only. The supplied action determines and retains outcomes. */
export function useDiceRoll<T>({
  retainResultOnError = false,
}: { retainResultOnError?: boolean } = {}) {
  const [result, setResult] = useState<T>();
  const [busy, setBusy] = useState(false);
  const [animating, setAnimating] = useState(false);
  const [error, setError] = useState<string>();
  const inFlight = useRef(false);
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  const run = async (action: () => Promise<T>) => {
    if (inFlight.current) return;
    inFlight.current = true;
    setBusy(true);
    setError(undefined);
    try {
      const next = await action();
      if (!mounted.current) return;
      setResult(next);
      setAnimating(true);
      await waitForRollReveal();
    } catch (cause) {
      if (mounted.current) {
        if (!retainResultOnError) setResult(undefined);
        setError(
          cause instanceof AtlasApiError && cause.status < 500
            ? cause.message
            : "Couldn’t confirm the roll.",
        );
      }
    } finally {
      inFlight.current = false;
      if (mounted.current) {
        setAnimating(false);
        setBusy(false);
      }
    }
  };
  return { result, busy, animating, error, run };
}
