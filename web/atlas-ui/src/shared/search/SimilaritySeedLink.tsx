import { ConfigProvider } from "antd";
import type { SimilaritySeed } from "../../generated/atlas";
import { seedNavigation } from "./similaritySource";
import { AtlasLink } from "../../app/AtlasLink";
import { recordDestination } from "../../app/destinations";
import { isPrimaryActivation } from "../../app/navigation";
import { useReferenceRecordNavigation } from "../records/ReferenceHelp";

/** The owning Search response supplies the criterion's reading capability. */
export function SimilaritySeedLink({
  seed,
  title,
  disabled,
}: {
  seed: SimilaritySeed;
  title: string;
  disabled?: boolean;
}) {
  const onReference = useReferenceRecordNavigation();
  const { componentDisabled } = ConfigProvider.useConfig();
  const unavailable = disabled ?? componentDisabled;
  const selection = seed.embedded_path?.length ? seedNavigation(seed) : undefined;
  return (
    <AtlasLink
      {...recordDestination({ recordKey: seed.record_key, selection })}
      disabled={unavailable}
      onClick={(event) => {
        if (unavailable || !onReference || !isPrimaryActivation(event)) return;
        event.preventDefault();
        onReference(seed.record_key, event.currentTarget, selection);
      }}
    >
      {title}
    </AtlasLink>
  );
}
