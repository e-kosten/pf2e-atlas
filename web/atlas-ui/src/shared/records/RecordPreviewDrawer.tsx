import { Drawer } from "antd";
import type { RecordDetailView } from "../../generated/atlas";
import { RecordDetailPane } from "./RecordDetailPane";
import { RecordPreviewActions } from "./RecordPreviewActions";

type RecordPreviewContentProps = {
  detail: RecordDetailView | undefined;
  loading: boolean;
  onClose: () => void;
  onOpenFullPage: () => void;
  onReference: (recordKey: string) => void;
};

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
