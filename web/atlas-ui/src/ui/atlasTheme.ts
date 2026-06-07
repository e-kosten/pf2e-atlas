import type { ThemeConfig } from "antd";
import { theme as antTheme } from "antd";
import type { CSSProperties } from "react";

export type ColorSchemePreference = "system" | "light" | "dark";
export type ResolvedColorScheme = "light" | "dark";

export const COLOR_SCHEME_STORAGE_KEY = "atlas-ui-color-scheme";

type AtlasPalette = {
  bg: string;
  panel: string;
  panelSubtle: string;
  text: string;
  muted: string;
  line: string;
  accent: string;
  accentStrong: string;
  warning: string;
  danger: string;
  shadow: string;
};

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
  color: {
    light: {
      bg: "#f6f7f4",
      panel: "#ffffff",
      panelSubtle: "#eef3f0",
      text: "#1d2522",
      muted: "#62706b",
      line: "#d8dfdc",
      accent: "#1f766f",
      accentStrong: "#14544f",
      warning: "#a56a00",
      danger: "#9b2f2f",
      shadow: "0 16px 36px rgba(24, 38, 33, 0.08)",
    },
    dark: {
      bg: "#111816",
      panel: "#18211e",
      panelSubtle: "#202b27",
      text: "#edf3ef",
      muted: "#a6b4ae",
      line: "#33423d",
      accent: "#4fb7aa",
      accentStrong: "#7dd4ca",
      warning: "#d39b3d",
      danger: "#e06f6f",
      shadow: "0 16px 36px rgba(0, 0, 0, 0.28)",
    },
  } satisfies Record<ResolvedColorScheme, AtlasPalette>,
};

export function atlasCssVariables(scheme: ResolvedColorScheme): CSSProperties {
  const color = atlasTheme.color[scheme];
  return {
    colorScheme: scheme,
    "--bg": color.bg,
    "--panel": color.panel,
    "--panel-subtle": color.panelSubtle,
    "--text": color.text,
    "--muted": color.muted,
    "--line": color.line,
    "--accent": color.accent,
    "--accent-strong": color.accentStrong,
    "--warning": color.warning,
    "--danger": color.danger,
    "--shadow": color.shadow,
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
  const color = atlasTheme.color[scheme];
  return {
    algorithm: scheme === "dark" ? antTheme.darkAlgorithm : antTheme.defaultAlgorithm,
    token: {
      colorPrimary: color.accent,
      colorInfo: color.accent,
      colorLink: color.accentStrong,
      colorBgBase: color.bg,
      colorBgContainer: color.panel,
      colorBgElevated: color.panel,
      colorText: color.text,
      colorTextSecondary: color.muted,
      colorBorder: color.line,
      colorBorderSecondary: color.line,
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
        borderColor: color.line,
        headerBg: color.panelSubtle,
        headerColor: color.text,
        rowHoverBg: color.panelSubtle,
      },
      Tag: {
        borderRadiusSM: atlasTheme.radius.pill,
      },
    },
  };
}
