import { ConfigProvider } from "antd";
import { Suspense, lazy, useEffect, useState } from "react";
import {
  COLOR_SCHEME_STORAGE_KEY,
  antDesignTheme,
  atlasCssVariables,
  type ColorSchemePreference,
  type ResolvedColorScheme,
} from "../shared/theme/atlasTheme";
import { AppShell } from "./AppShell";
import {
  ATLAS_ROUTE_CHANGE_EVENT,
  currentAtlasRoute,
  navigateToAtlasRoute,
  type AtlasRoute,
} from "./routes";
import { useSearchWorkspace } from "../features/search/useSearchWorkspace";

const SearchView = lazy(() =>
  import("../features/search/SearchView").then((module) => ({
    default: module.SearchView,
  })),
);
const EncounterIndexView = lazy(() =>
  import("../features/encounters/EncounterIndexView").then((module) => ({
    default: module.EncounterIndexView,
  })),
);
const EncounterDetailView = lazy(() =>
  import("../features/encounters/EncounterDetailView").then((module) => ({
    default: module.EncounterDetailView,
  })),
);
const EncounterEditView = lazy(() =>
  import("../features/encounters/EncounterEditView").then((module) => ({
    default: module.EncounterEditView,
  })),
);
const ListIndexView = lazy(() =>
  import("../features/lists/ListIndexView").then((module) => ({
    default: module.ListIndexView,
  })),
);
const ListDetailView = lazy(() =>
  import("../features/lists/ListDetailView").then((module) => ({
    default: module.ListDetailView,
  })),
);
const ListEditView = lazy(() =>
  import("../features/lists/ListEditView").then((module) => ({
    default: module.ListEditView,
  })),
);
const RecordView = lazy(() =>
  import("../features/records/RecordViews").then((module) => ({
    default: module.RecordView,
  })),
);
const ReaderView = lazy(() =>
  import("../features/records/RecordViews").then((module) => ({
    default: module.ReaderView,
  })),
);
const PresentationMocksView = lazy(() =>
  import("../features/presentation-mocks/PresentationMocksView").then((module) => ({
    default: module.PresentationMocksView,
  })),
);

export function AtlasApp() {
  const [colorScheme, setColorScheme] =
    useState<ColorSchemePreference>(readStoredColorScheme);
  const [route, setRoute] = useState<AtlasRoute>(currentAtlasRoute);
  const systemColorScheme = useSystemColorScheme();
  const resolvedColorScheme =
    colorScheme === "system" ? systemColorScheme : colorScheme;
  const workspace = useSearchWorkspace({ enabled: route.kind === "search" });

  useEffect(() => {
    document.documentElement.dataset.theme = resolvedColorScheme;
    Object.entries(atlasCssVariables(resolvedColorScheme)).forEach(
      ([property, value]) => {
        document.documentElement.style.setProperty(property, String(value));
      },
    );
  }, [resolvedColorScheme]);

  useEffect(() => {
    const onRouteChange = () => setRoute(currentAtlasRoute());
    window.addEventListener("popstate", onRouteChange);
    window.addEventListener(ATLAS_ROUTE_CHANGE_EVENT, onRouteChange);
    return () => {
      window.removeEventListener("popstate", onRouteChange);
      window.removeEventListener(ATLAS_ROUTE_CHANGE_EVENT, onRouteChange);
    };
  }, []);

  const updateColorScheme = (preference: ColorSchemePreference) => {
    setColorScheme(preference);
    localStorage.setItem(COLOR_SCHEME_STORAGE_KEY, preference);
  };

  return (
    <AppShell
      activeView={route.kind}
      colorScheme={colorScheme}
      onColorSchemeChange={updateColorScheme}
      onNavigateSearch={() =>
        navigateToAtlasRoute({ kind: "search", selectedRecordKey: null })
      }
      onNavigateEncounters={() => navigateToAtlasRoute({ kind: "encounters" })}
      onNavigateLists={() => navigateToAtlasRoute({ kind: "lists" })}
      onNavigatePresentationMocks={() =>
        navigateToAtlasRoute({ kind: "presentationMocks" })
      }
      resolvedColorScheme={resolvedColorScheme}
      workspace={workspace}
    >
      <ConfigProvider theme={antDesignTheme(resolvedColorScheme)}>
        <Suspense fallback={<RouteLoading />}>
          {route.kind === "search" && <SearchView workspace={workspace} />}
          {route.kind === "presentationMocks" && <PresentationMocksView />}
          {route.kind === "encounters" && <EncounterIndexView route={route} />}
          {route.kind === "encounter" && <EncounterDetailView route={route} />}
          {route.kind === "encounterEdit" && <EncounterEditView route={route} />}
          {route.kind === "lists" && <ListIndexView route={route} />}
          {route.kind === "list" && <ListDetailView route={route} />}
          {route.kind === "listEdit" && <ListEditView route={route} />}
          {route.kind === "record" && <RecordView route={route} />}
          {route.kind === "reader" && <ReaderView route={route} />}
        </Suspense>
      </ConfigProvider>
    </AppShell>
  );
}

function RouteLoading() {
  return <main className="detail-empty">Loading</main>;
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
