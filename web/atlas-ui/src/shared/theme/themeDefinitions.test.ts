import { theme as antTheme } from "antd";
// @ts-expect-error Vitest runs in Node; the product build intentionally omits Node types.
import { readFileSync } from "node:fs";
import { antDesignTheme, atlasCssVariables } from "./atlasTheme";
import { atlasPalettes, darkNeutralRamp } from "./themeDefinitions";

function luminance(hex: string) {
  const channels = hex
    .slice(1)
    .match(/../g)!
    .map((part) => {
      const value = parseInt(part, 16) / 255;
      return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
    });
  return channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722;
}

function contrast(foreground: string, background: string) {
  const values = [luminance(foreground), luminance(background)].sort((a, b) => b - a);
  return (values[0] + 0.05) / (values[1] + 0.05);
}

describe("Atlas product palette", () => {
  for (const scheme of ["light", "dark"] as const) {
    it(`${scheme} keeps text, rarity, and HP roles distinguishable`, () => {
      const palette = atlasPalettes[scheme];
      for (const surface of [palette.background, palette.surface, palette.elevated]) {
        expect(contrast(palette.text, surface)).toBeGreaterThanOrEqual(7);
        expect(contrast(palette.muted, surface)).toBeGreaterThanOrEqual(4.5);
      }
      expect(contrast(palette.onPrimary, palette.primary)).toBeGreaterThanOrEqual(4.5);
      expect(
        contrast(palette.selection.text, palette.selection.background),
      ).toBeGreaterThanOrEqual(4.5);
      expect(
        contrast(palette.shell.text, palette.shell.background),
      ).toBeGreaterThanOrEqual(4.5);
      expect(
        contrast(palette.brand.text, palette.shell.background),
      ).toBeGreaterThanOrEqual(4.5);
      expect(
        contrast(palette.navigation.activeText, palette.navigation.activeBackground),
      ).toBeGreaterThanOrEqual(4.5);
      expect(
        contrast(palette.navigation.hoverText, palette.navigation.hoverBackground),
      ).toBeGreaterThanOrEqual(4.5);
      for (const badge of Object.values(palette.rarity)) {
        expect(contrast(badge.text, badge.background)).toBeGreaterThanOrEqual(4.5);
      }
      expect(Object.keys(palette.rarity)).toEqual([
        "common",
        "uncommon",
        "rare",
        "unique",
      ]);
      for (const fill of [
        palette.hp.healthy,
        palette.hp.wounded,
        palette.hp.critical,
        palette.hp.temporary,
      ]) {
        expect(contrast(fill, palette.hp.missing)).toBeGreaterThanOrEqual(3);
      }
    });

    it(`${scheme} passes accepted anchors through Ant and CSS tokens`, () => {
      const palette = atlasPalettes[scheme];
      const config = antDesignTheme(scheme);
      const tokens = antTheme.getDesignToken(config);
      const variables = atlasCssVariables(scheme);

      expect(antDesignTheme(scheme)).toBe(config);
      expect(tokens.colorPrimary.toLowerCase()).toBe(palette.primary.toLowerCase());
      expect(tokens.colorBgLayout.toLowerCase()).toBe(palette.background.toLowerCase());
      expect(tokens.colorBgContainer.toLowerCase()).toBe(palette.surface.toLowerCase());
      expect(tokens.colorText.toLowerCase()).toBe(palette.text.toLowerCase());
      expect(tokens.colorTextDescription.toLowerCase()).toBe(
        palette.muted.toLowerCase(),
      );
      expect(tokens.colorPrimaryBg.toLowerCase()).toBe(
        palette.selection.background.toLowerCase(),
      );
      expect(tokens.colorLink.toLowerCase()).toBe(palette.link.toLowerCase());
      expect(variables["--hp-temp" as keyof typeof variables]).toBe(
        palette.hp.temporary,
      );
      expect(variables["--rarity-common-text" as keyof typeof variables]).toBe(
        palette.rarity.common.text,
      );
      expect(variables["--rarity-rare-text" as keyof typeof variables]).toBe(
        palette.rarity.rare.text,
      );
      expect(variables["--rarity-unique-text" as keyof typeof variables]).toBe(
        palette.rarity.unique.text,
      );
      expect(variables["--filter-include-text" as keyof typeof variables]).toBe(
        palette.filter.include.text,
      );
      expect(variables["--filter-exclude-text" as keyof typeof variables]).toBe(
        palette.filter.exclude.text,
      );
      expect(variables["--selection-text" as keyof typeof variables]).toBe(
        palette.selection.text,
      );
      expect(variables["--context-bar-bg" as keyof typeof variables]).toBe(
        palette.contextBar,
      );
      expect(variables["--brand-text" as keyof typeof variables]).toBe(
        palette.brand.text,
      );
      expect(config.components?.Menu).toMatchObject({
        itemColor: palette.shell.text,
        itemSelectedColor: palette.navigation.activeText,
        itemSelectedBg: palette.navigation.activeBackground,
        itemHoverColor: palette.navigation.hoverText,
        itemHoverBg: palette.navigation.hoverBackground,
        darkItemSelectedColor: palette.navigation.activeText,
        darkItemColor: palette.shell.text,
        darkItemSelectedBg: palette.navigation.activeBackground,
        darkItemHoverColor: palette.navigation.hoverText,
        darkItemHoverBg: palette.navigation.hoverBackground,
      });
      expect(config.components?.Tabs).toMatchObject({
        inkBarColor: palette.navigation.activeText,
        itemActiveColor: palette.navigation.pressedText,
        itemHoverColor: palette.navigation.hoverText,
        itemSelectedColor: palette.navigation.activeText,
      });
      expect(config.components?.Table?.headerBg).toBe(palette.surface);
    });
  }

  it("maps one ordered warm-umber ramp through every dark structural role", () => {
    const ramp = darkNeutralRamp;
    const palette = atlasPalettes.dark;
    const config = antDesignTheme("dark");
    const tokens = antTheme.getDesignToken(config);
    const variables = atlasCssVariables("dark");
    const orderedColors = [
      ramp.shell,
      ramp.background,
      ramp.surface,
      ramp.elevated,
      ramp.subtle,
      ramp.separator,
      ramp.border,
    ];

    expect(antDesignTheme("dark")).toBe(config);
    orderedColors.slice(1).forEach((color, index) => {
      expect(luminance(color)).toBeGreaterThan(luminance(orderedColors[index]));
    });
    expect(palette).toMatchObject({
      background: ramp.background,
      surface: ramp.surface,
      elevated: ramp.elevated,
      subtle: ramp.subtle,
      separator: ramp.separator,
      border: ramp.border,
      contextBar: ramp.surface,
      shell: { background: ramp.shell, border: ramp.separator },
    });
    expect(tokens.colorBgLayout.toLowerCase()).toBe(ramp.background.toLowerCase());
    expect(tokens.colorBgContainer.toLowerCase()).toBe(ramp.surface.toLowerCase());
    expect(tokens.colorFillAlter.toLowerCase()).toBe(ramp.subtle.toLowerCase());
    expect(variables["--panel-subtle" as keyof typeof variables]).toBe(
      tokens.colorFillAlter,
    );
    for (const surface of [
      ramp.shell,
      ramp.background,
      ramp.surface,
      ramp.elevated,
      ramp.subtle,
    ]) {
      expect(contrast(palette.text, surface)).toBeGreaterThanOrEqual(7);
      expect(contrast(palette.muted, surface)).toBeGreaterThanOrEqual(4.5);
    }
  });

  it("keeps the pre-React light fallback aligned with runtime variables", () => {
    const themeFallbackCss = readFileSync("src/styles/theme.css", "utf8") as string;
    const root = themeFallbackCss.match(/:root\s*\{([\s\S]*?)\}/)?.[1];
    expect(root).toBeDefined();
    const fallback = Object.fromEntries(
      Array.from(root!.matchAll(/(--[a-z0-9-]+)\s*:\s*([^;]+);/g), (match) => [
        match[1],
        normalizeCssValue(match[2]),
      ]),
    );

    for (const [name, value] of Object.entries(atlasCssVariables("light"))) {
      expect(fallback[name], name).toBe(normalizeCssValue(String(value)));
    }
  });
});

function normalizeCssValue(value: string) {
  return value.trim().replace(/\s+/g, " ").toLowerCase();
}
