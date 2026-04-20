import React from "react";

import {
  TerminalPaneScreen,
  TerminalTwoPaneScreen,
  runDerivedTagTerminalApp,
  useDerivedTagTerminalApp,
} from "../../../tui/terminal-ui.js";
import {
  DEFAULT_DERIVED_TAG_MIGRATION_REVIEW_SERVICES,
  type DerivedTagMigrationReviewServices,
} from "./review-controller.js";
import { useDerivedTagMigrationReviewScreenController } from "./review-ui-controller.js";
import type { DerivedTagMigrationSession } from "../types.js";

export type DerivedTagMigrationReviewResult = {
  imported: boolean;
  session: DerivedTagMigrationSession;
};

export function DerivedTagMigrationReviewScreen({
  rootPath,
  initialSession,
  onComplete,
  services = DEFAULT_DERIVED_TAG_MIGRATION_REVIEW_SERVICES,
}: {
  rootPath: string;
  initialSession: DerivedTagMigrationSession;
  onComplete: (result: DerivedTagMigrationReviewResult) => void;
  services?: DerivedTagMigrationReviewServices;
}): React.JSX.Element {
  const controller = useDerivedTagMigrationReviewScreenController({
    rootPath,
    initialSession,
    onComplete,
    services,
  });

  if (controller.screen.kind === "detail-only") {
    return <TerminalPaneScreen {...controller.screen.props} />;
  }

  return <TerminalTwoPaneScreen {...controller.screen.props} />;
}

function DerivedTagMigrationReviewRoot({
  rootPath,
  initialSession,
  onComplete,
}: {
  rootPath: string;
  initialSession: DerivedTagMigrationSession;
  onComplete: (result: DerivedTagMigrationReviewResult) => void;
}): React.JSX.Element {
  return (
    <DerivedTagMigrationReviewScreen rootPath={rootPath} initialSession={initialSession} onComplete={onComplete} />
  );
}

export async function runDerivedTagMigrationReviewUi(
  rootPath: string,
  initialSession: DerivedTagMigrationSession,
): Promise<DerivedTagMigrationReviewResult> {
  let result: DerivedTagMigrationReviewResult | undefined;

  await runDerivedTagTerminalApp(
    <ReviewRunner
      rootPath={rootPath}
      initialSession={initialSession}
      onComplete={(nextResult) => {
        result = nextResult;
      }}
    />,
  );

  if (result === undefined) {
    throw new Error("Review did not complete.");
  }
  return result;
}

function ReviewRunner({
  rootPath,
  initialSession,
  onComplete,
}: {
  rootPath: string;
  initialSession: DerivedTagMigrationSession;
  onComplete: (result: DerivedTagMigrationReviewResult) => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  return (
    <DerivedTagMigrationReviewRoot
      rootPath={rootPath}
      initialSession={initialSession}
      onComplete={(result) => {
        onComplete(result);
        terminal.exitApp(result);
      }}
    />
  );
}
