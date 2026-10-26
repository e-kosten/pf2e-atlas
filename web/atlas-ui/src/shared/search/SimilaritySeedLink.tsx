import { ConfigProvider, Typography } from "antd";
import type { SimilaritySeed } from "../../generated/atlas";
import { recordDestination, traitDestination } from "../../app/destinations";
import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";
import { RecordReferenceDrawer } from "../records/RecordReferenceDrawer";
import { useRecordPreview } from "../records/useRecordPreview";

/** A criterion references its source without changing the selected search result. */
export function SimilaritySeedLink({
  seed,
  title,
  disabled,
}: {
  seed: SimilaritySeed;
  title: string;
  disabled?: boolean;
}) {
  const { componentDisabled } = ConfigProvider.useConfig();
  const unavailable = disabled ?? componentDisabled;
  if (seed.kind === "trait")
    return (
      <Typography.Link
        disabled={unavailable}
        href={destinationHref(traitDestination({ traitKey: seed.trait_key }))}
      >
        {title}
      </Typography.Link>
    );
  return (
    <RecordSeedLink
      key={seed.record_key}
      recordKey={seed.record_key}
      title={title}
      disabled={unavailable}
    />
  );
}

function RecordSeedLink({
  recordKey,
  title,
  disabled,
}: {
  recordKey: string;
  title: string;
  disabled?: boolean;
}) {
  const preview = useRecordPreview();
  return (
    <>
      <Typography.Link
        href={destinationHref(recordDestination({ recordKey }))}
        disabled={disabled}
        onClick={(event) => {
          if (!isPrimaryActivation(event)) return;
          event.preventDefault();
          preview.open(recordKey, event.currentTarget);
        }}
      >
        {title}
      </Typography.Link>
      {preview.recordKey && (
        <RecordReferenceDrawer
          anchor={preview.anchor}
          readingHistory={preview.readingHistory}
          landing={preview.landing}
          detail={preview.detail}
          loading={preview.loading}
          error={preview.error}
          onRetry={preview.retry}
          onClose={preview.close}
          onReference={preview.open}
          onRankChange={preview.changeRank}
          onOpenFullPage={() => {
            navigateDestination(
              recordDestination({
                recordKey: preview.recordKey!,
                selection: preview.selection,
                landing: preview.landing,
              }),
            );
            preview.close();
          }}
        />
      )}
    </>
  );
}
