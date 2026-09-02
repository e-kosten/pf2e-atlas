import { Button, Typography } from "antd";
import { X } from "lucide-react";
import type { CreatureSurfaceSpellView } from "../../generated/atlas";
import { PreviewPopover, usePreviewPopoverClose } from "../ui/overlays/PreviewPopover";
import { RecordPreviewActions } from "./RecordPreviewActions";
import { RecordPreviewContext } from "./RecordPreviewContext";
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
  const content = spell.content ?? [];
  if (!target && !content.length) {
    return <span>{spell.label}</span>;
  }
  const metadata = [
    formatRank(spell.rank),
    ...(spell.traits ?? []).map(formatSlug),
    ...spellContext(spell),
  ].join(" · ");

  return (
    <PreviewPopover
      actions={
        target ? (
          <SpellPopoverActions onOpenSpellRecord={() => onOpenSpellRecord(target)} />
        ) : (
          <TargetlessSpellPopoverActions />
        )
      }
      ariaLabel={`${spell.label} spell details`}
      content={
        <RecordPreviewContext.Provider value={null}>
          <div className="creature-sheet__standalone-spell">
            <span className="creature-sheet__spell-heading">
              <small>{metadata}</small>
            </span>
            {content.map((document) => (
              <RichContent
                content={document}
                key={document.content_key}
                onReference={onReference}
              />
            ))}
          </div>
        </RecordPreviewContext.Provider>
      }
      title={spell.label}
    >
      {(open) =>
        target ? (
          <Typography.Link
            aria-expanded={open}
            aria-haspopup="dialog"
            href={`/records/${encodeURIComponent(target)}`}
          >
            {spell.label}
          </Typography.Link>
        ) : (
          <Button
            aria-expanded={open}
            aria-haspopup="dialog"
            className="creature-sheet__spell-trigger"
            size="small"
            type="link"
          >
            {spell.label}
          </Button>
        )
      }
    </PreviewPopover>
  );
}

function TargetlessSpellPopoverActions() {
  const close = usePreviewPopoverClose();
  return (
    <Button
      aria-label="Close spell preview"
      icon={<X size={14} />}
      onClick={close}
      size="small"
    />
  );
}

function spellContext(spell: CreatureSurfaceSpellView) {
  const context = spell.context;
  if (!context) return [];
  return [
    context.contextual_label,
    context.group ? `Group ${context.group}` : undefined,
    context.uses?.maximum === undefined
      ? undefined
      : `${context.uses.maximum} ${context.uses.maximum === 1 ? "use" : "uses"}`,
  ].filter((detail): detail is string => Boolean(detail));
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
