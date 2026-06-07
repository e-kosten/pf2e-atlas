import { ConfigProvider, theme as antTheme } from "antd";
import { MantineProvider } from "@mantine/core";
import { useEffect, useState } from "react";
import { AntPrototype } from "./AntPrototype";
import { MantinePrototype } from "./MantinePrototype";
import { PrototypeShell } from "./PrototypeShell";
import { useAtlasWorkspace } from "./useAtlasWorkspace";
import type { LibraryPrototype } from "../state/searchState";

export function AtlasPrototypeApp() {
  const [activeLibrary, setActiveLibrary] = useState<LibraryPrototype>("ant");
  const [darkMode, setDarkMode] = useState(false);
  const workspace = useAtlasWorkspace();

  useEffect(() => {
    document.documentElement.dataset.theme = darkMode ? "dark" : "light";
  }, [darkMode]);

  const content =
    activeLibrary === "ant" ? (
      <ConfigProvider
        theme={{
          algorithm: darkMode
            ? antTheme.darkAlgorithm
            : antTheme.defaultAlgorithm,
          token: {
            colorPrimary: "#1f766f",
            borderRadius: 6,
            fontFamily:
              "Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, sans-serif",
          },
        }}
      >
        <AntPrototype workspace={workspace} />
      </ConfigProvider>
    ) : (
      <MantineProvider
        defaultColorScheme={darkMode ? "dark" : "light"}
        theme={{
          primaryColor: "teal",
          radius: { md: "6px" },
          fontFamily:
            "Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, sans-serif",
        }}
      >
        <MantinePrototype workspace={workspace} />
      </MantineProvider>
    );

  return (
    <PrototypeShell
      activeLibrary={activeLibrary}
      darkMode={darkMode}
      onDarkModeChange={setDarkMode}
      onLibraryChange={setActiveLibrary}
      workspace={workspace}
    >
      {content}
    </PrototypeShell>
  );
}
