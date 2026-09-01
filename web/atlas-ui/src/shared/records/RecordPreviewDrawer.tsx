import { Drawer } from "antd";
import { RecordDetailPane } from "./RecordDetailPane";
import { RecordPreviewActions } from "./RecordPreviewActions";
import type { RecordPreviewContentProps } from "./recordPreviewTypes";

export function RecordPreviewDrawer({
  detail,
  loading,
  onClose,
  onOpenFullPage,
  onReference,
  open,
  afterOpenChange,
}: {
  afterOpenChange: (open: boolean) => void;
  open: boolean;
} & RecordPreviewContentProps) {
  return (
    <Drawer
      afterOpenChange={afterOpenChange}
      className="record-preview-drawer"
      closable={false}
      destroyOnHidden
      open={open}
      placement="right"
      title="Reference"
      width="min(560px, calc(100vw - 24px))"
      extra={<RecordPreviewActions onClose={onClose} onOpenFullPage={onOpenFullPage} />}
      onClose={onClose}
    >
      <section aria-label="Reference preview" role="dialog">
        <RecordDetailPane
          detail={detail}
          errors={[]}
          loading={loading}
          onReference={onReference}
        />
      </section>
    </Drawer>
  );
}
