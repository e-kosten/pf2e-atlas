import { skipToken, useQuery } from "@tanstack/react-query";
import { useLayoutEffect, useId, useRef, useState } from "react";

export function useSettingsOpening<T>(
  read: (ref: string, signal: AbortSignal) => Promise<T>,
  initialRef?: string,
) {
  const owner = useId();
  const sequence = useRef(0);
  const [request, setRequest] = useState<
    | {
        ref: string;
        attempt: number | "direct";
        closing: boolean;
        opener: HTMLElement | null;
      }
    | undefined
  >(
    initialRef
      ? { ref: initialRef, attempt: "direct", closing: false, opener: null }
      : undefined,
  );
  const [previousInitialRef, setPreviousInitialRef] = useState(initialRef);
  if (initialRef !== previousInitialRef) {
    setPreviousInitialRef(initialRef);
    setRequest(
      initialRef
        ? { ref: initialRef, attempt: "direct", closing: false, opener: null }
        : undefined,
    );
  }
  const query = useQuery({
    queryKey: ["settings-opening", owner, request?.ref, request?.attempt],
    queryFn: request ? ({ signal }) => read(request.ref, signal) : skipToken,
    enabled: !!request && !request.closing,
    staleTime: Infinity,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    gcTime: 0,
    retry: false,
  });
  const preparing = !!request && !request.closing && query.isPending;
  useLayoutEffect(() => {
    const trigger = request?.opener;
    if (!preparing || !trigger) return;
    const document = trigger.ownerDocument;
    const leave = (event: Event) => {
      if (!event.composedPath().includes(trigger)) setRequest(undefined);
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape") setRequest(undefined);
    };
    document.addEventListener("pointerdown", leave, true);
    document.addEventListener("focusin", leave, true);
    document.addEventListener("keydown", escape, true);
    return () => {
      document.removeEventListener("pointerdown", leave, true);
      document.removeEventListener("focusin", leave, true);
      document.removeEventListener("keydown", escape, true);
    };
  }, [preparing, request?.opener]);
  return {
    ref: request?.ref,
    opener: request?.opener,
    data: query.data,
    error: query.error,
    preparing,
    open: !!query.data && !request?.closing,
    begin(ref: string, trigger: HTMLElement | null) {
      if (preparing && request?.ref === ref) return;
      setRequest({ ref, attempt: ++sequence.current, closing: false, opener: trigger });
    },
    retry: () => void query.refetch(),
    close: () => setRequest((current) => current && { ...current, closing: true }),
    clear: () => setRequest((current) => (current?.closing ? undefined : current)),
    abandon: () => setRequest(undefined),
  };
}
