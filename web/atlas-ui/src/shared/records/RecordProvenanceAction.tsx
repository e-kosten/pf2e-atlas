import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { referenceDestination, useReferenceHelpNavigation } from "./ReferenceHelp";
import { BookCopy } from "lucide-react";
import type { RecordNavigationView } from "../../generated/atlas";
import { PaneIconLink } from "../ui/actions/PaneAction";

export function RecordProvenanceAction({
  navigation,
  title,
}: {
  navigation: RecordNavigationView;
  title?: string;
}) {
  const label = title ? `Open general entry: ${title}` : "Open general entry";
  const openHelp = useReferenceHelpNavigation();
  const help = {
    definition_key: null,
    destination: {
      kind: "record" as const,
      navigation,
    },
  };
  return (
    <PaneIconLink
      outlined
      href={destinationHref(referenceDestination(help.destination))}
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
