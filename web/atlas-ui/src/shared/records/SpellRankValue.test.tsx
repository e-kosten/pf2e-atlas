import {
  fireEvent,
  render as renderWithContext,
  screen,
  waitFor,
} from "@testing-library/react";
import { RouterTestContext } from "../../test/router";
import { describe, expect, it, vi } from "vitest";
import { ConfigProvider } from "antd";
import { ReferenceHelpLink, ReferenceHelpProvider } from "./ReferenceHelp";
import { SpellRankValue } from "./SpellRankValue";
import type { SpellRankChangeView } from "../../generated/atlas";

const render = (ui: React.ReactNode) =>
  renderWithContext(ui, { wrapper: RouterTestContext });

const damageChange: SpellRankChangeView = {
  fact: { kind: "damage", component_id: "main" },
  base_rank: 3,
  base_value: "6d6 fire",
  explanation: "+2d6 per additional rank",
};

describe("SpellRankValue", () => {
  it("offers base context on activation and dismisses Escape without closing its host", async () => {
    const hostEscape = vi.fn();
    window.addEventListener("keydown", hostEscape);
    render(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <SpellRankValue change={damageChange} label="Damage">
          10d6 fire
        </SpellRankValue>
      </ConfigProvider>,
    );
    const marker = screen.getByRole("button", {
      name: /Heightening details for Damage: 10d6 fire/,
    });
    fireEvent.click(marker);
    const explanation = await screen.findByRole("dialog");
    expect(explanation).toHaveTextContent("Base rank: 6d6 fire");
    expect(explanation).not.toHaveTextContent("Base rank 3");
    expect(explanation).not.toHaveTextContent("+2d6 per additional rank");
    try {
      fireEvent.keyDown(explanation, { key: "Escape" });
      expect(hostEscape).not.toHaveBeenCalled();
    } finally {
      window.removeEventListener("keydown", hostEscape);
    }
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    await waitFor(() => expect(marker).toHaveFocus());
  });

  it("opens only on activation and keeps the value and open explanation when context changes", async () => {
    const content = (change: SpellRankChangeView) => (
      <ConfigProvider theme={{ token: { motion: false } }}>
        <SpellRankValue change={change} label="Damage">
          10d6 fire
        </SpellRankValue>
      </ConfigProvider>
    );
    const { rerender } = render(content(damageChange));
    const marker = screen.getByRole("button", {
      name: /Heightening details for Damage: 10d6 fire/,
    });
    marker.focus();
    fireEvent.mouseEnter(marker);
    expect(screen.queryByRole("dialog")).toBeNull();
    fireEvent.click(marker);
    expect(await screen.findByRole("dialog")).toHaveTextContent("6d6 fire");
    const explanation = screen.getByRole("dialog");
    rerender(content({ ...damageChange, base_value: "8d6 fire" }));
    expect(
      screen.getByRole("button", { name: /Heightening details for Damage: 10d6 fire/ }),
    ).toBe(marker);
    await waitFor(() =>
      expect(screen.getByRole("dialog")).toHaveTextContent("8d6 fire"),
    );
    expect(screen.getByRole("dialog")).toBe(explanation);
    fireEvent.click(screen.getByRole("button", { name: "Close explanation" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });

  it("adds no control when there is no verified change", () => {
    render(<SpellRankValue label="Target">1 creature</SpellRankValue>);
    expect(screen.getByText("1 creature")).toBeVisible();
    expect(screen.queryByRole("button")).toBeNull();
    expect(screen.queryByRole("dialog")).toBeNull();
  });
});

it("keeps definition and heightening actions separate for interactive categorical content", async () => {
  const help = {
    definition_key: "damage:fire",
    availability: "available" as const,
    destination: {
      kind: "record" as const,
      navigation: {
        record_key: "actions:fire",
        embedded_path: [],
        field: null,
        passage: null,
        source_fingerprint: null,
      },
    },
    related_filters: [],
  };
  const onHelp = vi.fn();
  const { container } = render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <ReferenceHelpProvider onHelp={onHelp}>
        <SpellRankValue change={damageChange} label="Damage" interactiveContent>
          10d6 <ReferenceHelpLink help={help}>fire</ReferenceHelpLink>
        </SpellRankValue>
      </ReferenceHelpProvider>
    </ConfigProvider>,
  );
  expect(container.querySelector("button a, a button")).toBeNull();
  const link = screen.getByRole("link", { name: "fire" });
  fireEvent.click(link);
  expect(onHelp).toHaveBeenCalledWith(help, link, undefined);
  expect(screen.queryByRole("dialog")).toBeNull();
  const explanation = screen.getByRole("button", {
    name: "Heightening details for Damage",
  });
  fireEvent.click(explanation);
  expect(await screen.findByRole("dialog")).toHaveTextContent("Base rank: 6d6 fire");
  fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  await waitFor(() => expect(explanation).toHaveFocus());
});
