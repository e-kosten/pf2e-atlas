import { createMemoryHistory } from "@tanstack/react-router";
import { atlasRouter } from "./router";
import { recordDestination } from "./destinations";
import { destinationHref, navigateDestination } from "./navigation";
import { documentSection } from "./urlState";

it.each(["creature-record:key-casting", "literal%23/雪", "literal%2F-id"])(
  "retains the literal document section %s in native links and navigation",
  async (section) => {
    atlasRouter.update({
      history: createMemoryHistory({ initialEntries: ["/search"] }),
    });
    await atlasRouter.load();
    const destination = recordDestination({ recordKey: "record:key", section });
    const href = destinationHref(destination);
    expect(documentSection(new URL(href, window.location.origin).href)).toBe(section);
    await navigateDestination(destination);
    expect(documentSection(atlasRouter.state.location.href)).toBe(section);
  },
);
