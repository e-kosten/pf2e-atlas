import { Button, Select } from "antd";
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
  filterState,
  list,
  lists,
  listsLoading,
  loading,
  onSelectList,
}: {
  currentSlug: string;
  filterState: FilterPanelState;
  list: SavedListSummaryView | undefined;
  lists: SavedListSummaryView[];
  listsLoading: boolean;
  loading: boolean;
  onSelectList: (slug: string) => void;
}) {
  if (loading) {
    return <div className="list-info-pane">Loading list</div>;
  }
  if (!list) {
    return <div className="list-info-pane">List not found</div>;
  }
  const listOptions = lists.some((listOption) => listOption.slug === list.slug)
    ? lists
    : [list, ...lists];
  return (
    <section className="list-info-pane">
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
      <FilterControls
        filterState={filterState}
        includeResultOptions={false}
        includeSearch
      />
    </section>
  );
}
