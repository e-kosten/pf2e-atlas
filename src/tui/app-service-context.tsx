import React from "react";

import type { Pf2eTerminalAppServices } from "./app-services.js";

const Pf2eTerminalAppServicesContext = React.createContext<Pf2eTerminalAppServices | null>(null);

export function Pf2eTerminalAppServicesProvider({
  children,
  services,
}: {
  children: React.ReactNode;
  services: Pf2eTerminalAppServices;
}): React.JSX.Element {
  return (
    <Pf2eTerminalAppServicesContext.Provider value={services}>
      {children}
    </Pf2eTerminalAppServicesContext.Provider>
  );
}

export function usePf2eTerminalAppServices(): Pf2eTerminalAppServices {
  const services = React.useContext(Pf2eTerminalAppServicesContext);
  if (!services) {
    throw new Error("Pf2eTerminalAppServicesContext is not available.");
  }
  return services;
}
