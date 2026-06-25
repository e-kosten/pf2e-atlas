import type { ThemeConfig } from "antd";
import { theme as antTheme } from "antd";
import type { CSSProperties } from "react";

export type ColorSchemePreference = "system" | "light" | "dark";
export type ResolvedColorScheme = "light" | "dark";

export const COLOR_SCHEME_STORAGE_KEY = "atlas-ui-color-scheme";

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
    "--warning": token.colorWarningText,
    "--danger": token.colorErrorText,
    "--shadow": token.boxShadowSecondary,
    "--overlay-backdrop": token.colorBgMask,
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
  return {
    algorithm: scheme === "dark" ? antTheme.darkAlgorithm : antTheme.defaultAlgorithm,
    token: {
      borderRadius: atlasTheme.radius.md,
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
      },
      Tag: {
        borderRadiusSM: atlasTheme.radius.pill,
      },
    },
  };
}
