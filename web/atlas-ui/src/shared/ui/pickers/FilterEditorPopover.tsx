import { Popover, type PopoverProps } from "antd";

type FilterEditorPopoverProps = Omit<
  PopoverProps,
  "arrow" | "destroyOnHidden" | "placement" | "trigger"
>;

export function FilterEditorPopover(props: FilterEditorPopoverProps) {
  return (
    <Popover
      {...props}
      arrow={false}
      destroyOnHidden
      placement="bottom"
      trigger="click"
    />
  );
}
