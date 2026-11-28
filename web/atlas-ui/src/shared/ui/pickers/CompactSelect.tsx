import { Select, theme, type SelectProps } from "antd";

type CompactOption = { label: string; value: string };

/** Intrinsic width for a small, fixed set of single-choice text options. */
export function CompactSelect({
  className,
  options,
  ...props
}: Omit<SelectProps<string, CompactOption>, "mode" | "showSearch" | "options"> & {
  options: CompactOption[];
}) {
  const { token } = theme.useToken();
  return (
    <div className={["compact-select", className].filter(Boolean).join(" ")}>
      <div
        className="compact-select__measure"
        aria-hidden="true"
        style={{
          paddingInline: token.controlPaddingHorizontal,
          paddingInlineEnd:
            token.controlPaddingHorizontal + token.fontSizeIcon + token.marginXS,
        }}
      >
        {options.map(({ label, value }) => (
          <div key={value}>{label}</div>
        ))}
      </div>
      <Select {...props} options={options} style={{ width: "100%", ...props.style }} />
    </div>
  );
}
