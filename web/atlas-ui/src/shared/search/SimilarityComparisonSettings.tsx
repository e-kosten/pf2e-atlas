import { useId } from "react";
import { Form, Select, InputNumber, Slider, Typography } from "antd";
import type { SimilarityPreferences } from "./searchPreferences";
import { SearchSettingLabel } from "./SearchSettingLabel";

export function SimilarityComparisonSettings({
  value,
  onChange,
  trait,
  methodHelp,
  selectedCount,
  identitySelected,
  invalidDecay,
}: {
  value: SimilarityPreferences;
  onChange: (value: SimilarityPreferences) => void;
  trait: boolean;
  methodHelp: string;
  selectedCount: number;
  identitySelected: boolean;
  invalidDecay: boolean;
}) {
  const id = useId();
  const usesParts = value.method !== "baseline";
  return (
    <>
      <Form.Item
        label={<SearchSettingLabel label="Comparison method" help={methodHelp} />}
        htmlFor={`${id}-method`}
      >
        <Select
          id={`${id}-method`}
          aria-label="Comparison method"
          value={value.method}
          options={[
            {
              value: "passages",
              label: "Passage matches",
            },
            {
              value: "content-average",
              label: "Content average",
            },
            {
              value: "baseline",
              label: trait ? "Trait definition" : "Name and identity",
            },
          ]}
          onChange={(method) => onChange({ ...value, method })}
        />
      </Form.Item>
      {usesParts && !trait && (
        <Form.Item
          label={
            <SearchSettingLabel
              label="Section matching"
              help={
                value.candidateParts === "corresponding"
                  ? "Only sections with matching purposes can contribute, including across record kinds. For example, descriptions compare with descriptions. If a result lacks that type of section, it contributes no match."
                  : "Any selected section can match any content in a result, even when the sections serve different purposes."
              }
            />
          }
          htmlFor={`${id}-candidate`}
        >
          <Select
            id={`${id}-candidate`}
            aria-label="Section matching"
            value={value.candidateParts}
            options={[
              { value: "any", label: "Any section" },
              {
                value: "corresponding",
                label: "Corresponding sections",
              },
            ]}
            onChange={(candidateParts) =>
              onChange({
                ...value,
                candidateParts,
              })
            }
          />
        </Form.Item>
      )}
      {value.method === "passages" && (
        <Form.Item
          label={
            <SearchSettingLabel
              label="Match decay"
              help="Sections are ordered by match strength for each result. 0 uses only the strongest section; 1 gives all selected sections equal weight. Values in between give each additional section less weight."
            />
          }
          validateStatus={invalidDecay ? "error" : undefined}
          help={invalidDecay ? "Enter a number from 0 through 1." : undefined}
        >
          <div className="search-settings__decay">
            <Slider
              aria-label="Match decay slider"
              min={0}
              max={1}
              step={0.05}
              value={invalidDecay ? 0 : value.decay}
              onChange={(decay) =>
                onChange({
                  ...value,
                  decay,
                })
              }
            />
            <InputNumber
              aria-label="Match decay"
              min={0}
              max={1}
              step={0.05}
              value={Number.isFinite(value.decay) ? value.decay : null}
              onInput={(text) => {
                onChange({
                  ...value,
                  decay: text.trim() ? Number(text) : Number.NaN,
                });
              }}
              onChange={(decay) => {
                onChange({
                  ...value,
                  decay: typeof decay === "number" ? decay : Number.NaN,
                });
              }}
            />
          </div>
          <div className="search-settings__scale">
            <span>{identitySelected ? "Strongest match" : "Strongest section"}</span>
            <span>
              {identitySelected ? "All selections equally" : "All sections equally"}
            </span>
          </div>
          {!invalidDecay && (
            <Typography.Text type="secondary">
              {selectedCount === 1
                ? identitySelected
                  ? "One selection; decay has no effect."
                  : "One section selected; decay has no effect."
                : `Contributions: 1, ${Number(value.decay.toFixed(4))}, ${Number((value.decay ** 2).toFixed(4))}…`}
            </Typography.Text>
          )}
        </Form.Item>
      )}
    </>
  );
}
