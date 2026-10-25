import { usePageTitle } from "../shared/browser/usePageTitle";
import { ConfigProvider, Alert, App } from "antd";
import { Activity, Suspense, lazy, useEffect, useLayoutEffect, useState } from "react";
import { Outlet, useRouterState } from "@tanstack/react-router";
import {
  COLOR_SCHEME_STORAGE_KEY,
  DARK_NEUTRAL_LEVEL_SESSION_KEY,
  DARK_TEXT_PALETTE_SESSION_KEY,
  applyAtlasDocumentTheme,
  atlasPopoverStyles,
  antDesignTheme,
  type ColorSchemePreference,
  type DarkNeutralLevel,
  type DarkTextPalette,
  type ResolvedColorScheme,
} from "../shared/theme/atlasTheme";
import {
  DEFAULT_DARK_NEUTRAL_LEVEL,
  DEFAULT_DARK_TEXT_PALETTE,
  isDarkNeutralLevel,
  isDarkTextPalette,
} from "../shared/theme/themeDefinitions";
import { AppShell } from "./AppShell";
import { LinkPickerContext } from "../shared/filters/LinkPickerContext";
import { LinkConditionPicker } from "../shared/filters/LinkConditionPicker";

import { useSearchWorkspace } from "../features/search/useSearchWorkspace";
import { useQuery } from "@tanstack/react-query";
import { getReadiness } from "../api/atlasApi";
import {
  SearchProfileContext,
  useSearchProfileSelection,
} from "../shared/search/SearchProfileContext";
const SearchView = lazy(() =>
  import("../features/search/SearchView").then((module) => ({
    default: module.SearchView,
  })),
);

export function AtlasLayout() {
  const [colorScheme, setColorScheme] =
    useState<ColorSchemePreference>(readStoredColorScheme);
  const [darkNeutralLevel, setDarkNeutralLevel] = useState<DarkNeutralLevel>(
    readStoredDarkNeutralLevel,
  );
  const [darkTextPalette, setDarkTextPalette] = useState<DarkTextPalette>(
    readStoredDarkTextPalette,
  );
  const section = useRouterState({
    select: (state) =>
      [...state.matches].reverse().find((match) => match.staticData.section)?.staticData
        .section ?? "search",
  });
  const systemColorScheme = useSystemColorScheme();
  const resolvedColorScheme =
    colorScheme === "system" ? systemColorScheme : colorScheme;
  const readiness = useQuery({ queryKey: ["readiness"], queryFn: getReadiness });
  const searchProfile = useSearchProfileSelection(readiness.data);
  const workspace = useSearchWorkspace({
    enabled: section === "search",
    searchProfile: searchProfile.id,
    pendingUrlProfile: searchProfile.pendingUrlProfile,
    searchProfileStored: searchProfile.storedVectors,
    searchProfileUsable: searchProfile.usable,
    includeSearchProfileInUrl:
      searchProfile.profiles.filter((profile) => profile.stored_vectors).length !== 1 ||
      !searchProfile.storedVectors,
  });
  useLayoutEffect(() => {
    applyAtlasDocumentTheme(
      document,
      resolvedColorScheme,
      darkNeutralLevel,
      darkTextPalette,
    );
  }, [resolvedColorScheme, darkNeutralLevel, darkTextPalette]);
  const updateColorScheme = (preference: ColorSchemePreference) => {
    setColorScheme(preference);
    try {
      localStorage.setItem(COLOR_SCHEME_STORAGE_KEY, preference);
    } catch {
      // Browser preferences are optional, including when storage is unavailable.
    }
  };
  const updateDarkNeutralLevel = (level: DarkNeutralLevel) => {
    setDarkNeutralLevel(level);
    try {
      sessionStorage.setItem(DARK_NEUTRAL_LEVEL_SESSION_KEY, level);
    } catch {
      // A temporary comparison remains usable when storage is unavailable.
    }
  };
  const updateDarkTextPalette = (palette: DarkTextPalette) => {
    setDarkTextPalette(palette);
    try {
      sessionStorage.setItem(DARK_TEXT_PALETTE_SESSION_KEY, palette);
    } catch {
      // A temporary comparison remains usable when storage is unavailable.
    }
  };
  return (
    <ConfigProvider
      theme={antDesignTheme(resolvedColorScheme, darkNeutralLevel, darkTextPalette)}
      popover={{
        styles: atlasPopoverStyles(resolvedColorScheme, darkNeutralLevel),
      }}
      select={{
        classNames: {
          popup: { root: "atlas-select-popup", listItem: "atlas-select-option" },
        },
      }}
    >
      <App component={false}>
        <SearchProfileContext value={searchProfile}>
          <LinkPickerContext value={LinkConditionPicker}>
            <AppShell
              activeView={section}
              colorScheme={colorScheme}
              onColorSchemeChange={updateColorScheme}
              darkNeutralStudy={{
                value: darkNeutralLevel,
                onChange: updateDarkNeutralLevel,
              }}
              darkTextPaletteStudy={{
                value: darkTextPalette,
                onChange: updateDarkTextPalette,
              }}
              resolvedColorScheme={resolvedColorScheme}
              readiness={readiness.data}
              readinessError={readiness.error}
              checkingReadiness={readiness.isFetching}
              onCheckReadiness={() => void readiness.refetch()}
            >
              <>
                <Activity mode={section === "search" ? "visible" : "hidden"}>
                  <Suspense fallback={<main className="detail-empty">Loading</main>}>
                    <SearchView workspace={workspace} />
                  </Suspense>
                </Activity>
                <Outlet />
              </>
            </AppShell>
          </LinkPickerContext>
        </SearchProfileContext>
      </App>
    </ConfigProvider>
  );
}
export function InvalidRoute({ message }: { message: string }) {
  usePageTitle("Invalid link");
  return <Alert type="error" title={message} />;
}
function readStoredColorScheme(): ColorSchemePreference {
  try {
    const value = localStorage.getItem(COLOR_SCHEME_STORAGE_KEY);
    return value === "light" || value === "dark" || value === "system"
      ? value
      : "system";
  } catch {
    return "system";
  }
}
function readStoredDarkNeutralLevel(): DarkNeutralLevel {
  try {
    const value = sessionStorage.getItem(DARK_NEUTRAL_LEVEL_SESSION_KEY);
    return isDarkNeutralLevel(value) ? value : DEFAULT_DARK_NEUTRAL_LEVEL;
  } catch {
    return DEFAULT_DARK_NEUTRAL_LEVEL;
  }
}
function readStoredDarkTextPalette(): DarkTextPalette {
  try {
    const value = sessionStorage.getItem(DARK_TEXT_PALETTE_SESSION_KEY);
    return isDarkTextPalette(value) ? value : DEFAULT_DARK_TEXT_PALETTE;
  } catch {
    return DEFAULT_DARK_TEXT_PALETTE;
  }
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
