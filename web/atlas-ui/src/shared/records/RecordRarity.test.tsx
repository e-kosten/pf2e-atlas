import { render, screen } from "@testing-library/react";
import { RecordRarity } from "./RecordRarity";

describe("RecordRarity", () => {
  it.each(["common", "uncommon", "rare", "unique"])(
    "renders the %s rarity",
    (rarity) => {
      render(<RecordRarity rarity={rarity} />);
      expect(
        screen.getByLabelText(`Rarity: ${rarity[0].toUpperCase()}${rarity.slice(1)}`),
      ).toBeInTheDocument();
    },
  );

  it("omits an empty rarity", () => {
    const { container } = render(<RecordRarity rarity=" " />);
    expect(container).toBeEmptyDOMElement();
  });
});
