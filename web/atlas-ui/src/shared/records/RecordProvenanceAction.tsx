import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { referenceDestination, useReferenceHelpNavigation } from "./ReferenceHelp";
import { BookCopy } from "lucide-react";
import type { ReferenceHelpView } from "../../generated/atlas";
import { PaneIconLink } from "../ui/actions/PaneAction";

export function RecordProvenanceAction({
  help,
  title,
}: {
  help: ReferenceHelpView;
  title?: string;
}) {
  const label = title ? `Open general entry: ${title}` : "Open general entry";
  const openHelp = useReferenceHelpNavigation();
  return (
    <PaneIconLink
      outlined
      href={destinationHref(
        referenceDestination(help.destination, help.definition_key),
      )}
      label={label}
      icon={<BookCopy size={16} aria-hidden="true" />}
      onClick={(event) => {
        if (!isPrimaryActivation(event)) return;
        event.preventDefault();
        openHelp(help, event.currentTarget);
      }}
    />
  );
}
