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
    render(<RecordRarity rarity={rarity} />);
    expect(screen.getByLabelText(`Rarity: ${expected}`)).toBeInTheDocument();
  });

  it.each([null, undefined, "", " "])("omits unavailable rarity %s", (rarity) => {
    const { container } = render(<RecordRarity rarity={rarity} />);
    expect(container).toBeEmptyDOMElement();
  });
});
