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
import { ReferenceHelpProvider } from "../records/ReferenceHelp";

it("keeps both trait destinations exact and native while ordinary clicks open contextual readers", () => {
  const router = createAtlasRouter(
    createMemoryHistory({ initialEntries: ["/search?q=fire"] }),
  );
  function Owner({ children }: { children: ReactNode }) {
    const profile = useSearchProfileSelection(readinessFixture());
    return <SearchProfileContext value={profile}>{children}</SearchProfileContext>;
  }
  const onHelp = vi.fn(),
    onReference = vi.fn();
  render(
    <RouterTestContext router={router}>
      <Owner>
        <ReferenceHelpProvider onHelp={onHelp}>
          <SimilaritySeedLink
            seed={{ kind: "trait", trait_key: "trait:fire" }}
            title="Fire seed"
            help={{
              definition_key: "trait:fire",
              destination: { kind: "definition", key: "trait:fire" },
            }}
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
        </ReferenceHelpProvider>
      </Owner>
    </RouterTestContext>,
  );
  const seed = screen.getByRole("link", { name: "Fire seed" });
  const section = screen.getByRole("link", { name: "Fire definition" });
  for (const link of [seed, section]) {
    expect(link).toHaveAttribute("href", "/traits/trait:fire");
    fireEvent.click(link, { metaKey: true });
  }
  expect(onHelp).not.toHaveBeenCalled();
  expect(onReference).not.toHaveBeenCalled();
  fireEvent.click(seed);
  expect(onHelp).toHaveBeenCalledWith(
    {
      definition_key: "trait:fire",
      destination: { kind: "definition", key: "trait:fire" },
    },
    seed,
    undefined,
  );
  fireEvent.click(section);
  expect(onReference).toHaveBeenCalledWith(
    expect.objectContaining({ owner_label: "Fire definition" }),
    section,
  );
});
