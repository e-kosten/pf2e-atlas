import { Tag, Typography } from "antd";
import type { RecordBadgeView } from "../../generated/atlas";
import {
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import { sentenceCase } from "../ui/labels";

/** Definition navigation is supplied by Rust; other badge namespaces stay plain. */
export function TraitBadge({ trait }: { trait: RecordBadgeView }) {
  const label = sentenceCase(trait.label);
  const route = trait.trait_key
    ? { kind: "trait" as const, traitKey: trait.trait_key }
    : null;
  return (
    <Tag>
      {route ? (
        <Typography.Link
          href={atlasRoutePath(route)}
          onClick={(event) => {
            event.stopPropagation();
            if (!shouldHandleAtlasRouteClick(event)) return;
            event.preventDefault();
            navigateToAtlasRoute(route);
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
