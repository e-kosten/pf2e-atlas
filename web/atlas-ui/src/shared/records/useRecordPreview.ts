import { useCallback, useRef, useState } from "react";
import type { RecordPreviewAnchor } from "./recordPreviewTypes";
import type { RecordNavigationView } from "../../generated/atlas";
import { useRecordDetail } from "./useRecordDetail";

export function useRecordPreview() {
  const [recordKey, setRecordKey] = useState<string | null>(null);
  const [anchor, setAnchor] = useState<RecordPreviewAnchor | null>(null);
  const [selection, setSelection] = useState<RecordNavigationView>();
  const opener = useRef<HTMLElement | null>(null);
  const detail = useRecordDetail(recordKey, selection);

  const close = useCallback(() => {
    if (opener.current?.isConnected) opener.current.focus();
    opener.current = null;
    setRecordKey(null);
    setAnchor(null);
    setSelection(undefined);
  }, []);

  const open = useCallback(
    (
      nextRecordKey: string,
      anchor?: HTMLElement,
      navigation?: RecordNavigationView,
    ) => {
      opener.current ??=
        anchor ??
        (document.activeElement instanceof HTMLElement ? document.activeElement : null);
      setRecordKey(nextRecordKey);
      setAnchor((current) => current ?? anchor ?? null);
      setSelection(navigation);
    },
    [],
  );

  return {
    anchor,
    close,
    detail: detail.data,
    error: detail.error,
    retry: () => {
      void detail.refetch();
    },
    loading: detail.isLoading || detail.isFetching,
    open,
    recordKey,
    selection,
  };
}
