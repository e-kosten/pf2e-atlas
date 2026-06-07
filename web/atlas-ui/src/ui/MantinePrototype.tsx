import {
  Badge,
  Button,
  Checkbox,
  Group,
  MultiSelect,
  NumberInput,
  ScrollArea,
  Select,
  Table,
  TextInput,
} from "@mantine/core";
import { ChevronLeft, ChevronRight, Search } from "lucide-react";
import {
  KIND_OPTIONS,
  RARITY_OPTIONS,
  SORT_OPTIONS,
  TRAIT_OPTIONS,
  type SortKey,
} from "../state/searchState";
import { RecordPresentation } from "./recordPresentation";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";

export function MantinePrototype({
  workspace,
}: {
  workspace: AtlasWorkspaceState;
}) {
  const { search, setSearch, resultPage } = workspace;

  return (
    <main className="workspace-grid workspace-grid--mantine">
      <aside className="filter-panel">
        <TextInput
          leftSection={<Search size={16} />}
          placeholder="Search records"
          value={search.query}
          onChange={(event) =>
            setSearch({
              ...search,
              query: event.currentTarget.value,
              mode: event.currentTarget.value.trim()
                ? "text_search"
                : "browse",
            })
          }
        />
        <MultiSelect
          data={KIND_OPTIONS}
          label="Kinds"
          searchable
          value={search.kinds}
          onChange={(kinds) => setSearch({ ...search, kinds })}
        />
        <MultiSelect
          data={RARITY_OPTIONS}
          label="Rarity"
          value={search.rarity}
          onChange={(rarity) => setSearch({ ...search, rarity })}
        />
        <MultiSelect
          data={TRAIT_OPTIONS}
          label="Traits"
          searchable
          value={search.traits}
          onChange={(traits) => setSearch({ ...search, traits })}
        />
        <Checkbox
          checked={search.traitOperator === "include_any"}
          label="Match any selected trait"
          onChange={(event) =>
            setSearch({
              ...search,
              traitOperator: event.currentTarget.checked
                ? "include_any"
                : "include_all",
            })
          }
        />
        <MultiSelect
          data={TRAIT_OPTIONS}
          label="Excluded traits"
          searchable
          value={search.excludedTraits}
          onChange={(excludedTraits) =>
            setSearch({ ...search, excludedTraits })
          }
        />
        <Group grow align="end">
          <NumberInput
            label="Min level"
            min={0}
            max={30}
            value={search.levelMin ?? ""}
            onChange={(levelMin) =>
              setSearch({
                ...search,
                levelMin:
                  typeof levelMin === "number" ? levelMin : null,
              })
            }
          />
          <NumberInput
            label="Max level"
            min={0}
            max={30}
            value={search.levelMax ?? ""}
            onChange={(levelMax) =>
              setSearch({
                ...search,
                levelMax:
                  typeof levelMax === "number" ? levelMax : null,
              })
            }
          />
        </Group>
        <Group grow align="end">
          <Select
            data={SORT_OPTIONS}
            label="Sort"
            value={search.sort}
            onChange={(sort) =>
              setSearch({ ...search, sort: (sort ?? "record_key") as SortKey })
            }
          />
          <NumberInput
            label="Page size"
            min={10}
            max={100}
            step={5}
            value={search.pageSize}
            onChange={(pageSize) =>
              setSearch({
                ...search,
                pageSize: typeof pageSize === "number" ? pageSize : 25,
              })
            }
          />
        </Group>
      </aside>
      <section className="results-panel">
        <div className="panel-heading">
          <div>
            <h2>Results</h2>
            <p>
              {resultPage
                ? `${resultPage.page.total.toLocaleString()} records`
                : "No result window yet"}
            </p>
          </div>
          <PaginationControls workspace={workspace} />
        </div>
        <ScrollArea>
          <Table highlightOnHover stickyHeader>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>Record</Table.Th>
                <Table.Th>Kind</Table.Th>
                <Table.Th>Level</Table.Th>
                <Table.Th>Traits</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {(resultPage?.rows ?? []).map((row) => (
                <Table.Tr key={row.record.record_key}>
                  <Table.Td>
                    <button
                      className="row-link"
                      onClick={() => workspace.selectRecord(row.record.record_key)}
                      type="button"
                    >
                      <span>{row.record.title}</span>
                      <small>{row.record.record_key}</small>
                    </button>
                  </Table.Td>
                  <Table.Td>{row.record.kind_label}</Table.Td>
                  <Table.Td>{row.record.level_label ?? ""}</Table.Td>
                  <Table.Td>
                    <Group gap={4}>
                      {(row.record.traits ?? []).slice(0, 4).map((trait) => (
                        <Badge key={trait.value} variant="light">
                          {trait.label}
                        </Badge>
                      ))}
                    </Group>
                  </Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        </ScrollArea>
      </section>
      <section className="detail-panel">
        <RecordPresentation
          detail={workspace.recordDetail}
          loading={workspace.detailLoading}
          onReference={workspace.selectRecord}
        />
      </section>
    </main>
  );
}

function PaginationControls({ workspace }: { workspace: AtlasWorkspaceState }) {
  const page = workspace.resultPage?.page;
  return (
    <Group gap={6} className="pager">
      <Button
        disabled={workspace.pageNumber <= 1}
        leftSection={<ChevronLeft size={16} />}
        onClick={() => workspace.setPageNumber(workspace.pageNumber - 1)}
        variant="subtle"
      />
      <span>{page ? `${page.number}` : workspace.pageNumber}</span>
      <Button
        disabled={!page?.has_more}
        rightSection={<ChevronRight size={16} />}
        onClick={() => workspace.setPageNumber(page?.next_page ?? workspace.pageNumber + 1)}
        variant="subtle"
      />
    </Group>
  );
}
