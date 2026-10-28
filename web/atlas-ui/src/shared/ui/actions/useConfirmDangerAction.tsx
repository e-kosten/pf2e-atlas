import { ErrorNotice } from "../../feedback/ErrorNotice";
import { App, Space } from "antd";
import { FeedbackModalFrame } from "../../feedback/GlobalFeedback";
import { feedbackModalStyles } from "../overlays/AtlasModal";

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
      modalRender: (node) => <FeedbackModalFrame active>{node}</FeedbackModalFrame>,
      styles: feedbackModalStyles(),
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
                <ErrorNotice error={error} title={`${okText} could not be confirmed`} />
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
