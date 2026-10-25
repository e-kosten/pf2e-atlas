import type { GlobalToken, ThemeConfig } from "antd";
import { theme as antTheme } from "antd";
import type { CSSProperties } from "react";
import {
  resolveAtlasPalette,
  type AtlasPalette,
  type ResolvedColorScheme,
} from "./themeDefinitions";

export type ColorSchemePreference = "system" | "light" | "dark";
export type { ResolvedColorScheme } from "./themeDefinitions";

export const COLOR_SCHEME_STORAGE_KEY = "atlas-ui-color-scheme";

export const atlasTheme = {
  radius: { sm: 4, md: 6, lg: 8, pill: 999 },
  controlHeight: 34,
  density: { panelGap: 16, panelPadding: 16 },
};

const FONT_FAMILY =
  'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif';

export function atlasCssVariables(scheme: ResolvedColorScheme): CSSProperties {
  const palette = resolveAtlasPalette(scheme);
  const token = antTheme.getDesignToken(antDesignTheme(scheme));
  return {
    "--theme-font": FONT_FAMILY,
    "--heading-color": palette.text,
    "--record-kind-text": palette.recordKind.text,
    "--record-kind-bg": palette.recordKind.background,
    "--record-kind-border": palette.recordKind.border,
    "--context-bar-bg": palette.contextBar,
    "--shell-bg": palette.shell.background,
    "--shell-text": palette.shell.text,
    "--shell-border": palette.shell.border,
    "--brand-text": palette.brand.text,
    "--filter-include-text": palette.filter.include.text,
    "--filter-include-bg": palette.filter.include.background,
    "--filter-include-border": palette.filter.include.border,
    "--filter-exclude-text": palette.filter.exclude.text,
    "--filter-exclude-bg": palette.filter.exclude.background,
    "--filter-exclude-border": palette.filter.exclude.border,
    ...Object.fromEntries(
      Object.entries(palette.rarity).flatMap(([rarity, colors]) => [
        [`--rarity-${rarity}-text`, colors.text],
        [`--rarity-${rarity}-bg`, colors.background],
        [`--rarity-${rarity}-border`, colors.border],
      ]),
    ),
    "--bg": token.colorBgLayout,
    "--panel": token.colorBgContainer,
    "--panel-subtle": token.colorFillAlter,
    "--panel-elevated": token.colorBgElevated,
    "--text": token.colorText,
    "--muted": token.colorTextDescription,
    "--line": token.colorBorderSecondary,
    "--split": token.colorSplit,
    "--primary": token.colorPrimary,
    "--primary-active": token.colorPrimaryActive,
    "--link": token.colorLink,
    "--selection-bg": token.colorPrimaryBg,
    "--selection-text": palette.selection.text,
    "--selection-hover-bg": token.colorPrimaryBgHover,
    "--selection-border": token.colorPrimaryBorder,
    "--warning": token.colorWarningText,
    "--danger": token.colorErrorText,
    "--shadow": token.boxShadowSecondary,
    "--overlay-backdrop": token.colorBgMask,
    "--hp-current": palette.hp.healthy,
    "--hp-bloodied": palette.hp.wounded,
    "--hp-critical": palette.hp.critical,
    "--hp-current-text": palette.status.success.text,
    "--hp-bloodied-text": palette.status.warning.text,
    "--hp-critical-text": palette.status.error.text,
    "--hp-temp": palette.hp.temporary,
    "--hp-missing": palette.hp.missing,
    "--radius-sm": `${atlasTheme.radius.sm}px`,
    "--radius-md": `${atlasTheme.radius.md}px`,
    "--radius-lg": `${atlasTheme.radius.lg}px`,
    "--radius-pill": `${atlasTheme.radius.pill}px`,
    "--space-control": `${token.paddingXS}px`,
    "--space-group": `${token.padding}px`,
    "--space-section": `${token.paddingLG}px`,
    "--control-height": `${atlasTheme.controlHeight}px`,
    "--panel-gap": `${atlasTheme.density.panelGap}px`,
    "--panel-padding": `${atlasTheme.density.panelPadding}px`,
  } as CSSProperties;
}

export function antDesignTheme(scheme: ResolvedColorScheme): ThemeConfig {
  return scheme === "light" ? lightThemeConfig : darkThemeConfig;
}

function statusTokens(palette: AtlasPalette): Partial<GlobalToken> {
  return Object.fromEntries(
    Object.entries(palette.status).flatMap(([role, colors]) => {
      const name = `color${role[0].toUpperCase()}${role.slice(1)}`;
      return [
        [name, colors.text],
        [`${name}Text`, colors.text],
        [`${name}TextHover`, colors.text],
        [`${name}TextActive`, colors.text],
        [`${name}Bg`, colors.background],
        [`${name}BgHover`, colors.background],
        [`${name}Border`, colors.border],
        [`${name}BorderHover`, colors.border],
      ];
    }),
  ) as Partial<GlobalToken>;
}

