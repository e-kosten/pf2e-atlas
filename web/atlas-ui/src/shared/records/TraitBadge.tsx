import { Tag } from "antd";
import type { RecordBadgeView } from "../../generated/atlas";
import { ReferenceHelpLink } from "./ReferenceHelp";
import { sentenceCase } from "../ui/labels";
/** Definition navigation is supplied by Rust; other badge namespaces stay plain. */
export function TraitBadge({ trait }: { trait: RecordBadgeView }) {
  const label = sentenceCase(trait.label);
  return (
    <Tag className="record-trait-badge">
      <ReferenceHelpLink help={trait.help}>{label}</ReferenceHelpLink>
    </Tag>
  );
}
