import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ConfigProvider } from "antd";
import { SpellRankMarker } from "./SpellRankMarker";
import type { SpellRankChangeView } from "../../generated/atlas";

const damageChange: SpellRankChangeView = {
  fact: { kind: "damage", component_id: "main" },
  base_rank: 3,
  base_value: "6d6 fire",
  explanation: "+2d6 per additional rank",
};

describe("SpellRankMarker", () => {
  it("offers base context on activation and dismisses Escape without closing its host", async () => {
    const hostEscape = vi.fn();
    window.addEventListener("keydown", hostEscape);
    render(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <SpellRankMarker change={damageChange} label="Damage" />
      </ConfigProvider>,
    );
    const marker = screen.getByRole("button", {
      name: "Heightening details for Damage",
    });
    fireEvent.click(marker);
    const tooltip = await screen.findByRole("tooltip");
    expect(tooltip).toHaveTextContent("Base rank 3: 6d6 fire");
    expect(tooltip).toHaveTextContent("+2d6 per additional rank");
    try {
      fireEvent.keyDown(marker, { key: "Escape" });
      expect(hostEscape).not.toHaveBeenCalled();
    } finally {
      window.removeEventListener("keydown", hostEscape);
    }
    await waitFor(() => expect(screen.queryByRole("tooltip")).toBeNull());
  });

  it("keeps the marker and focus when the explanation changes", async () => {
    const { rerender } = render(
      <SpellRankMarker change={damageChange} label="Damage" />,
    );
    const marker = screen.getByRole("button", {
      name: "Heightening details for Damage",
    });
    marker.focus();
    expect(await screen.findByRole("tooltip")).toHaveTextContent("6d6 fire");
    rerender(
      <SpellRankMarker
        change={{ ...damageChange, base_value: "8d6 fire" }}
        label="Damage"
      />,
    );
    expect(screen.getByRole("button", { name: "Heightening details for Damage" })).toBe(
      marker,
    );
    expect(marker).toHaveFocus();
    await waitFor(() =>
      expect(screen.getByRole("tooltip")).toHaveTextContent("8d6 fire"),
    );
  });

  it("adds no control when there is no verified change", () => {
    render(<SpellRankMarker label="Target" />);
    expect(screen.queryByRole("button")).toBeNull();
    expect(screen.queryByRole("tooltip")).toBeNull();
  });
});
