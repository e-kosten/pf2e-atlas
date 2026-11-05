import type { QueryRelationshipDirection } from "../../generated/atlas";

export const linkScopeCopy: Record<
  QueryRelationshipDirection,
  { main: string; whole: string }
> = {
  incoming: {
    main: "Check links in each result’s main entry.",
    whole:
      "Also check links in each result’s contents, such as its abilities, spells, and equipment.",
  },
  outgoing: {
    main: "Check links in the selected entry itself.",
    whole:
      "Also check links in the selected entry’s contents, such as its abilities, spells, and equipment.",
  },
};
