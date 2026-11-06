import { Button, ConfigProvider } from "antd";
import type { ReactNode } from "react";
import type { SimilaritySeed } from "../../generated/atlas";
import { AtlasLink } from "../../app/AtlasLink";
import { recordDestination } from "../../app/destinations";
import { isPrimaryActivation } from "../../app/navigation";
import { useDefinitionDetail } from "../records/useDefinitionDetail";
import { isArtifactGenerationChanged } from "../queries/referenceQueries";
import { ErrorNotice } from "../feedback/ErrorNotice";
import {
  ReferenceHelpLink,
  useReferenceHelpGeneration,
  useReferenceRecordNavigation,
} from "../records/ReferenceHelp";
import { ReferenceHelpHost } from "../records/ReferenceHelpHost";

type SeedLinkProps = {
  seed: SimilaritySeed;
  title: string;
  disabled?: boolean;
  generation?: string;
  onRefresh?: () => Promise<void>;
  renderLayout?: (link: ReactNode, feedback: ReactNode) => ReactNode;
};

/** A criterion references its source without changing the selected search result. */
export function SimilaritySeedLink(props: SeedLinkProps) {
  return (
    <ReferenceHelpHost>
      <SeedLink {...props} />
    </ReferenceHelpHost>
  );
}

function SeedLink({
  seed,
  title,
  disabled,
  generation: expectedGeneration,
  onRefresh,
  renderLayout,
}: SeedLinkProps) {
  const onReference = useReferenceRecordNavigation();
  const inheritedGeneration = useReferenceHelpGeneration();
  const generation = expectedGeneration ?? inheritedGeneration;
  const { componentDisabled } = ConfigProvider.useConfig();
  const unavailable = disabled ?? componentDisabled;
  const definition = useDefinitionDetail(
    seed.kind === "trait" && !unavailable ? seed.trait_key : null,
    generation,
  );
  const changed = isArtifactGenerationChanged(definition.error);
  const refreshOwner = changed && (generation !== undefined || onRefresh !== undefined);
  const link =
    seed.kind === "trait" ? (
      unavailable ? (
        title
      ) : (
        <ReferenceHelpLink
          help={changed ? undefined : definition.data?.help}
          generation={definition.data?.artifact_generation}
        >
          {title}
        </ReferenceHelpLink>
      )
    ) : (
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
  const recovery = (
    <Button
      disabled={refreshOwner && !onRefresh}
      onClick={() => (refreshOwner ? void onRefresh?.() : void definition.refetch())}
    >
      {refreshOwner ? "Refresh filters" : "Retry"}
    </Button>
  );
  const feedback =
    seed.kind === "trait" && !unavailable && definition.error ? (
      <ErrorNotice
        compact
        error={definition.error}
        title="Definition unavailable"
        action={recovery}
      />
    ) : null;
  return renderLayout ? (
    renderLayout(link, feedback)
  ) : (
    <>
      {link}
      {feedback}
    </>
  );
}
