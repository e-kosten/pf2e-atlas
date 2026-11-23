import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Button, Typography } from "antd";
import { useSavedLists } from "./savedListQueries";
import { listDestination } from "../../app/destinations";
import { destinationHref } from "../../app/navigation";

export function CheckListDirectory({ onChecked }: { onChecked: () => void }) {
  const client = useQueryClient();
  const lists = useSavedLists({ enabled: false, includeArchived: true });
  const [checked, setChecked] = useState(false);
  async function check() {
    const result = await lists.refetch();
    if (result.isSuccess) {
      await client.invalidateQueries({ queryKey: ["saved-lists"], exact: true });
      setChecked(true);
      onChecked();
    }
  }
  return (
    <div>
      <Button size="small" loading={lists.isFetching} onClick={() => void check()}>
        Check list directory
      </Button>
      {lists.isError && <p role="status">The list directory could not load.</p>}
      {checked && lists.data && (
        <div
          aria-label="List directory"
          role="region"
          className="list-create-directory"
        >
          {lists.data.lists.length ? (
            <ul>
              {lists.data.lists.map((list) => (
                <li key={list.list_key}>
                  <Typography.Link
                    href={destinationHref(
                      listDestination({ slug: list.slug, selectedRecordKey: null }),
                    )}
                    target="_blank"
                    rel="noopener"
                  >
                    {list.name} ({list.slug})
                  </Typography.Link>
                </li>
              ))}
            </ul>
          ) : (
            <p>No saved lists.</p>
          )}
        </div>
      )}
    </div>
  );
}
