import { Badge, Button, Group, ScrollArea, Table } from "@mantine/core";
import { ChevronLeft, ChevronRight } from "lucide-react";
import type { AtlasWorkspaceState } from "../useAtlasWorkspace";

export function MantineResults({
  workspace,
}: {
  workspace: AtlasWorkspaceState;
}) {
  return (
    <section className="results-panel">
      <div className="panel-heading">
        <div>
          <h2>Results</h2>
          <p>
            {workspace.resultPage
              ? `${workspace.resultPage.page.total.toLocaleString()} records`
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
            {(workspace.resultPage?.rows ?? []).map((row) => (
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
        onClick={() =>
          workspace.setPageNumber(page?.next_page ?? workspace.pageNumber + 1)
        }
        variant="subtle"
      />
    </Group>
  );
}
