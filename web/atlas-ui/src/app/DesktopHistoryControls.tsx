import { Button, Tooltip } from "antd";
import { ArrowLeft, ArrowRight } from "lucide-react";
import { useSyncExternalStore } from "react";
import { useDesktopTab } from "./DesktopTabContext";
import { useDesktopAttachment, type HistoryDirection } from "./desktopBridge";
import { macHistory } from "./desktopMacHistory";
import { useAtlasRouter } from "./router";

export function DesktopHistoryControls() {
  const platform = useDesktopAttachment()?.historyNavigation;
  const tab = useDesktopTab();
  if (platform === "windows" && tab)
    return <WindowsHistoryControls active={tab.active} />;
  if (platform === "macos") return <MacHistoryControls />;
  return null;
}

function WindowsHistoryControls({ active }: { active: boolean }) {
  const { history } = useAtlasRouter();
  const back = useSyncExternalStore(history.subscribe, () => history.canGoBack());
  const forward = useSyncExternalStore(
    history.subscribe,
    () => history.location.state.__TSR_index < history.length - 1,
  );
  return (
    <HistoryButtons
      platform="windows"
      back={back && active}
      forward={forward && active}
      traverse={(direction) => history[direction]()}
    />
  );
}

function MacHistoryControls() {
  const { back, forward } = useSyncExternalStore(
    macHistory.subscribe,
    macHistory.getSnapshot,
  );
  return (
    <HistoryButtons
      platform="macos"
      back={back}
      forward={forward}
      traverse={macHistory.traverse}
    />
  );
}

function HistoryButtons({
  platform,
  back,
  forward,
  traverse,
}: {
  platform: "macos" | "windows";
  back: boolean;
  forward: boolean;
  traverse: (direction: HistoryDirection) => void;
}) {
  const mac = platform === "macos";
  return (
    <div className="topbar__history" role="group" aria-label="Tab history">
      <Tooltip
        title={mac ? "Back (⌘[)" : "Back (Alt+Left)"}
        trigger={["hover", "focus"]}
      >
        <Button
          type="text"
          aria-label="Back"
          aria-keyshortcuts={mac ? "Meta+[" : "Alt+ArrowLeft"}
          disabled={!back}
          icon={<ArrowLeft size={18} />}
          onClick={() => traverse("back")}
        />
      </Tooltip>
      <Tooltip
        title={mac ? "Forward (⌘])" : "Forward (Alt+Right)"}
        trigger={["hover", "focus"]}
      >
        <Button
          type="text"
          aria-label="Forward"
          aria-keyshortcuts={mac ? "Meta+]" : "Alt+ArrowRight"}
          disabled={!forward}
          icon={<ArrowRight size={18} />}
          onClick={() => traverse("forward")}
        />
      </Tooltip>
    </div>
  );
}
