import "antd/dist/reset.css";
import "./styles.css";

import React from "react";
import { createRoot } from "react-dom/client";
import { QueryClientProvider } from "@tanstack/react-query";
import { AtlasApp } from "./app/AtlasApp";
import { createAtlasQueryClient } from "./app/queryClient";

const queryClient = createAtlasQueryClient();

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <AtlasApp />
    </QueryClientProvider>
  </React.StrictMode>,
);
