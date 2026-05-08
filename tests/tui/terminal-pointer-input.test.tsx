import React from "react";

import { Box, Text } from "ink";
import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it } from "vitest";

import {
  DerivedTagTerminalProvider,
  TerminalTextScreen,
  TerminalTwoPaneScreen,
  useDerivedTagTerminalSize,
  useRegisterDerivedTagTerminalPointerRegion,
} from "../../src/tui/terminal-ui.js";
import { TerminalCenteredOverlayPanel } from "../../src/tui/framework/screen-components.js";

afterEach(() => {
  cleanup();
});

async function flushInkFrames(count = 2): Promise<void> {
  for (let index = 0; index < count; index += 1) {
    await new Promise((resolve) => {
      setTimeout(resolve, 0);
    });
  }
}

function PaneWheelHarness(): React.JSX.Element {
  const [lastEvent, setLastEvent] = React.useState("none");

  return (
    <TerminalTwoPaneScreen
      title="Pointer Routing"
      leftWidth={20}
      left={{
        title: "Left",
        lines: [{ text: "left pane" }],
        pointerRegion: {
          onPointerEvent: (event) => {
            if (event.kind !== "wheel") {
              return false;
            }
            setLastEvent(`left:${event.deltaY}`);
            return true;
          },
        },
      }}
      right={{
        title: "Right",
        lines: [{ text: "right pane" }],
        pointerRegion: {
          onPointerEvent: (event) => {
            if (event.kind !== "wheel") {
              return false;
            }
            setLastEvent(`right:${event.deltaY}`);
            return true;
          },
        },
      }}
      footer={[{ text: `last=${lastEvent}` }]}
    />
  );
}

function OverlayCaptureHarness(): React.JSX.Element {
  const [backgroundCount, setBackgroundCount] = React.useState(0);
  const size = useDerivedTagTerminalSize();

  useRegisterDerivedTagTerminalPointerRegion({
    rect: { x: 0, y: 0, width: size.width, height: size.height },
    priority: 0,
    onPointerEvent: (event) => {
      if (event.kind !== "wheel") {
        return false;
      }
      setBackgroundCount((current) => current + 1);
      return true;
    },
  });

  return (
    <>
      <TerminalTextScreen title="Pointer Overlay" body={[{ text: `background=${backgroundCount}` }]} />
      <TerminalCenteredOverlayPanel width={size.width} height={size.height} capturePointerEvents>
        <Box width={size.width} height={size.height} alignItems="center" justifyContent="center">
          <Text>Overlay</Text>
        </Box>
      </TerminalCenteredOverlayPanel>
    </>
  );
}

function ResizablePaneHarness(): React.JSX.Element {
  const [leftWidth, setLeftWidth] = React.useState(24);

  return (
    <TerminalTwoPaneScreen
      title="Pane Resize"
      leftWidth={leftWidth}
      resize={{ onLeftWidthChange: setLeftWidth }}
      left={{
        title: "Left",
        lines: [{ text: "left pane" }],
      }}
      right={{
        title: "Right",
        lines: [{ text: "right pane" }],
      }}
      footer={[{ text: `leftWidth=${leftWidth}` }]}
    />
  );
}

describe("terminal pointer input", () => {
  it("routes wheel events to the hovered pane region", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <PaneWheelHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("last=none");

    app.stdin.write("\u001b[<64;30;6M");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("last=right:-1");

    app.stdin.write("\u001b[<65;5;6M");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("last=left:1");
  });

  it("lets centered overlays capture wheel input before background regions", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <OverlayCaptureHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("background=0");

    app.stdin.write("\u001b[<64;40;8M");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("background=0");
  });

  it("resizes two-pane screens by dragging the separator", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <ResizablePaneHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("leftWidth=24");

    app.stdin.write("\u001b[<0;25;4M");
    await flushInkFrames();
    app.stdin.write("\u001b[<32;35;4M");
    await flushInkFrames();
    app.stdin.write("\u001b[<0;35;4m");
    await flushInkFrames();

    expect(app.lastFrame()).toContain("leftWidth=34");
  });
});
