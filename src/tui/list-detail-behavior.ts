import type { TerminalListDetailNotificationTone } from "./list-detail-presentation.js";

export const TERMINAL_LIST_DETAIL_FOCUS_POLICY = "explicit-only";

export type TerminalListDetailFocusPolicy = typeof TERMINAL_LIST_DETAIL_FOCUS_POLICY;

export type TerminalListDetailRightIntentKind = "drill" | "open" | "preview" | "none";

export type TerminalListDetailDestinationAvailability = "available" | "unavailable" | "already-satisfied";

export type TerminalListDetailDeadEndPolicy = "notify" | "noop";

export type TerminalListDetailRightBehaviorContract = {
  rightIntent: TerminalListDetailRightIntentKind;
  destination:
    | {
        availability: "available";
        perform: () => void;
      }
    | {
        availability: Exclude<TerminalListDetailDestinationAvailability, "available">;
      };
  deadEndPolicy?: TerminalListDetailDeadEndPolicy;
};

export type TerminalListDetailRightBehaviorResolution =
  | {
      kind: "success";
      focusPolicy: TerminalListDetailFocusPolicy;
      rightIntent: Exclude<TerminalListDetailRightIntentKind, "none">;
    }
  | {
      kind: "dead-end";
      availability: Exclude<TerminalListDetailDestinationAvailability, "available">;
      deadEndPolicy: TerminalListDetailDeadEndPolicy;
      focusPolicy: TerminalListDetailFocusPolicy;
      rightIntent: TerminalListDetailRightIntentKind;
      notification:
        | {
            message: string;
            tone: TerminalListDetailNotificationTone;
          }
        | null;
    };

export function getTerminalListDetailDeadEndMessage(args: {
  rightIntent: TerminalListDetailRightIntentKind;
  availability: Exclude<TerminalListDetailDestinationAvailability, "available">;
}): string {
  if (args.rightIntent === "preview" && args.availability === "already-satisfied") {
    return "Preview is already visible.";
  }
  if (args.rightIntent === "preview") {
    return "No preview is available for the focused entry.";
  }
  if (args.rightIntent === "drill") {
    return "No deeper destination is available for the focused entry.";
  }
  if (args.rightIntent === "open") {
    return "Nothing can be opened from the focused entry.";
  }
  return "No rightward action is available for the focused entry.";
}

export function resolveTerminalListDetailRightBehavior(
  contract: TerminalListDetailRightBehaviorContract,
): TerminalListDetailRightBehaviorResolution {
  const focusPolicy = TERMINAL_LIST_DETAIL_FOCUS_POLICY;

  if (contract.rightIntent !== "none" && contract.destination.availability === "available") {
    return {
      kind: "success",
      focusPolicy,
      rightIntent: contract.rightIntent,
    };
  }

  const availability = contract.destination.availability === "available" ? "unavailable" : contract.destination.availability;
  const deadEndPolicy = contract.deadEndPolicy ?? "notify";

  return {
    kind: "dead-end",
    availability,
    deadEndPolicy,
    focusPolicy,
    rightIntent: contract.rightIntent,
    notification:
      deadEndPolicy === "notify"
        ? {
            message: getTerminalListDetailDeadEndMessage({
              rightIntent: contract.rightIntent,
              availability,
            }),
            tone: "warning",
          }
        : null,
  };
}

export function applyTerminalListDetailRightBehavior(args: {
  contract: TerminalListDetailRightBehaviorContract;
  showNotification: (options: { message: string; tone?: TerminalListDetailNotificationTone }) => void;
}): TerminalListDetailRightBehaviorResolution {
  const resolution = resolveTerminalListDetailRightBehavior(args.contract);
  if (resolution.kind === "success") {
    if (args.contract.destination.availability === "available") {
      args.contract.destination.perform();
    }
    return resolution;
  }

  if (resolution.notification) {
    args.showNotification(resolution.notification);
  }
  return resolution;
}
