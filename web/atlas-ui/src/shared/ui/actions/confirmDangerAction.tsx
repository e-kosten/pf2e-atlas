import { Modal } from "antd";

export function confirmDangerAction({
  content,
  okText,
  onConfirm,
  title,
}: {
  content?: string;
  okText: string;
  onConfirm: () => void;
  title: string;
}) {
  Modal.confirm({
    title,
    content,
    okText,
    okButtonProps: { danger: true },
    cancelText: "Cancel",
    onOk: onConfirm,
  });
}
