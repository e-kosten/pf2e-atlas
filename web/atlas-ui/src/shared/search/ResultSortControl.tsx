import {
  ArrowDownOutlined,
  ArrowUpOutlined,
  CloseOutlined,
  CheckOutlined,
  CopyOutlined,
  PlusOutlined,
  ReloadOutlined,
} from "@ant-design/icons";
import { Button, Popover, Select, Tooltip, Typography, theme } from "antd";
import type { GetRef } from "antd";
import { useRef, useState } from "react";
import type {
  ResultSort,
  SortDirection,
  SortField,
  SortRule,
} from "../../generated/atlas";
import {
  effectiveResultSort,
  INVALID_SORT_SEED,
  isResultSortError,
  parseRandomSeed,
  type SearchFormState,
} from "../filters/searchState";
import { EditableCommitField } from "../ui/forms/EditableCommitField";
import "../../styles/ui/result-sort.css";

const FIELDS: { value: SortField; label: string; short: string }[] = [
  { value: "name", label: "Name", short: "Name" },
  { value: "level", label: "Level / Rank", short: "Level" },
  { value: "unit_price", label: "Unit price", short: "Price" },
];

function newSeed(previous?: number): number {
  const seed = crypto.getRandomValues(new Uint32Array(1))[0];
  return seed === previous ? (seed + 1) >>> 0 : seed;
}

function ruleLabel(rule: SortRule): string {
  return `${FIELDS.find((field) => field.value === rule.field)!.short} ${rule.direction === "asc" ? "↑" : "↓"}`;
}

