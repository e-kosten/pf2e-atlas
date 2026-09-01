import { Typography } from "antd";
import type { CreatureSurfaceSpellView } from "../../generated/atlas";
import { PreviewPopover, usePreviewPopoverClose } from "../ui/overlays/PreviewPopover";
import { RecordPreviewActions } from "./RecordPreviewActions";
import { RichContent, type ReferenceHandler } from "./RecordRichContent";
import { formatRank, formatSlug } from "./recordFormatting";

export function SpellOccurrencePreviewPopover({
  onOpenSpellRecord,
  onReference,
  spell,
}: {
  onOpenSpellRecord: (recordKey: string) => void;
  onReference: ReferenceHandler;
  spell: CreatureSurfaceSpellView;
}) {
  const target = spell.target_record_key;
  if (!target) {
    return <span>{spell.label}</span>;
  }
  const metadata = [
    formatRank(spell.rank),
    ...(spell.traits ?? []).map(formatSlug),
  ].join(" · ");

  return (
    <PreviewPopover
      actions={
        <SpellPopoverActions onOpenSpellRecord={() => onOpenSpellRecord(target)} />
      }
      ariaLabel={`${spell.label} spell details`}
      content={
        <div className="creature-sheet__standalone-spell">
          <span className="creature-sheet__spell-heading">
            <small>{metadata}</small>
          </span>
          {(spell.content ?? []).map((document) => (
            <RichContent
              content={document}
              key={document.content_key}
              onReference={onReference}
            />
          ))}
        </div>
      }
      title={spell.label}
    >
      {(open) => (
        <Typography.Link
          aria-expanded={open}
          aria-haspopup="dialog"
          href={`/records/${encodeURIComponent(target)}`}
        >
          {spell.label}
        </Typography.Link>
      )}
    </PreviewPopover>
  );
}

function SpellPopoverActions({ onOpenSpellRecord }: { onOpenSpellRecord: () => void }) {
  const close = usePreviewPopoverClose();
  return (
    <RecordPreviewActions
      closeLabel="Close spell preview"
      onClose={close}
      onOpenFullPage={() => {
        close();
        onOpenSpellRecord();
      }}
      openLabel="Open spell record"
    />
  );
}
