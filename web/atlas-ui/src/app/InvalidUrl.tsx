import { useRouter, useRouterState } from "@tanstack/react-router";
import { Alert, Button, Space } from "antd";
import { usePageTitle } from "../shared/browser/usePageTitle";
import { DocumentPagePane } from "../shared/records/DocumentPagePane";
import { recordDestination, searchDestination, listDestination } from "./destinations";
import { destinationHref, isPrimaryActivation } from "./navigation";
import { ReadingUrlError } from "./readingUrl";
import { parseSearchParams, stringifySearchParams } from "./urlState";
import { canonicalRouteSearch } from "./routeSearch";

function RecoveryLink({ href, children }: { href: string; children: React.ReactNode }) {
  const router = useRouter();
  return (
    <Button
      href={href}
      onClick={(event) => {
        if (!isPrimaryActivation(event)) return;
        event.preventDefault();
        void router.navigate({ href, replace: true }).then(() => {
          requestAnimationFrame(() => {
            document.getElementById("atlas-main")?.focus({ preventScroll: true });
          });
        });
      }}
    >
      {children}
    </Button>
  );
}

export function InvalidUrl({ error }: { error: unknown }) {
  usePageTitle("Invalid link");
  const location = useRouterState({ select: (state) => state.location });
  const match = useRouterState({
    select: (state) =>
      [...state.matches].reverse().find((entry) => "recordKey" in entry.params),
  });
  const cause =
    error instanceof Error && "cause" in error && error.cause instanceof Error
      ? error.cause
      : error;
  const recordKey =
    match && "recordKey" in match.params ? String(match.params.recordKey) : undefined;
  let resultsHref: string | undefined;
  if (cause instanceof ReadingUrlError && recordKey) {
    const search = parseSearchParams(location.searchStr);
    for (const key of [
      "rank",
      "at",
      "focus",
      "selection",
      "landing",
      "record_view",
      "record_tab",
    ])
      delete search[key];
    try {
      if (match?.staticData.section === "search")
        resultsHref = destinationHref(
          searchDestination({
            query: stringifySearchParams(canonicalRouteSearch("search", search)),
          }),
        );
      else if (match && "slug" in match.params)
        resultsHref = destinationHref(
          listDestination({
            slug: String(match.params.slug),
            selectedRecordKey: null,
            query: stringifySearchParams(canonicalRouteSearch("list", search)),
          }),
        );
    } catch {
      // Independent invalid criteria cannot be presented as a restored results view.
    }
  }
  return (
    <main className="record-view">
      <DocumentPagePane>
        <Alert
          type="error"
          showIcon
          title="This link couldn’t be restored"
          description={
            <Space orientation="vertical" size="middle" style={{ width: "100%" }}>
              <span>
                {cause instanceof Error ? cause.message : "Invalid navigation URL"}
              </span>
              <Space wrap role="group" aria-label="Link recovery">
                {recordKey && (
                  <RecoveryLink
                    href={destinationHref(recordDestination({ recordKey }))}
                  >
                    Open record
                  </RecoveryLink>
                )}
                {resultsHref && (
                  <RecoveryLink href={resultsHref}>Return to results</RecoveryLink>
                )}
                <RecoveryLink href={destinationHref(searchDestination())}>
                  Open Search
                </RecoveryLink>
              </Space>
            </Space>
          }
        />
      </DocumentPagePane>
    </main>
  );
}

export function LinkNotFound() {
  usePageTitle("Link not found");
  return (
    <main className="record-view">
      <DocumentPagePane>
        <Alert
          type="warning"
          showIcon
          title="Link not found"
          action={
            <RecoveryLink href={destinationHref(searchDestination())}>
              Open Search
            </RecoveryLink>
          }
        />
      </DocumentPagePane>
    </main>
  );
}
