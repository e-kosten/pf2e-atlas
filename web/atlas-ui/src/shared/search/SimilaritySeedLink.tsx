import { ConfigProvider } from "antd";
import type { ReferenceHelpView, SimilaritySeed } from "../../generated/atlas";
import { AtlasLink } from "../../app/AtlasLink";
import { recordDestination } from "../../app/destinations";
import { isPrimaryActivation } from "../../app/navigation";
import {
  ReferenceHelpLink,
  useReferenceRecordNavigation,
} from "../records/ReferenceHelp";

/** The owning Search response supplies the criterion's reading capability. */
export function SimilaritySeedLink({
  seed,
  title,
  disabled,
  help,
  generation,
}: {
  seed: SimilaritySeed;
  title: string;
  disabled?: boolean;
  help?: ReferenceHelpView;
  generation?: string;
}) {
  const onReference = useReferenceRecordNavigation();
  const { componentDisabled } = ConfigProvider.useConfig();
  const unavailable = disabled ?? componentDisabled;
  if (seed.kind === "trait")
    return unavailable ? (
      title
    ) : (
      <ReferenceHelpLink help={help} generation={generation}>
        {title}
      </ReferenceHelpLink>
    );
  return (
    <AtlasLink
      {...recordDestination({ recordKey: seed.record_key })}
      disabled={unavailable}
      onClick={(event) => {
        if (unavailable || !onReference || !isPrimaryActivation(event)) return;
        event.preventDefault();
        onReference(seed.record_key, event.currentTarget);
      }}
    >
      {title}
    </AtlasLink>
  );
}
