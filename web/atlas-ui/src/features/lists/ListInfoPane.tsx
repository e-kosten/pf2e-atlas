import { Alert, Button, Select, Spin } from "antd";
import { Pencil } from "lucide-react";
import type { SavedListSummaryView } from "../../generated/atlas";
import {
  listEditPath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import { FilterControls } from "../../shared/filters/FilterPanel";
import type { FilterPanelState } from "../../shared/filters/filterControls";

export function ListInfoPane({
  currentSlug,
  error,
  filterState,
  list,
  lists,
  listsLoading,
  loading,
  onSelectList,
  onRetry,
}: {
  currentSlug: string;
  error?: Error | null;
  filterState: FilterPanelState;
  list: SavedListSummaryView | undefined;
  lists: SavedListSummaryView[];
  listsLoading: boolean;
  loading: boolean;
  onSelectList: (slug: string) => void;
  onRetry: () => void;
}) {
  const errorNotice = error ? (
    <Alert
      type="error"
      showIcon
      title={list ? "Unable to refresh this list" : "Unable to load this list"}
      description={error.message}
      action={<Button onClick={onRetry}>Retry list</Button>}
    />
  ) : null;
  if (loading) {
    return (
      <section className="list-info-pane" role="status">
        <Spin size="small" aria-label="Loading list" /> Loading list…
      </section>
    );
  }
  if (!list) {
    return (
      <section className="list-info-pane">{errorNotice ?? "List not found"}</section>
    );
  }
  const listOptions = lists.some((listOption) => listOption.slug === list.slug)
    ? lists
    : [list, ...lists];
  return (
    <section className="list-info-pane">
      {errorNotice}
      <Select
        aria-label="Selected list"
        className="list-info-pane__select"
        loading={listsLoading}
        onChange={onSelectList}
        options={listOptions.map((listOption) => ({
          label: listOption.name,
          value: listOption.slug,
        }))}
        value={currentSlug}
      />
      <div>
        <h2>{list.name}</h2>
      </div>
      {list.description && (
        <p className="list-info-pane__description">{list.description}</p>
      )}
      <Button
        href={listEditPath(list.slug)}
        icon={<Pencil size={16} />}
        onClick={(event) => {
          if (!shouldHandleAtlasRouteClick(event)) {
            return;
          }
          event.preventDefault();
          navigateToAtlasRoute({ kind: "listEdit", slug: list.slug });
        }}
      >
        Edit
      </Button>
      <FilterControls filterState={filterState} includeSearch />
    </section>
  );
}