export function ResultSortControl({
  search,
  setSearch,
  surface = "search",
  disabled = false,
}: {
  search: SearchFormState;
  setSearch: (next: SearchFormState) => void;
  surface?: "search" | "saved_list";
  disabled?: boolean;
}) {
  const sort = effectiveResultSort(search, surface);
  const [open, setOpen] = useState(false);
  const trigger = useRef<HTMLButtonElement>(null);
  const fieldRefs = useRef(new Map<SortField, GetRef<typeof Select>>());
  const summary =
    sort.kind === "attributes"
      ? sort.rules.map(ruleLabel).join(", ")
      : sort.kind === "saved_order"
        ? "Saved order"
        : sort.kind === "random"
          ? "Random"
          : "Relevance";
  function commit(next?: ResultSort) {
    setSearch({
      ...search,
      sort: next,
      filterError: isResultSortError(search.filterError) ? null : search.filterError,
    });
  }
  function updateRules(rules: SortRule[]) {
    commit(rules.length ? { kind: "attributes", rules } : undefined);
  }
  function close() {
    setOpen(false);
    requestAnimationFrame(() => trigger.current?.focus({ preventScroll: true }));
  }
  const modes = [
    ...(surface === "saved_list"
      ? [{ value: "saved_order", label: "Saved order" }]
      : search.query.trim() || search.similarSeed
        ? [{ value: "relevance", label: "Relevance" }]
        : []),
    { value: "attributes", label: "By field" },
    { value: "random", label: "Random" },
  ];
  return (
    <div className="result-sort">
      <Popover
        trigger="click"
        placement="bottom"
        arrow={false}
        open={open && !disabled}
        onOpenChange={setOpen}
        styles={{ root: { maxWidth: "calc(100vw - 24px)" } }}
        content={
          <section
            className="result-sort__popover"
            role="dialog"
            aria-label="Sort results"
            onKeyDownCapture={(event) => {
              if (event.key === "Escape" && !event.defaultPrevented) {
                const target = event.target;
                // The seed editor cancels its draft before dismissing this dialog.
                // An open Select consumes its first Escape to close its own menu.
                if (
                  target instanceof HTMLInputElement &&
                  (target.closest(".result-sort__seed") ||
                    (target.getAttribute("role") === "combobox" &&
                      target.getAttribute("aria-expanded") === "true"))
                )
                  return;
                event.preventDefault();
                event.stopPropagation();
                close();
              }
            }}
          >
            <div className="result-sort__mode">
              <Typography.Text strong>Order</Typography.Text>
              <Select
                aria-label="Sort mode"
                value={
                  modes.some((mode) => mode.value === sort.kind) ? sort.kind : undefined
                }
                placeholder="Choose order"
                options={modes}
                onSelect={(kind) => {
                  if (kind === "attributes")
                    commit(
                      sort.kind === "attributes"
                        ? sort
                        : {
                            kind: "attributes",
                            rules: [{ field: "name", direction: "asc" }],
                          },
                    );
                  else if (kind === "random")
                    commit(
                      sort.kind === "random"
                        ? sort
                        : { kind: "random", seed: newSeed() },
                    );
                  else commit({ kind: kind as "relevance" | "saved_order" });
                }}
              />
            </div>
            {sort.kind === "attributes" && (
              <>
                {sort.rules.map((rule, index) => (
                  <div className="result-sort__rule" key={rule.field}>
                    <Typography.Text strong className="result-sort__rule-label">
                      {index === 0 ? "Sort by" : "Then by"}
                    </Typography.Text>
                    <Select
                      ref={(node) => {
                        if (node) fieldRefs.current.set(rule.field, node);
                        else fieldRefs.current.delete(rule.field);
                      }}
                      aria-label={`Sort field ${index + 1}`}
                      value={rule.field}
                      options={FIELDS.map((field) => ({
                        value: field.value,
                        label: field.label,
                        disabled: sort.rules.some(
                          (other, otherIndex) =>
                            otherIndex !== index && other.field === field.value,
                        ),
                      }))}
                      onSelect={(field) => {
                        updateRules(
                          sort.rules.map((other, otherIndex) =>
                            otherIndex === index ? { ...other, field } : other,
                          ),
                        );
                        requestAnimationFrame(() =>
                          fieldRefs.current.get(field)?.focus(),
                        );
                      }}
                    />
                    <Select
                      aria-label={`Sort direction ${index + 1}`}
                      value={rule.direction}
                      options={[
                        {
                          value: "asc",
                          label: rule.field === "name" ? "A–Z" : "Low to high",
                        },
                        {
                          value: "desc",
                          label: rule.field === "name" ? "Z–A" : "High to low",
                        },
                      ]}
                      onSelect={(direction: SortDirection) =>
                        updateRules(
                          sort.rules.map((other, otherIndex) =>
                            otherIndex === index ? { ...other, direction } : other,
                          ),
                        )
                      }
                    />
                    <div className="result-sort__rule-actions">
                      <Button
                        aria-label={`Move sort rule ${index + 1} up`}
                        icon={<ArrowUpOutlined />}
                        disabled={index === 0}
                        onClick={() => {
                          const rules = [...sort.rules];
                          [rules[index - 1], rules[index]] = [
                            rules[index],
                            rules[index - 1],
                          ];
                          updateRules(rules);
                        }}
                      />
                      <Button
                        aria-label={`Move sort rule ${index + 1} down`}
                        icon={<ArrowDownOutlined />}
                        disabled={index === sort.rules.length - 1}
                        onClick={() => {
                          const rules = [...sort.rules];
                          [rules[index + 1], rules[index]] = [
                            rules[index],
                            rules[index + 1],
                          ];
                          updateRules(rules);
                        }}
                      />
                      <Button
                        aria-label={`Remove sort rule ${index + 1}`}
                        icon={<CloseOutlined />}
                        onClick={() =>
                          updateRules(
                            sort.rules.filter((_, otherIndex) => otherIndex !== index),
                          )
                        }
                      />
                    </div>
                  </div>
                ))}
                <Button
                  aria-label="Add another sort"
                  icon={<PlusOutlined />}
                  disabled={sort.rules.length >= FIELDS.length}
                  onClick={() => {
                    const field = FIELDS.find(
                      (candidate) =>
                        !sort.rules.some((rule) => rule.field === candidate.value),
                    );
                    if (field)
                      updateRules([
                        ...sort.rules,
                        { field: field.value, direction: "asc" },
                      ]);
                  }}
                >
                  Add another sort
                </Button>
              </>
            )}
            {sort.kind === "random" && (
              <>
                <SeedField
                  seed={sort.seed}
                  onCommit={(seed) => commit({ kind: "random", seed })}
                  onCancel={close}
                  onShuffle={() => commit({ kind: "random", seed: newSeed(sort.seed) })}
                />
              </>
            )}
          </section>
        }
      >
        <Button
          ref={trigger}
          size="small"
          disabled={disabled}
          className="result-sort__trigger"
          aria-label={`Sort: ${summary}`}
          aria-expanded={open && !disabled}
          title={`Sort: ${summary}`}
        >
          <span>Sort:</span>
          <span className="result-sort__summary">
            {sort.kind === "attributes" ? ruleLabel(sort.rules[0]) : summary}
          </span>
          {sort.kind === "attributes" && sort.rules.length > 1 && (
            <span>+{sort.rules.length - 1}</span>
          )}
        </Button>
      </Popover>
    </div>
  );
}

