import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { Button, Tooltip } from "antd";
import { Layers } from "lucide-react";
import { useRef, useState } from "react";
import type {
  EffectReferenceLabelView,
  RecordNavigationView,
} from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";
/** The server chooses labels from source facts; this control only renders them. */
export function EffectReferenceLink({
  label,
  target,
  onReference,
}: {
  label: EffectReferenceLabelView;
  target: RecordNavigationView;
  onReference: RecordReferenceHandler;
}) {
  const [tipOpen, setTipOpen] = useState(false);
  const link = useRef<HTMLAnchorElement>(null);
  return (
    <Tooltip
      title={label.title}
      trigger={["hover", "focus"]}
      open={tipOpen}
      onOpenChange={(open) =>
        setTipOpen(open && link.current?.getAttribute("aria-expanded") !== "true")
      }
    >
      <Button
        ref={link}
        type="link"
        size="small"
        className="effect-reference-link"
        icon={<Layers size={13} aria-hidden="true" />}
        aria-label={`Preview ${label.title}`}
        href={destinationHref(
          recordDestination({
            recordKey: target.record_key,
            selection: target,
          }),
        )}
        onClick={(event) => {
          if (!isPrimaryActivation(event)) return;
          event.preventDefault();
          setTipOpen(false);
          onReference(target.record_key, event.currentTarget, target);
        }}
      >
        {label.text}
      </Button>
    </Tooltip>
  );
}
