import { Modal } from "antd";
import { useCallback, useState } from "react";

export type DangerActionConfirmation = {
  content?: string;
  okText: string;
  onConfirm: () => void;
  title: string;
};

export function useConfirmDangerAction() {
  const [confirmation, setConfirmation] = useState<DangerActionConfirmation | null>(
    null,
  );
  const confirmDangerAction = useCallback(
    (nextConfirmation: DangerActionConfirmation) => {
      setConfirmation(nextConfirmation);
    },
    [],
  );
  const confirmationModal = (
    <Modal
      cancelText="Cancel"
      closable={false}
      destroyOnHidden
      maskClosable={false}
      okButtonProps={{ danger: true }}
      okText={confirmation?.okText}
      onCancel={() => setConfirmation(null)}
      onOk={async () => {
        await confirmation?.onConfirm();
        setConfirmation(null);
      }}
      open={Boolean(confirmation)}
      title={confirmation?.title}
    >
      {confirmation?.content}
    </Modal>
  );
  return { confirmationModal, confirmDangerAction };
}
