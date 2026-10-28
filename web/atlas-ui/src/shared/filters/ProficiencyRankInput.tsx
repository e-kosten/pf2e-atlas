import { Select } from "antd";

const RANKS = ["Untrained", "Trained", "Expert", "Master", "Legendary"];

/** Named UI choices preserve the query contract's numeric rank values. */
export function ProficiencyRankInput({
  label,
  value,
  range,
  onChange,
}: {
  label: string;
  value: unknown;
  range: boolean;
  onChange: (value: unknown) => void;
}) {
  const select = (name: string, selected: unknown, change: (value: string) => void) => {
    const current = selected == null || selected === "" ? undefined : String(selected);
    const options = RANKS.map((name, rank) => ({ value: String(rank), label: name }));
    if (current !== undefined && !options.some((option) => option.value === current))
      options.push({ value: current, label: `Rank ${current}` });
    return (
      <Select
        virtual={false}
        className="filter-value-control"
        aria-label={name}
        placeholder="Choose a rank…"
        allowClear
        value={current}
        options={options}
        onChange={(next) => change(next ?? "")}
      />
    );
  };
  if (!range) return select(label, value, onChange);
  const bounds = Array.isArray(value) ? value : ["", ""];
  return (
    <div className="filter-range">
      {select(label + " minimum", bounds[0], (next) => onChange([next, bounds[1]]))}
      <span>to</span>
      {select(label + " maximum", bounds[1], (next) => onChange([bounds[0], next]))}
    </div>
  );
}
