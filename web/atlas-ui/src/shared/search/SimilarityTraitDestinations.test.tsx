import type { ReactNode } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import { createMemoryHistory } from "@tanstack/react-router";
import { createAtlasRouter } from "../../app/router";
import { readinessFixture } from "../../test/fixtures";
import { RouterTestContext } from "../../test/router";
import { similarityPartsFixture } from "../../test/similarityFixtures";
import { SimilaritySeedLink } from "./SimilaritySeedLink";
import { SimilaritySectionChoices } from "./SimilaritySectionChoices";
import {
  SearchProfileContext,
  useSearchProfileSelection,
} from "./SearchProfileContext";
import {
  DEFAULT_SEARCH_PREFERENCES,
  SEARCH_PREFERENCES_KEY,
} from "./searchPreferences";
import { ReferencePreviewContext } from "../records/ReferencePreviewContext";

it.each(["all", "prefer"] as const)(
  "preserves explicit versions=%s in both native trait destinations against the opposite launch preference",
  (versions) => {
    localStorage.setItem(
      SEARCH_PREFERENCES_KEY,
      JSON.stringify({
        ...DEFAULT_SEARCH_PREFERENCES,
        preferRemaster: versions === "all",
      }),
    );
    const router = createAtlasRouter(
      createMemoryHistory({ initialEntries: [`/search?versions=${versions}`] }),
    );
    function Owner({ children }: { children: ReactNode }) {
      const profile = useSearchProfileSelection(readinessFixture());
      return <SearchProfileContext value={profile}>{children}</SearchProfileContext>;
    }
    const openTrait = vi.fn(),
      onReference = vi.fn();
    render(
      <RouterTestContext router={router}>
        <Owner>
          <ReferencePreviewContext value={{ open: vi.fn(), openTrait }}>
            <SimilaritySeedLink
              seed={{ kind: "trait", trait_key: "trait:fire" }}
              title="Fire seed"
            />
            <SimilaritySectionChoices
              seed={{ kind: "trait", trait_key: "trait:fire" }}
              parts={[
                {
                  ...similarityPartsFixture().parts[0],
                  label: "Definition",
                  owner_label: "Fire definition",
                },
              ]}
              selected={null}
              onChange={vi.fn()}
              onReference={onReference}
            />
          </ReferencePreviewContext>
        </Owner>
      </RouterTestContext>,
    );
    const seed = screen.getByRole("link", { name: "Fire seed" });
    const section = screen.getByRole("link", { name: "Fire definition" });
    for (const link of [seed, section]) {
      expect(link).toHaveAttribute("href", `/traits/trait:fire?versions=${versions}`);
      fireEvent.click(link, { metaKey: true });
    }
    expect(openTrait).not.toHaveBeenCalled();
    expect(onReference).not.toHaveBeenCalled();
  },
);
