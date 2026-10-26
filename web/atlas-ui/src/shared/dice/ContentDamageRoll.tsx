import { Button, Popover } from "antd";
import { Dices, X } from "lucide-react";
import { useState } from "react";
import { rollContentDamage } from "../../api/atlasApi";
import type {
  ContentDamageComponentView,
  ContentDamageRollView,
  RollContentDamageRequest,
} from "../../generated/atlas";
import { RollResult, UnrolledRoll } from "./RollResult";
import { useDicePopoverFocus } from "./useDicePopoverFocus";
import { useDiceRoll } from "./useDiceRoll";
import "../../styles/ui/content-damage-roll.css";

/** Transient reader dice. Reopening shows the result; only Reroll replaces it. */
export function ContentDamageRoll({
  request,
  components,
}: {
  request: RollContentDamageRequest;
  components: ContentDamageComponentView[];
}) {
  const [open, setOpen] = useState(false);
  const roll = useDiceRoll<ContentDamageRollView>({ retainResultOnError: true });
  const { trigger, content, afterOpenChange } = useDicePopoverFocus(open);
  const label = components.map((c) => `${c.formula} ${c.label}`).join(" + ");
  const act = () => void roll.run(() => rollContentDamage(request));
  return (
    <Popover
      trigger="click"
      placement="bottom"
      open={open}
      afterOpenChange={afterOpenChange}
      onOpenChange={(next) => {
        setOpen(next);
        if (next && !roll.result && !roll.error && !roll.busy) act();
      }}
      content={
        <div
          ref={content}
          tabIndex={-1}
          role="dialog"
          aria-label={label}
          className="content-damage-roll"
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              event.stopPropagation();
              setOpen(false);
            }
          }}
        >
          <div className="content-damage-roll__toolbar">
            <strong>Roll result</strong>
            <Button
              type="text"
              icon={<X size={16} />}
              aria-label="Close roll"
              onClick={() => setOpen(false)}
            />
          </div>
          {components.map((part, index) => (
            <div key={index} className="content-damage-roll__component">
              <span>{part.label}</span>
              {roll.result?.components[index] ? (
                <RollResult
                  outcome={roll.result.components[index].outcome}
                  pending={roll.animating}
                />
              ) : (
                <UnrolledRoll
                  expression={part.expression}
                  formula={part.formula}
                  pending={roll.busy}
                >
                  <span aria-hidden="true">…</span>
                </UnrolledRoll>
              )}
            </div>
          ))}
          {roll.error && (
            <span role="alert">
              {roll.error}
              {roll.result && " Previous result shown."}
            </span>
          )}
          <Button
            disabled={roll.busy}
            loading={roll.busy && !roll.animating ? { delay: 300 } : false}
            onClick={act}
          >
            {roll.result ? "Reroll" : roll.error ? "Retry" : "Roll"}
          </Button>
        </div>
      }
    >
      <Button
        ref={trigger}
        type="text"
        size="small"
        icon={<Dices size={15} />}
        aria-label={`${roll.result ? "View roll for" : "Roll"} ${label}`}
        title={`${roll.result ? "View roll for" : "Roll"} ${label}`}
      />
    </Popover>
  );
}
