import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";
import { traitDestination } from "../../app/destinations";
import { Tag, Typography } from "antd";
import type { RecordBadgeView } from "../../generated/atlas";
import { sentenceCase } from "../ui/labels";
/** Definition navigation is supplied by Rust; other badge namespaces stay plain. */
export function TraitBadge({ trait }: { trait: RecordBadgeView }) {
  const label = sentenceCase(trait.label);
  const route = trait.trait_key
    ? traitDestination({
        traitKey: trait.trait_key,
      })
    : null;
  return (
    <Tag className="record-trait-badge">
      {route ? (
        <Typography.Link
          style={{ fontSize: "inherit", lineHeight: "inherit" }}
          href={destinationHref(route)}
          onClick={(event) => {
            event.stopPropagation();
            if (!isPrimaryActivation(event)) return;
            event.preventDefault();
            navigateDestination(route);
          }}
        >
          {label}
        </Typography.Link>
      ) : (
        label
      )}
    </Tag>
  );
}
