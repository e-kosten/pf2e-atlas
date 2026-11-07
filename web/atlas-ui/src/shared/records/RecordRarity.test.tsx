import { render, screen } from "@testing-library/react";
import { RecordRarity } from "./RecordRarity";

describe("RecordRarity", () => {
  it.each([
    ["common", "Common"],
    ["uncommon", "Uncommon"],
    ["rare", "Rare"],
    ["unique", "Unique"],
    [" RARE ", "Rare"],
    ["legendary", "Legendary"],
  ])("renders rarity %s as %s", (rarity, expected) => {
    render(<RecordRarity rarity={{ value: rarity, label: expected }} />);
    expect(screen.getByLabelText(`Rarity: ${expected}`)).toBeInTheDocument();
  });

  it.each([null, undefined, "", " "])("omits unavailable rarity %s", (rarity) => {
    const { container } = render(
      <RecordRarity
        rarity={rarity == null ? rarity : { value: rarity, label: rarity }}
      />,
    );
    expect(container).toBeEmptyDOMElement();
  });
});
