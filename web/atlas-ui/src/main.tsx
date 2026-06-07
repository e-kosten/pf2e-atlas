import "@mantine/core/styles.css";
import "antd/dist/reset.css";
import "./styles.css";

import React from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AtlasPrototypeApp } from "./ui/AtlasPrototypeApp";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 20_000,
    },
  },
});

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <AtlasPrototypeApp />
    </QueryClientProvider>
  </React.StrictMode>,
);
