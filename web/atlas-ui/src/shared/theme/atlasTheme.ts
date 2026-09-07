import type { ThemeConfig } from "antd";
import { theme as antTheme } from "antd";
import type { CSSProperties } from "react";

export type ColorSchemePreference = "system" | "light" | "dark";
export type ResolvedColorScheme = "light" | "dark";

export const COLOR_SCHEME_STORAGE_KEY = "atlas-ui-color-scheme";
const COLOR_PREFIX = "#";

export const atlasTheme = {
  fontFamily:
    "Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, sans-serif",
  radius: {
    sm: 4,
    md: 6,
    lg: 8,
    pill: 999,
  },
  controlHeight: 34,
  density: {
    panelGap: 14,
    panelPadding: 14,
  },
};

export function atlasCssVariables(scheme: ResolvedColorScheme): CSSProperties {
  const token = antTheme.getDesignToken(themeConfig(scheme));
  return {
    colorScheme: scheme,
    "--bg": token.colorBgLayout,
    "--panel": token.colorBgContainer,
    "--panel-subtle": token.colorFillAlter,
    "--panel-elevated": token.colorBgElevated,
    "--text": token.colorText,
    "--muted": token.colorTextDescription,
    "--line": token.colorBorderSecondary,
    "--split": token.colorSplit,
    "--accent": token.colorPrimary,
    "--accent-strong": token.colorPrimaryActive,
    "--accent-text": token.colorLink,
    "--active-bg": token.colorPrimaryBg,
    "--active-bg-hover": token.colorPrimaryBgHover,
    "--active-border": token.colorPrimaryBorder,
    "--warning": token.colorWarningText,
    "--danger": token.colorErrorText,
    "--shadow": token.boxShadowSecondary,
    "--hp-current": token.colorSuccess,
    "--hp-bloodied": token.colorWarning,
    "--hp-critical": token.colorError,
    "--hp-temp": token.colorInfo,
    "--hp-missing": token.colorFillSecondary,
    "--radius-sm": `${atlasTheme.radius.sm}px`,
    "--radius-md": `${atlasTheme.radius.md}px`,
    "--radius-lg": `${atlasTheme.radius.lg}px`,
    "--radius-pill": `${atlasTheme.radius.pill}px`,
    "--control-height": `${atlasTheme.controlHeight}px`,
    "--panel-gap": `${atlasTheme.density.panelGap}px`,
    "--panel-padding": `${atlasTheme.density.panelPadding}px`,
  } as CSSProperties;
}

export function antDesignTheme(scheme: ResolvedColorScheme): ThemeConfig {
  return themeConfig(scheme);
}

function themeConfig(scheme: ResolvedColorScheme): ThemeConfig {
  const primary = atlasThemeSeed(scheme, "primary");
  const link = atlasThemeSeed(scheme, "link");
  return {
    algorithm: scheme === "dark" ? antTheme.darkAlgorithm : antTheme.defaultAlgorithm,
    token: {
      borderRadius: atlasTheme.radius.md,
      colorInfo: primary,
      colorLink: link,
      colorLinkHover: link,
      colorLinkActive: link,
      colorTextDescription:
        scheme === "dark" ? `${COLOR_PREFIX}bfbfbf` : `${COLOR_PREFIX}595959`,
      colorPrimary: primary,
      colorTextLightSolid:
        scheme === "dark" ? `${COLOR_PREFIX}141414` : `${COLOR_PREFIX}ffffff`,
      controlHeight: atlasTheme.controlHeight,
      fontFamily: atlasTheme.fontFamily,
    },
    components: {
      Button: {
        borderRadius: atlasTheme.radius.md,
        controlHeight: atlasTheme.controlHeight,
      },
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
      Table: {
        borderColor: "var(--line)",
        headerBg: "var(--panel-subtle)",
        headerColor: "var(--text)",
        rowHoverBg: "var(--panel-subtle)",
        rowSelectedBg: "var(--active-bg)",
        rowSelectedHoverBg: "var(--active-bg-hover)",
      },
      Tag: {
        borderRadiusSM: atlasTheme.radius.pill,
      },
    },
  };
}

function atlasThemeSeed(scheme: ResolvedColorScheme, role: "primary" | "link") {
  const variableName = `--atlas-theme-${scheme}-${role}`;
  if (typeof document === "undefined") {
    return fallbackTealSeed(scheme, role);
  }
  return (
    getComputedStyle(document.documentElement).getPropertyValue(variableName).trim() ||
    fallbackTealSeed(scheme, role)
  );
}

function fallbackTealSeed(scheme: ResolvedColorScheme, role: "primary" | "link") {
  if (scheme === "light") {
    return `${COLOR_PREFIX}0f766e`;
  }
  return role === "primary" ? `${COLOR_PREFIX}2dd4bf` : `${COLOR_PREFIX}5eead4`;
}
