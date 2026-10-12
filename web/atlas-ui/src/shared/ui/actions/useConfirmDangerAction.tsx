import { App } from "antd";

export function useConfirmDangerAction() {
  const { modal, message } = App.useApp();
  return ({
    content,
    okText,
    onConfirm,
    title,
  }: {
    content?: string;
    okText: string;
    onConfirm: () => void | Promise<unknown>;
    title: string;
  }) => {
    const opener =
      document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const confirmation = modal.confirm({
      title,
      content,
      okText,
      okButtonProps: { danger: true },
      cancelText: "Cancel",
      focusable: { autoFocusButton: "cancel" },
      onOk: async () => {
        try {
          await onConfirm();
        } catch (error) {
          void message.error(error instanceof Error ? error.message : "Action failed");
          throw error;
        }
      },
      afterClose: () => {
        if (opener?.isConnected) opener.focus();
      },
    });
    // Contextual Modal's awaited mode consumes rejected onOk promises while keeping it open.
    void confirmation.then(
      () => {},
      () => {},
    );
    return confirmation;
  };
}
