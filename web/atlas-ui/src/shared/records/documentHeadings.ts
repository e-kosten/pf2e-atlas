/** Presentation of already prepared headings; never interprets source rules. */
export function documentHeadings(html: string, identity: string) {
  const parsed = new DOMParser().parseFromString(html, "text/html");
  const headings = [...parsed.body.querySelectorAll<HTMLElement>("h1,h2,h3,h4,h5,h6")];
  const firstLevel = Math.min(
    ...headings.map((heading) => Number(heading.tagName.slice(1))),
  );
  const prefix = `atlas-section-${Array.from(identity).reduce((hash, char) => Math.imul(hash ^ char.charCodeAt(0), 16777619), 2166136261) >>> 0}`;
  const outline: { id: string; label: string }[] = [];
  const occurrences = new Map<string, number>();
  headings.forEach((heading) => {
    const sourceLevel = Number(heading.tagName.slice(1));
    const replacement = parsed.createElement(
      `h${Math.min(6, sourceLevel - firstLevel + 3)}`,
    );
    for (const attribute of [...heading.attributes])
      replacement.setAttribute(attribute.name, attribute.value);
    replacement.classList.add("document-heading");
    const slug =
      (heading.textContent ?? "")
        .normalize("NFKC")
        .toLocaleLowerCase()
        .replace(/[^\p{L}\p{N}]+/gu, "-")
        .replace(/^-|-$/g, "") || "section";
    const occurrence = (occurrences.get(slug) ?? 0) + 1;
    occurrences.set(slug, occurrence);
    replacement.id =
      heading.id || `${prefix}-${slug}${occurrence > 1 ? `-${occurrence}` : ""}`;
    replacement.tabIndex = -1;
    replacement.append(...heading.childNodes);
    if (sourceLevel === firstLevel && replacement.textContent?.trim())
      outline.push({ id: replacement.id, label: replacement.textContent.trim() });
    heading.replaceWith(replacement);
  });
  return {
    html: parsed.body.innerHTML,
    outline: outline.length >= 3 && outline.length <= 12 ? outline : [],
  };
}
