import { Badge, Group, ScrollArea, Table } from "@mantine/core";
import { handleResultKeyboard, useActiveResultScroll } from "../resultKeyboard";
import type { AtlasWorkspaceState } from "../useAtlasWorkspace";

export function MantineResults({ workspace }: { workspace: AtlasWorkspaceState }) {
  const scrollRef = useActiveResultScroll<HTMLDivElement>(workspace.activeResultKey);

  return (
    <section className="results-panel">
      <ScrollArea
        aria-label="Results"
        className="results-scroll--focusable"
        onKeyDown={(event) => handleResultKeyboard(event, workspace)}
        ref={scrollRef}
        tabIndex={0}
      >
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
            {(workspace.resultPage?.rows ?? []).map((row) => (
              <Table.Tr
                className={
                  row.record.record_key === workspace.activeResultKey
                    ? "result-row result-row--active"
                    : "result-row"
                }
                key={row.record.record_key}
                data-active-result={
                  row.record.record_key === workspace.activeResultKey
                    ? "true"
                    : undefined
                }
                onMouseEnter={() => workspace.focusResult(row.record.record_key)}
              >
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
  );
}
