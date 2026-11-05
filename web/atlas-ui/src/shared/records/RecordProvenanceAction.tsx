import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { BookCopy } from "lucide-react";
import type { RecordNavigationView } from "../../generated/atlas";
import { PaneIconLink } from "../ui/actions/PaneAction";
import type { RecordReferenceHandler } from "./PreparedContent";

export function RecordProvenanceAction({
  navigation,
  title,
  onReference,
}: {
  navigation: RecordNavigationView;
  title?: string;
  onReference: RecordReferenceHandler;
}) {
  const label = title ? `Open general entry: ${title}` : "Open general entry";
  return (
    <PaneIconLink
      outlined
      href={destinationHref(
        recordDestination({
          recordKey: navigation.record_key,
          selection: navigation,
        }),
      )}
      label={label}
      icon={<BookCopy size={16} aria-hidden="true" />}
      onClick={(event) => {
        if (!isPrimaryActivation(event)) return;
        event.preventDefault();
        onReference(navigation.record_key, event.currentTarget, navigation);
      }}
    />
  );
}