function themeConfig(scheme: ResolvedColorScheme, palette: AtlasPalette): ThemeConfig {
  const statuses = statusTokens(palette);
  return {
    algorithm: (seed, map) => ({
      ...(scheme === "dark" ? antTheme.darkAlgorithm : antTheme.defaultAlgorithm)(
        seed,
        map,
      ),
      // Dark derivation otherwise re-tones the authored oxblood seed.
      colorPrimary: palette.primary,
      colorLink: palette.link,
      colorLinkHover: palette.link,
      colorLinkActive: palette.link,
      ...statuses,
    }),
    token: {
      ...statuses,
      borderRadius: atlasTheme.radius.md,
      controlHeight: atlasTheme.controlHeight,
      fontFamily: FONT_FAMILY,
      colorPrimary: palette.primary,
      colorPrimaryHover: palette.primaryHover,
      colorPrimaryActive: palette.primaryActive,
      colorPrimaryText: palette.link,
      colorPrimaryTextHover: palette.link,
      colorPrimaryTextActive: palette.link,
      colorPrimaryBg: palette.selection.background,
      colorPrimaryBgHover: palette.selection.hover,
      colorPrimaryBorder: palette.selection.border,
      colorPrimaryBorderHover: palette.selection.border,
      colorLink: palette.link,
      colorLinkHover: palette.link,
      colorLinkActive: palette.link,
      colorBgBase: palette.surface,
      colorBgLayout: palette.background,
      colorBgContainer: palette.surface,
      colorBgElevated: palette.elevated,
      colorFillAlter: palette.subtle,
      colorBgTextHover: palette.subtle,
      colorTextBase: palette.text,
      colorText: palette.text,
      colorTextSecondary: palette.muted,
      colorTextDescription: palette.muted,
      colorTextPlaceholder: palette.muted,
      colorTextTertiary: palette.muted,
      colorBorder: palette.border,
      colorBorderSecondary: palette.separator,
      colorSplit: palette.separator,
      colorWhite: palette.onPrimary,
      colorTextLightSolid: palette.onPrimary,
      colorBgSolid: palette.shell.background,
      colorBgSolidHover: palette.navigation.hoverBackground,
      colorBgSolidActive: palette.shell.background,
      colorBgSpotlight: palette.shell.background,
      colorBgMask: palette.backdrop,
      controlItemBgActive: palette.selection.background,
      controlItemBgActiveHover: palette.selection.hover,
      boxShadow: `0 8px 24px ${palette.shadow}`,
      boxShadowSecondary: `0 6px 18px ${palette.shadow}`,
    },
    components: {
      Menu: {
        itemBg: palette.shell.background,
        itemColor: palette.shell.text,
        itemSelectedColor: palette.navigation.activeText,
        itemSelectedBg: palette.navigation.activeBackground,
        itemHoverColor: palette.navigation.hoverText,
        itemHoverBg: palette.navigation.hoverBackground,
        darkItemBg: palette.shell.background,
        darkItemColor: palette.shell.text,
        darkItemSelectedColor: palette.navigation.activeText,
        darkItemSelectedBg: palette.navigation.activeBackground,
        darkItemHoverColor: palette.navigation.hoverText,
        darkItemHoverBg: palette.navigation.hoverBackground,
      },
      Button: {
        colorPrimary: palette.primary,
        primaryColor: palette.onPrimary,
        dangerColor: palette.onPrimary,
        borderRadius: atlasTheme.radius.md,
        controlHeight: atlasTheme.controlHeight,
      },
      Checkbox: { colorWhite: palette.onPrimary },
      Radio: { colorWhite: palette.onPrimary },
      Switch: { colorTextLightSolid: palette.onPrimary },
      Input: {
        borderRadius: atlasTheme.radius.md,
        controlHeight: atlasTheme.controlHeight,
      },
      InputNumber: {
        borderRadius: atlasTheme.radius.md,
        controlHeight: atlasTheme.controlHeight,
      },
      Select: {
        borderRadius: atlasTheme.radius.md,
        controlHeight: atlasTheme.controlHeight,
      },
      Tabs: {
        inkBarColor: palette.navigation.activeText,
        itemActiveColor: palette.navigation.pressedText,
        itemHoverColor: palette.navigation.hoverText,
        itemSelectedColor: palette.navigation.activeText,
      },
      Table: {
        borderColor: palette.separator,
        headerBg: palette.surface,
        headerColor: palette.text,
        headerSplitColor: palette.separator,
        headerSortActiveBg: palette.selection.background,
        headerSortHoverBg: palette.selection.hover,
        rowHoverBg: palette.subtle,
        rowSelectedBg: palette.selection.background,
        rowSelectedHoverBg: palette.selection.hover,
      },
      Tag: { borderRadiusSM: atlasTheme.radius.pill },
    },
  };
}

// Stable objects prevent appearance changes from remounting document readers.
const lightThemeConfig = themeConfig("light", resolveAtlasPalette("light"));
const darkThemeConfig = themeConfig("dark", resolveAtlasPalette("dark"));
