import { useState } from "react";
import { Button, Space, Typography } from "antd";
import type { ReferenceFilterActionView } from "../../generated/atlas";
import { searchDestination } from "../../app/destinations";
import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";
import { DEFAULT_SEARCH_STATE, searchStateQueryString } from "./searchState";

export type ReferenceFilterActionHandler = (
  action: ReferenceFilterActionView,
) => string | null;

/** One mechanical handoff for Rust-authored actions in reading and discovery. */
export function ReferenceFilterActions({
  actions,
  generation,
  onAction,
}: {
  actions: ReferenceFilterActionView[];
  generation?: string;
  onAction?: ReferenceFilterActionHandler;
}) {
  const [error, setError] = useState<string | null>(null);
  return (
    <Space
      orientation="vertical"
      align="start"
      style={{ maxWidth: "100%" }}
      styles={{ item: { maxWidth: "100%" } }}
    >
      {actions.map((action, index) => {
        const route =
          action.kind === "search"
            ? searchDestination({
                query: searchStateQueryString(
                  { ...DEFAULT_SEARCH_STATE, filter: action.predicate },
                  new URLSearchParams(window.location.search).get("search_profile"),
                ),
              })
            : searchDestination({ configureField: action.field.id });
        return (
          <Button
            styles={{
              root: {
                whiteSpace: "normal",
                height: "auto",
                textAlign: "start",
                overflowWrap: "anywhere",
                maxWidth: "100%",
              },
            }}
            key={index}
            href={onAction ? undefined : destinationHref(route)}
            onClick={(event) => {
              event.stopPropagation();
              if (!onAction && !isPrimaryActivation(event)) return;
              event.preventDefault();
              if (
                action.kind === "search" &&
                generation &&
                action.artifact_generation !== generation
              ) {
                setError(
                  "The reference collection changed. Refresh before using this filter.",
                );
                return;
              }
              const failure = onAction?.(action) ?? null;
              setError(failure);
              if (!onAction)
                navigateDestination(route, { dismissReadingOverlays: true });
            }}
          >
            {action.label}
          </Button>
        );
      })}
      {error && (
        <Typography.Text type="danger" role="status">
          {error}
        </Typography.Text>
      )}
    </Space>
  );
}
