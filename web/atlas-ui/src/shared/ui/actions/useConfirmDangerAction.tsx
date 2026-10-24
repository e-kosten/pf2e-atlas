import { Alert, App, Space } from "antd";

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
      mask: { closable: true },
      focusable: { autoFocusButton: "cancel" },
      onOk: async () => {
        confirmation.update({
          cancelButtonProps: { disabled: true },
          keyboard: false,
          mask: { closable: false },
        });
        try {
          await onConfirm();
        } catch (error) {
          confirmation.update({
            cancelButtonProps: { disabled: false },
            keyboard: true,
            mask: { closable: true },
            content: (
              <Space orientation="vertical">
                {content && <div>{content}</div>}
                <Alert
                  type="error"
                  title={
                    error instanceof Error
                      ? error.message
                      : `${okText} failed. Try again.`
                  }
                />
              </Space>
            ),
          });
          throw error;
        }
      },
      afterClose: () => {
        if (opener?.isConnected) opener.focus({ preventScroll: true });
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
