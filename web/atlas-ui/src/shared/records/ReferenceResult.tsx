import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { Button, Popover, Typography } from "antd";
import { useEffect, useRef, type ReactNode } from "react";
import type {
  RelationshipClauseSummaryView,
  RecordNavigationView,
  SearchResultRow,
} from "../../generated/atlas";
import { RelationshipEvidence } from "./RelationshipEvidence";
import { RecordBrowseSummary } from "./RecordBrowseSummary";
export function ReferenceResult({
  row,
  selected,
  disabled,
  onPreview,
  onOpenPage,
  matchNavigation,
  relationshipClauses,
  filter,
  generation,
  onRefreshResults,
  actions,
  context,
  qualifiers,
  searchCard,
  navigation,
  previewOnEnter,
}: {
  row: SearchResultRow;
  actions?: ReactNode;
  context?: ReactNode;
  qualifiers?: string[];
  searchCard?: boolean;
  navigation?: RecordNavigationView;
  previewOnEnter?: boolean;
  relationshipClauses?: RelationshipClauseSummaryView[];
  filter?: import("../../generated/atlas").QueryPredicate | null;
  generation?: string;
  onRefreshResults?: () => Promise<void>;
  selected: boolean;
  disabled: boolean;
  onPreview: (anchor: HTMLElement) => void;
  onOpenPage?: (navigation: RecordNavigationView) => void;
  matchNavigation?: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onNavigate: (navigation: RecordNavigationView, anchor: HTMLElement) => void;
  };
}) {
  const matchesOpen = matchNavigation?.open ?? false;
  const setMatchesOpen = matchNavigation?.onOpenChange;
  const trigger = useRef<HTMLButtonElement>(null);
  const matches = row.matching_sections ?? [];
  const relationshipEvidence =
    generation && filter
      ? (row.relationship_evidence ?? []).filter((evidence) =>
          relationshipClauses?.some(
            (clause) => clause.clause_id === evidence.clause_id,
          ),
        )
      : [];
  const matchCount = matchNavigation
    ? BigInt(matches.length) +
      relationshipEvidence.reduce(
        (count, evidence) => count + BigInt(evidence.page.total),
        0n,
      )
    : 0n;
  useEffect(() => {
    if (!matchesOpen) return;
    const escape = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      setMatchesOpen?.(false);
      trigger.current?.focus({ preventScroll: true });
    };
    document.addEventListener("keydown", escape, true);
    return () => document.removeEventListener("keydown", escape, true);
  }, [matchesOpen, setMatchesOpen]);
  const record = row.record;
  return (
    <div className={`reference-result${selected ? " reference-result--selected" : ""}`}>
      <RecordBrowseSummary
        record={record}
        selected={selected}
        disabled={disabled}
        onPreview={onPreview}
        context={context}
        qualifiers={qualifiers}
        searchCard={searchCard}
        previewOnEnter={previewOnEnter}
        href={
          navigation
            ? destinationHref(
                recordDestination({
                  recordKey: navigation.record_key,
                  selection: navigation,
                }),
              )
            : undefined
        }
      />
      {onOpenPage && row.exact_journal_pages && (
        <div className="reference-result__journal-pages">
          {row.exact_journal_pages.pages.map(({ title, navigation: page }) => (
            <Typography.Link
              key={JSON.stringify(page.embedded_path)}
              disabled={disabled}
              href={destinationHref(
                recordDestination({
                  recordKey: record.record_key,
                  selection: page,
                }),
              )}
              onClick={(event) => {
                if (!isPrimaryActivation(event)) return;
                event.preventDefault();
                onOpenPage(page);
              }}
            >
              {record.title} › {title}
            </Typography.Link>
          ))}
          {row.exact_journal_pages.additional_count > 0 && (
            <span>
              +{row.exact_journal_pages.additional_count} more in this journal
            </span>
          )}
        </div>
      )}
      {(matchCount > 0n || actions) && (
        <div className="reference-result__actions">
          {matchCount > 0n && (
            <Popover
              open={matchesOpen}
              onOpenChange={setMatchesOpen}
              trigger="click"
              placement="bottomLeft"
              destroyOnHidden
              content={
                <div
                  className="reference-match-list"
                  ref={(node) => {
                    node
                      ?.querySelector<HTMLAnchorElement>("a")
                      ?.focus({ preventScroll: true });
                  }}
                >
                  {matches.map((match) => (
                    <Button
                      key={JSON.stringify(match.navigation)}
                      color="default"
                      variant="text"
                      block
                      styles={{
                        root: {
                          height: "auto",
                          whiteSpace: "normal",
                          justifyContent: "flex-start",
                        },
                      }}
                      href={destinationHref(
                        recordDestination({
                          recordKey: match.navigation.record_key,
                          selection: match.navigation,
                        }),
                      )}
                      onClick={(event) => {
                        if (!isPrimaryActivation(event)) return;
                        event.preventDefault();
                        setMatchesOpen?.(false);
                        matchNavigation!.onNavigate(
                          match.navigation,
                          trigger.current ?? event.currentTarget,
                        );
                      }}
                    >
                      {match.label}
                    </Button>
                  ))}
                  {generation &&
                    relationshipEvidence.map((evidence) => {
                      const summary = relationshipClauses!.find(
                        (clause) => clause.clause_id === evidence.clause_id,
                      )!;
                      return (
                        <RelationshipEvidence
                          key={JSON.stringify([
                            generation,
                            summary.relationship,
                            row.record.record_key,
                          ])}
                          evidence={evidence}
                          filter={filter!}
                          summary={summary}
                          resultKey={row.record.record_key}
                          generation={generation}
                          onRefresh={onRefreshResults}
                          onNavigate={(navigation, anchor) => {
                            setMatchesOpen?.(false);
                            matchNavigation!.onNavigate(
                              navigation,
                              trigger.current ?? anchor,
                            );
                          }}
                        />
                      );
                    })}
                </div>
              }
            >
              <Button
                ref={trigger}
                className="reference-result__matches"
                color="default"
                variant="text"
                size="small"
                aria-expanded={matchesOpen}
                aria-label={`${matchCount.toString()} matches in ${record.title}`}
                styles={{
                  root: {
                    color: "var(--muted)",
                    outlineColor: "var(--text)",
                  },
                }}
                disabled={disabled}
              >
                Matches
              </Button>
            </Popover>
          )}{" "}
          {actions}
        </div>
      )}
    </div>
  );
}
