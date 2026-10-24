/** Presentation of already prepared headings; never interprets source rules. */
export function documentHeadings(html: string, identity: string) {
  const parsed = new DOMParser().parseFromString(html, "text/html");
  const headings = [...parsed.body.querySelectorAll<HTMLElement>("h1,h2,h3,h4,h5,h6")];
  const firstLevel = Math.min(
    ...headings
      .filter((heading) => !heading.closest("[data-atlas-inclusion]"))
      .map((heading) => Number(heading.tagName.slice(1))),
  );
  const prefix = `atlas-section-${Array.from(identity).reduce((hash, char) => Math.imul(hash ^ char.charCodeAt(0), 16777619), 2166136261) >>> 0}`;
  const outline: { id: string; label: string }[] = [];
  const occurrences = new Map<string, number>();
  const inclusionLevels = new Map<string, number>();
  headings.forEach((heading) => {
    const namespace = heading
      .closest("[data-atlas-inclusion]")
      ?.getAttribute("data-atlas-inclusion");
    if (namespace != null)
      inclusionLevels.set(
        namespace,
        Math.min(inclusionLevels.get(namespace) ?? 6, Number(heading.tagName.slice(1))),
      );
  });
  let hostLevel = 3;
  headings.forEach((heading) => {
    const inclusion = heading.closest("[data-atlas-inclusion]");
    const namespace = inclusion?.getAttribute("data-atlas-inclusion");
    const sourceLevel = Number(heading.tagName.slice(1));
    const replacement = parsed.createElement(
      `h${inclusion ? Math.min(6, hostLevel + 1 + sourceLevel - (inclusionLevels.get(namespace!) ?? sourceLevel)) : Math.min(6, sourceLevel - firstLevel + 3)}`,
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
    const identitySlug = inclusion ? `inclusion-${namespace}-${slug}` : slug;
    const occurrence = (occurrences.get(identitySlug) ?? 0) + 1;
    occurrences.set(identitySlug, occurrence);
    replacement.id =
      (!inclusion && heading.id) ||
      `${prefix}-${identitySlug}${occurrence > 1 ? `-${occurrence}` : ""}`;
    replacement.tabIndex = -1;
    replacement.append(...heading.childNodes);
    if (!inclusion) hostLevel = Number(replacement.tagName.slice(1));
    if (!inclusion && sourceLevel === firstLevel && replacement.textContent?.trim())
      outline.push({ id: replacement.id, label: replacement.textContent.trim() });
    heading.replaceWith(replacement);
  });
  return {
    html: parsed.body.innerHTML,
    outline: outline.length >= 3 && outline.length <= 12 ? outline : [],
  };
}
