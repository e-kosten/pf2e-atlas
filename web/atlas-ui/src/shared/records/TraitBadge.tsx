import { DisplayChip } from "../ui/DisplayChip";
import type { RecordBadgeView } from "../../generated/atlas";
import { ReferenceHelpLink } from "./ReferenceHelp";
import { sentenceCase } from "../ui/labels";
/** Definition navigation is supplied by Rust; other badge namespaces stay plain. */
export function TraitBadge({
  trait,
  linked = true,
}: {
  trait: RecordBadgeView;
  linked?: boolean;
}) {
  const label = sentenceCase(trait.label);
  return (
    <DisplayChip className="record-trait-badge">
      <ReferenceHelpLink help={linked ? trait.help : null}>{label}</ReferenceHelpLink>
    </DisplayChip>
  );
}
