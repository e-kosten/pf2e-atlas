import { RotateCw, X } from "lucide-react";
import { isSharedFailure } from "../feedback/errorMessages";
import { isArtifactGenerationChanged } from "../queries/referenceQueries";
import { PaneIconButton } from "../ui/actions/PaneAction";
import type { usePreparedReferenceSelection } from "./usePreparedReferenceSelection";

export function hasReferenceSelectionStatus(
  preparing: ReturnType<typeof usePreparedReferenceSelection>["current"],
) {
  return Boolean(
    preparing &&
    !isSharedFailure(preparing.error) &&
    (preparing.slow || preparing.error),
  );
}

export function ReferenceSelectionStatus({
  selection,
  onRetry = selection.retry,
}: {
  selection: ReturnType<typeof usePreparedReferenceSelection>;
  onRetry?: () => void;
}) {
  const preparing = selection.current;
  if (!preparing || !hasReferenceSelectionStatus(preparing)) return null;
  const generationChanged = isArtifactGenerationChanged(preparing.error);
  return (
    <div className="reference-selection-status" role="status">
      <span title={preparing.name}>
        {preparing.error
          ? generationChanged
            ? "Reference collection changed"
            : `Could not load ${preparing.name}`
          : `Loading ${preparing.name}…`}
      </span>
      {preparing.error && (
        <PaneIconButton
          size="small"
          icon={<RotateCw size={14} />}
          label={
            generationChanged ? "Refresh results" : `Retry loading ${preparing.name}`
          }
          onClick={onRetry}
        />
      )}
      <PaneIconButton
        size="small"
        icon={<X size={14} />}
        label="Cancel selection"
        onClick={selection.cancel}
      />
    </div>
  );
}
