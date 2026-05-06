import React from "react";
import { Box, render as renderInkApp, useApp, useInput, useWindowSize } from "ink";
import { useStdout } from "ink";

import { TERMINAL_DIALOG_CONTINUE_FOOTER } from "../interaction-bindings.js";
import { createDerivedTagTerminalActionTargetState } from "../action-target.js";
import { DerivedTagTerminalContext } from "./context.js";
import { dispatchDerivedTagTerminalPointerEvent, parseDerivedTagTerminalPointerEvent } from "./pointer-events.js";
import {
  buildOptionalSelectModalOptions,
  buildSelectModalOptions,
  getSelectPromptInitialIndex,
} from "./modal-helpers.js";
import { DerivedTagTerminalModalHost } from "./modal-host.js";
import { planTerminalModalStateLayout } from "./modal-planning.js";
import { isBlankedPromptPresentation } from "./prompt-presentation.js";
import { terminalSurfaceProps } from "./theme.js";
import type {
  DerivedTagTerminalContextValue,
  DerivedTagTerminalHyperlinkSupport,
  DerivedTagTerminalPointerRegion,
  DerivedTagTerminalMultiSelectPromptResult,
  DerivedTagTerminalOptionalSelectPromptResult,
  DerivedTagTerminalSelectPromptResult,
  DerivedTagTerminalPromptSession,
  DerivedTagTerminalProviderProps,
  DialogOptions,
  MultiSelectPromptOptions,
  OptionalSelectPromptOptions,
  SelectPromptOptions,
  TerminalModalOwnership,
  TerminalModalState,
  TextPromptOptions,
} from "./types.js";

type PromptOwner = Pick<TerminalModalOwnership, "ownerKind" | "sessionId"> & {
  assertActive?: () => void;
};

function detectHyperlinkSupport(stdout: NodeJS.WriteStream): DerivedTagTerminalHyperlinkSupport {
  const env = process.env;

  if (env.FORCE_HYPERLINK === "1") {
    return "supported";
  }

  if (
    env.FORCE_HYPERLINK === "0" ||
    env.CI ||
    !stdout.isTTY ||
    env.TERM === "dumb" ||
    env.TERM_PROGRAM === "Apple_Terminal"
  ) {
    return "unsupported";
  }

  if (
    env.WT_SESSION ||
    env.KITTY_WINDOW_ID ||
    env.WEZTERM_EXECUTABLE ||
    env.DOMTERM ||
    env.TERMINAL_EMULATOR === "JetBrains-JediTerm" ||
    env.VTE_VERSION ||
    env.TERM_PROGRAM === "iTerm.app" ||
    env.TERM_PROGRAM === "WezTerm"
  ) {
    return "supported";
  }

  return "unsupported";
}