function SeedField({
  seed,
  onCommit,
  onCancel,
  onShuffle,
}: {
  seed: number;
  onCommit: (seed: number) => void;
  onCancel: () => void;
  onShuffle: () => void;
}) {
  const { token } = theme.useToken();
  const [error, setError] = useState<{ seed: number; message: string } | null>(null);
  const [copyState, setCopyState] = useState<{
    seed: number;
    status: "idle" | "copied" | "failed";
    request: object | null;
  }>({ seed, status: "idle", request: null });
  // Retire feedback and pending actions even when a seed later returns to an old value.
  if (copyState.seed !== seed) setCopyState({ seed, status: "idle", request: null });
  const copyStatus = copyState.seed === seed ? copyState.status : "idle";
  async function copySeed() {
    const request = {};
    setCopyState({ seed, status: "idle", request });
    let status: "copied" | "failed";
    try {
      await navigator.clipboard.writeText(String(seed));
      status = "copied";
    } catch {
      status = "failed";
    }
    setCopyState((current) =>
      current.seed === seed && current.request === request
        ? { ...current, status }
        : current,
    );
  }
  return (
    <div className="result-sort__seed">
      <Typography.Text strong>Seed</Typography.Text>
      <div className="result-sort__seed-controls">
        <EditableCommitField
          className="result-sort__seed-input"
          ariaLabel="Random seed"
          value={String(seed)}
          inputMode="numeric"
          blurOnEnter={false}
          suffix={
            <Tooltip
              title={copyStatus === "copied" ? "Copied" : "Copy seed"}
              trigger={["hover", "focus"]}
            >
              <Button
                aria-label="Copy seed"
                type="text"
                size="small"
                disabled={error?.seed === seed}
                styles={{ root: { marginBlock: -token.lineWidth } }}
                icon={copyStatus === "copied" ? <CheckOutlined /> : <CopyOutlined />}
                onClick={() => void copySeed()}
              />
            </Tooltip>
          }
          onCancel={() => {
            setError(null);
            onCancel();
          }}
          onCommit={(value) => {
            const parsed = parseRandomSeed(value);
            if (parsed === null) {
              setError({ seed, message: INVALID_SORT_SEED });
              return;
            }
            setError(null);
            onCommit(parsed);
          }}
        />
        <Button aria-label="Shuffle" icon={<ReloadOutlined />} onClick={onShuffle}>
          Shuffle
        </Button>
      </div>
      <span className="sr-only" role="status">
        {copyStatus === "copied" ? "Seed copied" : ""}
      </span>
      {copyStatus === "failed" && (
        <Typography.Text type="danger" role="alert">
          Unable to copy seed. Select and copy it.
        </Typography.Text>
      )}
      {error?.seed === seed && (
        <Typography.Text type="danger" role="alert">
          {error.message}
        </Typography.Text>
      )}
    </div>
  );
}
