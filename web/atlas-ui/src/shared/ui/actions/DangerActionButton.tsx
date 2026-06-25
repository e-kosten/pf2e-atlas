import { Button } from "antd";
import type { ButtonProps } from "antd";
import { confirmDangerAction } from "./confirmDangerAction";

type DangerActionButtonProps = Omit<ButtonProps, "danger" | "onClick"> & {
  confirmContent?: string;
  confirmOkText: string;
  confirmTitle: string;
  onBeforeConfirm?: React.MouseEventHandler<HTMLElement>;
  onConfirm: () => void;
};

export function DangerActionButton({
  confirmContent,
  confirmOkText,
  confirmTitle,
  onBeforeConfirm,
  onConfirm,
  ...buttonProps
}: DangerActionButtonProps) {
  return (
    <Button
      {...buttonProps}
      danger
      onClick={(event) => {
        onBeforeConfirm?.(event);
        confirmDangerAction({
          title: confirmTitle,
          content: confirmContent,
          okText: confirmOkText,
          onConfirm,
        });
      }}
    />
  );
}
