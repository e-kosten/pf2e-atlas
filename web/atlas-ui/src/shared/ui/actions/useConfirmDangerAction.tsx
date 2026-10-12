import { App } from "antd";

export function useConfirmDangerAction() {
  const { modal } = App.useApp();
  return ({
    content,
    okText,
    onConfirm,
    title,
  }: {
    content?: string;
    okText: string;
    onConfirm: () => void;
    title: string;
  }) => {
    const opener =
      document.activeElement instanceof HTMLElement ? document.activeElement : null;
    return modal.confirm({
      title,
      content,
      okText,
      okButtonProps: { danger: true },
      cancelText: "Cancel",
      focusable: { autoFocusButton: "cancel" },
      onOk: onConfirm,
      afterClose: () => {
        if (opener?.isConnected) opener.focus();
      },
    });
  };
}
