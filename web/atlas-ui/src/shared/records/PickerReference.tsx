import { Button } from "antd";
import { AtlasLink } from "../../app/AtlasLink";
import { isPrimaryActivation, navigateDestination } from "../../app/navigation";
import { DefinitionDetailPane } from "./DefinitionDetailPane";
import { RecordDetailPane } from "./RecordDetailPane";
import { referenceDestination } from "./ReferenceHelp";
import { readingNavigationForUrl } from "./readingUrlNavigation";
import { usePreviewReadingPosition } from "./usePreviewReadingPosition";
import type { useRecordPreview } from "./useRecordPreview";

export function PickerReference({
  preview,
  label,
}: {
  preview: ReturnType<typeof useRecordPreview>;
  label: string;
}) {
  const { destination, detail, definition, readingHistory, landing } = preview;
  const { ref: contentRef, onLandingReady } = usePreviewReadingPosition(
    definition ?? detail,
    readingHistory,
    true,
    landing,
  );
  if (!destination) return null;
  const navigation = detail
    ? readingNavigationForUrl(detail.selected, { navigation: preview.selection })
    : undefined;
  const route = referenceDestination(
    destination.kind === "record" && navigation
      ? { ...destination, navigation }
      : destination,
    preview.help?.definition_key,
    landing,
  );
  const back = readingHistory.canGoBack ? readingHistory.onBack : preview.close;
  return (
    <div className="picker-reference" data-reference-host>
      <div className="picker-reference__actions" data-reference-header>
        <Button type="text" onClick={back}>
          {readingHistory.canGoBack
            ? "Back"
            : "Back to " + label.toLowerCase() + " choices"}
        </Button>
        <AtlasLink
          {...route}
          onClick={(event) => {
            if (!isPrimaryActivation(event)) return;
            event.preventDefault();
            navigateDestination(route, { dismissReadingOverlays: true });
          }}
        >
          Open full reference
        </AtlasLink>
      </div>
      <div className="picker-reference__body" data-reference-body>
        <section ref={contentRef} aria-label="Reference content">
          {destination.kind === "definition" ? (
            <DefinitionDetailPane
              detail={definition}
              loading={preview.loading}
              error={preview.error}
              onRetry={preview.retry}
              retryLabel={preview.retryLabel}
              onReference={preview.open}
            />
          ) : (
            <RecordDetailPane
              detail={detail}
              showRelated={false}
              loading={preview.loading}
              error={preview.error}
              onRetry={preview.retry}
              onReload={preview.reload}
              retryLabel={preview.retryLabel}
              reloadLabel={preview.reloadLabel}
              onReference={preview.open}
              onRankChange={preview.changeRank}
              landing={landing}
              onLandingReady={onLandingReady}
            />
          )}
        </section>
      </div>
    </div>
  );
}
