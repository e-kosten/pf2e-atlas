import { DownOutlined, UndoOutlined } from "@ant-design/icons";
import { Button, Dropdown, Space, Switch, Tooltip } from "antd";
import { useLayoutEffect, useRef, useState } from "react";
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";
import type {
  EncounterAbilityStateAction,
  EncounterAbilityStateView,
} from "../../generated/atlas";

export type AbilityStateHandler = (
  ability: EncounterAbilityStateView,
  action: EncounterAbilityStateAction,
) => Promise<boolean>;

export function EncounterAbilityControls({
  ability,
  disabled,
  onChange,
}: {
  ability: EncounterAbilityStateView;
  disabled: boolean;
  onChange?: AbilityStateHandler;
}) {
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string>();
  const inFlight = useRef(false);
  const container = useRef<HTMLDivElement>(null);
  const choiceTrigger = useRef<HTMLButtonElement>(null);
  const [choiceOpen, setChoiceOpen] = useState(false);
  const restoreFocus = useRef<HTMLElement | null>(null);
  useLayoutEffect(() => {
    if (pending || disabled || !restoreFocus.current) return;
    const target = restoreFocus.current;
    restoreFocus.current = null;
    if (
      target.isConnected &&
      container.current?.contains(target) &&
      document.activeElement === document.body
    )
      (target.hasAttribute("disabled")
        ? container.current.querySelector<HTMLElement>(
            "button:not(:disabled),input:not(:disabled)",
          )
        : target
      )?.focus({ preventScroll: true });
  }, [pending, disabled]);
  const state = ability.state;
  const effectLabel =
    state.kind === "option" && state.active
      ? (state.choices.find((choice) => choice.value === state.selection)?.label ?? "—")
      : "Off";
  const unavailable = disabled || pending || !onChange;
  const update = async (action: EncounterAbilityStateAction) => {
    if (inFlight.current || unavailable || !onChange) return;
    const focused = document.activeElement;
    restoreFocus.current =
      focused instanceof HTMLElement && container.current?.contains(focused)
        ? focused
        : null;
    inFlight.current = true;
    setPending(true);
    setError(undefined);
    try {
      if (!(await onChange(ability, action)))
        setError("Could not confirm the update. Refresh the encounter.");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Could not update ability.");
    } finally {
      inFlight.current = false;
      setPending(false);
    }
  };
  return (
    <div
      ref={container}
      className="encounter-ability-controls"
      aria-label={ability.label}
      aria-busy={pending}
    >
      <Space wrap align="center">
        {state.kind === "option" ? (
          <>
            {!state.always_active && state.choices.length === 0 && (
              <Switch
                aria-label={`${ability.label} active`}
                checked={state.active}
                checkedChildren="On"
                unCheckedChildren="Off"
                disabled={disabled || !onChange}
                loading={pending}
                onChange={(active) =>
                  void update({
                    kind: "set_option",
                    active,
                    selection: state.selection,
                  })
                }
              />
            )}
            {state.choices.length > 0 && (
              <div className="encounter-ability-controls__choice-label">
                <span>Effect</span>
                <Dropdown
                  // eslint-disable-next-line jsx-a11y/no-autofocus -- Focus enters the menu only after the user opens it.
                  autoFocus
                  trigger={["click"]}
                  open={choiceOpen && !unavailable}
                  onOpenChange={setChoiceOpen}
                  disabled={unavailable}
                  menu={{
                    "aria-label": `${ability.label} effect`,
                    onKeyDown: (event) => {
                      if (event.key === "Escape") {
                        event.preventDefault();
                        event.stopPropagation();
                        restoreOverlayFocus(choiceTrigger.current, event.currentTarget);
                        setChoiceOpen(false);
                        choiceTrigger.current?.focus({ preventScroll: true });
                      }
                    },
                    selectable: true,
                    selectedKeys: [state.active ? (state.selection ?? "") : "__off"],
                    items: [
                      ...(!state.always_active ? [{ key: "__off", label: "Off" }] : []),
                      ...state.choices.map((choice) => ({
                        key: choice.value,
                        label: `${choice.label}${choice.consequence ? ` · ${choice.consequence}` : ""}`,
                      })),
                    ],
                    onClick: ({ key }) => {
                      setChoiceOpen(false);
                      choiceTrigger.current?.focus({ preventScroll: true });
                      void update({
                        kind: "set_option",
                        active: key !== "__off",
                        selection: key === "__off" ? state.selection : key,
                      });
                    },
                  }}
                >
                  <Button
                    ref={choiceTrigger}
                    type="text"
                    icon={<DownOutlined />}
                    iconPlacement="end"
                    loading={pending ? { delay: 300 } : false}
                    disabled={unavailable}
                    aria-label={`${ability.label} effect: ${effectLabel}`}
                    aria-haspopup="menu"
                    aria-expanded={choiceOpen && !unavailable}
                    className="encounter-ability-controls__choice"
                    onKeyDown={(event) => {
                      if (event.key === "ArrowDown") {
                        event.preventDefault();
                        setChoiceOpen(true);
                      }
                    }}
                  >
                    {effectLabel}
                  </Button>
                </Dropdown>
              </div>
            )}
          </>
        ) : (
          <>
            <span className="encounter-ability-controls__remaining">
              {String(state.remaining)} of {String(state.maximum)} · per{" "}
              {state.interval}
            </span>
            <Button
              className="encounter-ability-controls__use"
              aria-label={`Use ${ability.label}`}
              disabled={unavailable || state.remaining === 0n}
              loading={pending ? { delay: 300 } : false}
              onClick={() => void update({ kind: "spend_use" })}
            >
              Use
            </Button>
            <Tooltip title="Restore one use">
              <Button
                type="text"
                icon={<UndoOutlined />}
                aria-label={`Restore use of ${ability.label}`}
                disabled={unavailable || state.remaining >= state.maximum}
                onClick={() => void update({ kind: "restore_use" })}
              />
            </Tooltip>
          </>
        )}
      </Space>
      {error && (
        <span className="encounter-inline-error" role="alert">
          {error}
        </span>
      )}
    </div>
  );
}
