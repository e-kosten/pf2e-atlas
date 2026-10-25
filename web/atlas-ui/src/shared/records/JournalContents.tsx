import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { Input, Typography } from "antd";
import { useState } from "react";
import type { EmbeddedRecordLinkView, FactView } from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";
import "../../styles/ui/journal-reading.css";
export function JournalContents({
  pages,
  onReference,
}: {
  pages: FactView<EmbeddedRecordLinkView[]>;
  onReference: RecordReferenceHandler;
}) {
  const [query, setQuery] = useState("");
  if (!pages.value?.length) return null;
  const needle = query.trim().toLocaleLowerCase();
  const visible = pages.value.filter((page) =>
    page.title.toLocaleLowerCase().includes(needle),
  );
  return (
    <section className="journal-contents" aria-label="Journal contents">
      <h3>Contents</h3>
      {(pages.value.length > 20 || query) && (
        <Input
          aria-label="Find a page"
          placeholder="Find a page…"
          value={query}
          allowClear
          onChange={(event) => setQuery(event.target.value)}
        />
      )}
      {needle && (
        <p className="muted" role="status">
          {visible.length} of {pages.value.length} pages
        </p>
      )}
      <ul className="journal-contents__pages">
        {visible.map((page) => (
          <li key={JSON.stringify(page.navigation.embedded_path)}>
            <Typography.Link
              href={destinationHref(
                recordDestination({
                  recordKey: page.navigation.record_key,
                  selection: page.navigation,
                }),
              )}
              onClick={(event) => {
                if (!isPrimaryActivation(event)) return;
                event.preventDefault();
                onReference(
                  page.navigation.record_key,
                  event.currentTarget,
                  page.navigation,
                );
              }}
            >
              {page.title}
            </Typography.Link>
          </li>
        ))}
      </ul>
      {!visible.length && <p>No matching pages.</p>}
    </section>
  );
}
