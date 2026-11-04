import { Checkbox, Typography } from "antd";
import type { SimilarityComponentView, SimilaritySeed } from "../../generated/atlas";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination, traitDestination } from "../../app/destinations";

export function SimilaritySectionChoices({
  parts,
  selected,
  onChange,
  onReference,
  seed,
}: {
  seed: SimilaritySeed;
  onReference: (part: SimilarityComponentView, opener: HTMLElement) => void;
  parts: SimilarityComponentView[];
  selected: string[] | null;
  onChange: (parts: string[] | null) => void;
}) {
  const groups = new Map<string, { label: string; parts: SimilarityComponentView[] }>();
  for (const part of parts) {
    const key = part.kind === "identity" ? "identity" : JSON.stringify(part.owners);
    const group = groups.get(key);
    if (group) group.parts.push(part);
    else groups.set(key, { label: part.owner_label || "Main entry", parts: [part] });
  }
  const available = parts.filter((part) => part.has_vectors);
  const chosen = new Set(selected ?? available.map((part) => part.selector));
  function toggle(selectors: string[], checked: boolean) {
    const allowed = new Set(available.map((part) => part.selector));
    const next = new Set([...chosen].filter((selector) => allowed.has(selector)));
    for (const selector of selectors)
      if (checked) next.add(selector);
      else next.delete(selector);
    onChange([...next]);
  }
  return (
    <div className="search-settings__parts">
      {[...groups].map(([key, { label: owner, parts: entries }]) => {
        const selectors = entries
          .filter((part) => part.has_vectors)
          .map((part) => part.selector);
        const count = selectors.filter((selector) => chosen.has(selector)).length;
        const singleSection = entries.length === 1;
        return (
          <div key={key} className="search-settings__part-group">
            {!singleSection && (
              <div className="search-settings__part-choice">
                <Checkbox
                  aria-label={`All sections in ${owner}`}
                  checked={selectors.length > 0 && count === selectors.length}
                  indeterminate={count > 0 && count < selectors.length}
                  disabled={!selectors.length}
                  onChange={(event) => toggle(selectors, event.target.checked)}
                />
                <strong>{owner}</strong>
              </div>
            )}
            <div className={singleSection ? undefined : "search-settings__part-fields"}>
              {entries.map((part) => (
                <div key={part.selector} className="search-settings__part-choice">
                  <Checkbox
                    aria-label={singleSection ? `${owner}: ${part.label}` : part.label}
                    checked={chosen.has(part.selector)}
                    disabled={!part.has_vectors}
                    onChange={(event) => toggle([part.selector], event.target.checked)}
                  />
                  <span>
                    <Typography.Link
                      strong={singleSection}
                      href={destinationHref(
                        seed.kind === "trait"
                          ? traitDestination({ traitKey: seed.trait_key })
                          : recordDestination({
                              recordKey: seed.record_key,
                              selection: part.navigation ?? undefined,
                            }),
                      )}
                      onClick={(event) => {
                        if (!isPrimaryActivation(event)) return;
                        event.preventDefault();
                        // Touch activation need not focus the link before this handler.
                        event.currentTarget.focus({ preventScroll: true });
                        onReference(part, event.currentTarget);
                      }}
                    >
                      {part.kind === "identity"
                        ? part.label
                        : singleSection
                          ? owner
                          : part.label}
                    </Typography.Link>
                    {singleSection && part.kind === "section" && (
                      <>
                        {" "}
                        <Typography.Text type="secondary">{part.label}</Typography.Text>
                      </>
                    )}
                    {!part.has_vectors ? " (no stored vector)" : ""}
                  </span>
                </div>
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}