export function DerivedTagTerminalProvider({
  children,
  hyperlinkSupport,
}: DerivedTagTerminalProviderProps): React.JSX.Element {
  const { exit } = useApp();
  const { stdout } = useStdout();
  const { columns, rows } = useWindowSize();
  const [modal, setModal] = React.useState<TerminalModalState>(null);
  const modalRef = React.useRef<TerminalModalState>(null);
  const nextLeaseIdRef = React.useRef(1);
  const nextSessionIdRef = React.useRef(1);
  const capabilities = React.useMemo(
    () => ({
      hyperlinkSupport: hyperlinkSupport ?? detectHyperlinkSupport(stdout),
    }),
    [hyperlinkSupport, stdout],
  );
  const modalLayout = React.useMemo(() => planTerminalModalStateLayout(modal, columns, rows), [columns, modal, rows]);
  const overlayBackdropActive = modalLayout?.centeredPromptBackground === "overlay";
  const blankedPromptActive = isBlankedPromptPresentation(modalLayout?.centeredPromptBackground);
  const pointerRegionsRef = React.useRef<(DerivedTagTerminalPointerRegion & { order: number })[]>([]);
  const nextPointerRegionOrderRef = React.useRef(1);
  const availableRows =
    modalLayout?.centeredPromptBackground
      ? blankedPromptActive
        ? 0
        : rows
      : modalLayout?.presentation === "inline"
        ? Math.max(0, rows - modalLayout.totalHeight)
        : modalLayout
          ? 0
          : rows;
  const shouldRenderChildren = !modalLayout || !blankedPromptActive;

  const setTrackedModal = React.useCallback((next: React.SetStateAction<TerminalModalState>): void => {
    setModal((current) => {
      const resolved = typeof next === "function" ? (next as (value: TerminalModalState) => TerminalModalState)(current) : next;
      modalRef.current = resolved;
      return resolved;
    });
  }, []);

  const settlePreemptedModal = React.useCallback((activeModal: Exclude<TerminalModalState, null>): void => {
    switch (activeModal.kind) {
      case "dialog":
        activeModal.resolve();
        return;
      case "text":
        activeModal.resolve(undefined);
        return;
      case "select":
        activeModal.resolve({ kind: "cancelled" });
        return;
      case "multiselect":
        activeModal.resolve({ kind: "cancelled" });
        return;
    }
  }, []);

  const clearModalLease = React.useCallback(
    (leaseId: number): void => {
      setTrackedModal((current) => (current?.ownership.leaseId === leaseId ? null : current));
    },
    [setTrackedModal],
  );

  const claimPromptLease = React.useCallback(
    <T,>(
      owner: PromptOwner,
      createModal: (ownership: TerminalModalOwnership, resolve: (value: T) => void) => Exclude<TerminalModalState, null>,
    ): Promise<T> => {
      owner.assertActive?.();
      return new Promise<T>((resolve) => {
        const ownership: TerminalModalOwnership = {
          leaseId: nextLeaseIdRef.current++,
          ownerKind: owner.ownerKind,
          sessionId: owner.sessionId,
        };
        const nextModal = createModal(ownership, resolve);
        const displacedModal = modalRef.current;
        setTrackedModal(nextModal);
        if (displacedModal) {
          queueMicrotask(() => {
            settlePreemptedModal(displacedModal);
          });
        }
      });
    },
    [setTrackedModal, settlePreemptedModal],
  );

  const createPromptSession = React.useCallback(
    (owner: PromptOwner): DerivedTagTerminalPromptSession => {
      const pauseForAnyKey = async (message: string): Promise<void> => {
        await claimPromptLease<void>(owner, (ownership, resolve) => ({
          ownership,
          kind: "dialog",
          options: {
            title: "Derived-Tag Workbench",
            body: message.split("\n").map((line) => ({ text: line })),
            footer: [{ text: TERMINAL_DIALOG_CONTINUE_FOOTER, tone: "dim" }],
          },
          resolve: () => resolve(),
        }));
      };

      const promptOptionalSelectOption = async <T,>(
        options: OptionalSelectPromptOptions<T>,
      ): Promise<DerivedTagTerminalOptionalSelectPromptResult<T>> => {
        const modalOptions = buildOptionalSelectModalOptions(options);
        return claimPromptLease(owner, (ownership, resolve) => ({
          ownership,
          kind: "select",
          options: modalOptions,
          selectedIndex: getSelectPromptInitialIndex(modalOptions.entries, options.selectedValue),
          filterText: "",
          filterMode: false,
          actionTargetState: createDerivedTagTerminalActionTargetState(),
          resolve: resolve as (
            value:
              | DerivedTagTerminalSelectPromptResult<unknown>
              | DerivedTagTerminalOptionalSelectPromptResult<unknown>,
          ) => void,
        }));
      };

      const promptMultiSelectOption = async <T extends string>(
        options: MultiSelectPromptOptions<T>,
      ): Promise<DerivedTagTerminalMultiSelectPromptResult<T>> => {
        const selectedIndex = Math.max(0, options.entries.findIndex((entry) => options.selectedValues?.includes(entry.value)));
        return claimPromptLease(owner, (ownership, resolve) => ({
          ownership,
          kind: "multiselect",
          options: options as MultiSelectPromptOptions<string>,
          selectedIndex,
          filterText: "",
          filterMode: false,
          selectedValues: options.selectedValues ? [...options.selectedValues] : [],
          resolve: resolve as (value: DerivedTagTerminalMultiSelectPromptResult<string>) => void,
        }));
      };

      const promptSelectOption = async <T,>(
        options: SelectPromptOptions<T>,
      ): Promise<DerivedTagTerminalSelectPromptResult<T>> => {
        const modalOptions = buildSelectModalOptions(options);
        return claimPromptLease(owner, (ownership, resolve) => ({
          ownership,
          kind: "select",
          options: modalOptions,
          selectedIndex: getSelectPromptInitialIndex(modalOptions.entries, options.selectedValue),
          filterText: "",
          filterMode: false,
          actionTargetState: createDerivedTagTerminalActionTargetState(),
          resolve: resolve as (
            value:
              | DerivedTagTerminalSelectPromptResult<unknown>
              | DerivedTagTerminalOptionalSelectPromptResult<unknown>,
          ) => void,
        }));
      };

      const promptTextInput = async (options: TextPromptOptions): Promise<string | undefined> =>
        claimPromptLease(owner, (ownership, resolve) => ({
          ownership,
          kind: "text",
          options,
          value: options.defaultValue ?? "",
          resolve,
        }));

      const showDialog = async (options: DialogOptions): Promise<void> => {
        await claimPromptLease<void>(owner, (ownership, resolve) => ({
          ownership,
          kind: "dialog",
          options,
          resolve: () => resolve(),
        }));
      };

      return {
        pauseForAnyKey,
        promptOptionalSelectOption,
        promptMultiSelectOption,
        promptSelectOption,
        promptTextInput,
        showDialog,
      };
    },
    [claimPromptLease],
  );
  const promptSession = React.useMemo(() => createPromptSession({ ownerKind: "shared", sessionId: null }), [createPromptSession]);

  const registerPointerRegion = React.useCallback((region: DerivedTagTerminalPointerRegion): (() => void) => {
    const order = nextPointerRegionOrderRef.current++;
    const entry = { ...region, order };
    pointerRegionsRef.current = [...pointerRegionsRef.current, entry];

    return () => {
      pointerRegionsRef.current = pointerRegionsRef.current.filter((candidate) => candidate.order !== order);
    };
  }, []);

  React.useEffect(() => {
    if (!stdout.isTTY || stdout !== process.stdout) {
      return undefined;
    }

    stdout.write("\u001b[?1000h\u001b[?1006h");
    return () => {
      stdout.write("\u001b[?1000l\u001b[?1006l");
    };
  }, [stdout]);

  useInput((input) => {
    const pointerEvent = parseDerivedTagTerminalPointerEvent(input);
    if (!pointerEvent) {
      return;
    }

    dispatchDerivedTagTerminalPointerEvent(pointerRegionsRef.current, pointerEvent);
  });

  const runPromptSession = React.useCallback(
    async <T,>(runner: (session: DerivedTagTerminalPromptSession) => Promise<T>): Promise<T> => {
      const sessionId = nextSessionIdRef.current++;
      const sessionState = { active: true };
      const session = createPromptSession({
        ownerKind: "session",
        sessionId,
        assertActive: () => {
          if (!sessionState.active) {
            throw new Error(`Prompt session ${sessionId} is no longer active.`);
          }
        },
      });

      try {
        return await runner(session);
      } finally {
        sessionState.active = false;
        const activeModal = modalRef.current;
        if (
          activeModal &&
          activeModal.ownership.ownerKind === "session" &&
          activeModal.ownership.sessionId === sessionId
        ) {
          clearModalLease(activeModal.ownership.leaseId);
          settlePreemptedModal(activeModal);
        }
      }
    },
    [clearModalLease, createPromptSession, settlePreemptedModal],
  );

  const createContextValue = React.useCallback(
    ({
      backdropActive,
      layoutHeight,
    }: {
      backdropActive: boolean;
      layoutHeight: number;
    }): DerivedTagTerminalContextValue => ({
      capabilities,
      backdropActive,
      exitApp: exit,
      getLayoutHeight: () => layoutHeight,
      getLayoutWidth: () => columns,
      getViewportHeight: () => rows,
      getViewportWidth: () => columns,
      modalActive: modal !== null,
      runPromptSession,
      pauseForAnyKey: promptSession.pauseForAnyKey,
      registerPointerRegion,
      promptOptionalSelectOption: promptSession.promptOptionalSelectOption,
      promptMultiSelectOption: promptSession.promptMultiSelectOption,
      promptSelectOption: promptSession.promptSelectOption,
      promptTextInput: promptSession.promptTextInput,
      showDialog: promptSession.showDialog,
    }),
    [capabilities, columns, exit, modal, promptSession, registerPointerRegion, rows, runPromptSession],
  );

  const modalContextValue = React.useMemo<DerivedTagTerminalContextValue>(
    () =>
      createContextValue({
        backdropActive: false,
        layoutHeight: rows,
      }),
    [createContextValue, rows],
  );
  const backgroundContextValue = React.useMemo<DerivedTagTerminalContextValue>(
    () =>
      createContextValue({
        backdropActive: overlayBackdropActive,
        layoutHeight: availableRows,
      }),
    [availableRows, createContextValue, overlayBackdropActive],
  );

  return (
    <>
      <DerivedTagTerminalContext.Provider value={backgroundContextValue}>
        <Box flexDirection="column" width={columns} height={rows} {...terminalSurfaceProps("app")}>
          <Box flexDirection="column" width={columns} height={shouldRenderChildren ? rows : 0}>
            {children}
          </Box>
        </Box>
      </DerivedTagTerminalContext.Provider>
      <DerivedTagTerminalContext.Provider value={modalContextValue}>
        {modal && modalLayout ? (
          <DerivedTagTerminalModalHost
            modal={modal}
            setModal={setTrackedModal}
            exitApp={exit}
            width={columns}
            layout={modalLayout}
          />
        ) : null}
      </DerivedTagTerminalContext.Provider>
    </>
  );
}

export async function runDerivedTagTerminalApp(node: React.ReactElement): Promise<void> {
  const instance = renderInkApp(<DerivedTagTerminalProvider>{node}</DerivedTagTerminalProvider>, {
    alternateScreen: true,
    exitOnCtrlC: false,
    patchConsole: true,
  });

  try {
    await instance.waitUntilExit();
  } finally {
    instance.cleanup();
  }
}
