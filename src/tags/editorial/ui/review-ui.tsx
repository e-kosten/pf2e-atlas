import React from "react";

import {
  TerminalPaneScreen,
  TerminalTwoPaneScreen,
  runDerivedTagTerminalApp,
  useDerivedTagTerminalApp,
} from "../../../tui/terminal-ui.js";
import {
  DEFAULT_DERIVED_TAG_REVIEW_SERVICES,
  type DerivedTagReviewServices,
} from "./review-controller.js";
import { useDerivedTagReviewScreenController } from "./review-ui-controller.js";
import type { DerivedTagReviewSession } from "../types.js";

export type DerivedTagReviewResult = {
  imported: boolean;
  session: DerivedTagReviewSession;
};

export function DerivedTagReviewScreen({
  rootPath,
  initialSession,
  onComplete,
  services = DEFAULT_DERIVED_TAG_REVIEW_SERVICES,
}: {
  rootPath: string;
  initialSession: DerivedTagReviewSession;
  onComplete: (result: DerivedTagReviewResult) => void;
  services?: DerivedTagReviewServices;
}): React.JSX.Element {
  const controller = useDerivedTagReviewScreenController({
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
  initialSession: DerivedTagReviewSession;
  onComplete: (result: DerivedTagReviewResult) => void;
}): React.JSX.Element {
  return (
    <DerivedTagReviewScreen rootPath={rootPath} initialSession={initialSession} onComplete={onComplete} />
  );
}

export async function runDerivedTagReviewUi(
  rootPath: string,
  initialSession: DerivedTagReviewSession,
): Promise<DerivedTagReviewResult> {
  let result: DerivedTagReviewResult | undefined;

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
  initialSession: DerivedTagReviewSession;
  onComplete: (result: DerivedTagReviewResult) => void;
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
