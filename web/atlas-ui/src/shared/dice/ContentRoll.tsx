import { Button, Popover } from "antd";
import { Dices, X } from "lucide-react";
import { useId, useState } from "react";
import { rollContent } from "../../api/atlasApi";
import type {
  ContentRollPresentationView,
  ContentRollResultView,
  RollContentRequest,
} from "../../generated/atlas";
import { RollResult, UnrolledRoll } from "./RollResult";
import { useDicePopoverFocus } from "./useDicePopoverFocus";
import { useDiceRoll } from "./useDiceRoll";
import "../../styles/ui/content-roll.css";

/** Transient reader dice. Reopening shows the result; only Reroll replaces it. */
export function ContentRoll({
  request,
  presentation,
}: {
  request: RollContentRequest;
  presentation: ContentRollPresentationView;
}) {
  const [open, setOpen] = useState(false);
  const headingId = useId();
  const { heading, components } = presentation;
  const roll = useDiceRoll<ContentRollResultView>({ retainResultOnError: true });
  const { trigger, content, afterOpenChange } = useDicePopoverFocus(open);
  const label = components
    .map((c) => `${c.formula}${c.label ? ` ${c.label}` : ""}`)
    .join(" + ");
  const act = () => void roll.run(() => rollContent(request));
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
        // eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions -- The focusable nonmodal roll dialog consumes Escape before its surrounding reader Drawer.
        <div
          ref={content}
          tabIndex={-1}
          role="dialog"
          aria-labelledby={headingId}
          className="atlas-dice-popover content-roll"
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              event.stopPropagation();
              setOpen(false);
            }
          }}
        >
          <div className="content-roll__toolbar">
            <strong id={headingId}>{heading}</strong>
            <Button
              type="text"
              icon={<X size={16} />}
              aria-label="Close roll"
              onClick={() => setOpen(false)}
            />
          </div>
          {components.map((part, index) => (
            <div key={index} className="content-roll__component">
              {components.length > 1 && part.label && <span>{part.label}</span>}
              {roll.result?.components[index] ? (
                <RollResult
                  outcome={roll.result.components[index].outcome}
                  unit={part.unit ?? undefined}
                  pending={roll.animating}
                />
              ) : (
                <UnrolledRoll
                  expression={part.expression}
                  unit={part.unit ?? undefined}
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
            aria-disabled={roll.busy}
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
