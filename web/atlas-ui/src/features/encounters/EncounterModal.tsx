import { AtlasModal, type AtlasModalProps } from "../../shared/ui/overlays/AtlasModal";
import { createContext, useContext, type ReactNode, type RefObject } from "react";

const WorkArea = createContext<RefObject<HTMLDivElement | null> | null>(null);
export function EncounterModalProvider({
  area,
  children,
}: {
  area: RefObject<HTMLDivElement | null>;
  children: ReactNode;
}) {
  return <WorkArea.Provider value={area}>{children}</WorkArea.Provider>;
}
export function EncounterModal({
  layout = "form",
  ...props
}: Omit<AtlasModalProps, "workArea" | "layout"> & { layout?: "form" | "document" }) {
  const area = useContext(WorkArea);
  return (
    <AtlasModal
      {...props}
      layout={layout}
      workArea={
        layout === "form" && area
          ? { root: area, selector: ".workspace-grid" }
          : undefined
      }
    />
  );
}
