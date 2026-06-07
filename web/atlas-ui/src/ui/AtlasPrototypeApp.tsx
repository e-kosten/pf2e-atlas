import { ConfigProvider } from "antd";
import { useEffect, useState } from "react";
import {
  COLOR_SCHEME_STORAGE_KEY,
  antDesignTheme,
  atlasCssVariables,
  type ColorSchemePreference,
  type ResolvedColorScheme,
} from "./atlasTheme";
import { AntPrototype } from "./AntPrototype";
import { PrototypeShell } from "./PrototypeShell";
import { useAtlasWorkspace } from "./useAtlasWorkspace";

export function AtlasPrototypeApp() {
  const [colorScheme, setColorScheme] =
    useState<ColorSchemePreference>(readStoredColorScheme);
  const systemColorScheme = useSystemColorScheme();
  const resolvedColorScheme =
    colorScheme === "system" ? systemColorScheme : colorScheme;
  const workspace = useAtlasWorkspace();

  useEffect(() => {
    document.documentElement.dataset.theme = resolvedColorScheme;
    Object.entries(atlasCssVariables(resolvedColorScheme)).forEach(
      ([property, value]) => {
        document.documentElement.style.setProperty(property, String(value));
      },
    );
  }, [resolvedColorScheme]);

  const updateColorScheme = (preference: ColorSchemePreference) => {
    setColorScheme(preference);
    localStorage.setItem(COLOR_SCHEME_STORAGE_KEY, preference);
  };

  return (
    <PrototypeShell
      colorScheme={colorScheme}
      onColorSchemeChange={updateColorScheme}
      resolvedColorScheme={resolvedColorScheme}
      workspace={workspace}
    >
      <ConfigProvider theme={antDesignTheme(resolvedColorScheme)}>
        <AntPrototype workspace={workspace} />
      </ConfigProvider>
    </PrototypeShell>
  );
}

function readStoredColorScheme(): ColorSchemePreference {
  const value = localStorage.getItem(COLOR_SCHEME_STORAGE_KEY);
  return value === "light" || value === "dark" || value === "system" ? value : "system";
}

function useSystemColorScheme(): ResolvedColorScheme {
  const [systemColorScheme, setSystemColorScheme] = useState<ResolvedColorScheme>(() =>
    getSystemColorScheme(),
  );

  useEffect(() => {
    const query = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => setSystemColorScheme(getSystemColorScheme());
    query.addEventListener("change", onChange);
    return () => query.removeEventListener("change", onChange);
  }, []);

  return systemColorScheme;
}

function getSystemColorScheme(): ResolvedColorScheme {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}
