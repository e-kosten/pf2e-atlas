import { ConfigProvider } from "antd";
import { MantineProvider } from "@mantine/core";
import { useEffect, useState } from "react";
import {
  COLOR_SCHEME_STORAGE_KEY,
  antDesignTheme,
  atlasCssVariables,
  mantineTheme,
  type ColorSchemePreference,
  type ResolvedColorScheme,
} from "./atlasTheme";
import { AntPrototype } from "./AntPrototype";
import { MantinePrototype } from "./MantinePrototype";
import { PrototypeShell } from "./PrototypeShell";
import { useAtlasWorkspace } from "./useAtlasWorkspace";
import type { LibraryPrototype } from "../state/searchState";

export function AtlasPrototypeApp() {
  const [activeLibrary, setActiveLibrary] = useState<LibraryPrototype>("ant");
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

  const content =
    activeLibrary === "ant" ? (
      <ConfigProvider theme={antDesignTheme(resolvedColorScheme)}>
        <AntPrototype workspace={workspace} />
      </ConfigProvider>
    ) : (
      <MantineProvider
        forceColorScheme={resolvedColorScheme}
        theme={mantineTheme(resolvedColorScheme)}
      >
        <MantinePrototype workspace={workspace} />
      </MantineProvider>
    );

  return (
    <PrototypeShell
      activeLibrary={activeLibrary}
      colorScheme={colorScheme}
      onColorSchemeChange={updateColorScheme}
      onLibraryChange={setActiveLibrary}
      resolvedColorScheme={resolvedColorScheme}
      workspace={workspace}
    >
      {content}
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
