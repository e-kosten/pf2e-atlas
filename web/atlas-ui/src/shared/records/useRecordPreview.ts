import { useQuery } from "@tanstack/react-query";
import { useCallback, useState } from "react";
import { getRecordDetail } from "../../api/atlasApi";
import type { RecordPreviewAnchor } from "./recordPreviewTypes";

export function useRecordPreview() {
  const [recordKey, setRecordKey] = useState<string | null>(null);
  const [anchor, setAnchor] = useState<RecordPreviewAnchor | null>(null);
  const detail = useQuery({
    queryKey: ["record-preview", recordKey],
    enabled: recordKey !== null,
    queryFn: () => getRecordDetail(recordKey!),
  });

  const close = useCallback(() => {
    setRecordKey(null);
    setAnchor(null);
  }, []);

  const open = useCallback((nextRecordKey: string, anchorRect?: DOMRect) => {
    setRecordKey(nextRecordKey);
    setAnchor(anchorRect ? recordPreviewAnchorFromRect(anchorRect) : null);
  }, []);

  return {
    anchor,
    close,
    detail: detail.data,
    error: detail.error,
    loading: detail.isLoading || detail.isFetching,
    open,
    recordKey,
  };
}

function recordPreviewAnchorFromRect(rect: DOMRect): RecordPreviewAnchor {
  return {
    top: rect.top,
    right: rect.right,
    bottom: rect.bottom,
    left: rect.left,
    width: rect.width,
    height: rect.height,
  };
}
