import { useState } from "react";
import { Button } from "antd";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { rootNavigation } from "../../shared/records/useReferenceReading";
import { useReferencePreview } from "../../shared/records/ReferencePreviewContext";
import { RemasterQueryInformation } from "../../shared/records/RemasterQueryInformation";

/** Retained saved identity is queried only on demand; successor navigation is separate. */
export function UnavailableRecordRemasterAction({
  recordKey,
  title,
  disabled,
}: {
  recordKey: string;
  title: string;
  disabled?: boolean;
}) {
  const [open, setOpen] = useState(false);
  const preview = useReferencePreview();
  return (
    <>
      <Button
        size="small"
        type="link"
        disabled={disabled || !preview}
        onClick={() => setOpen(true)}
      >
        Remaster information
      </Button>
      <AtlasModal
        open={open}
        destroyOnHidden
        title={`Remaster information · ${title}`}
        footer={null}
        onCancel={() => setOpen(false)}
      >
        <RemasterQueryInformation
          enabled={open}
          request={{
            subject: { kind: "record", selection: rootNavigation(recordKey) },
            limit: 20,
          }}
          onReference={(...args) => preview?.open(...args)}
        />
      </AtlasModal>
    </>
  );
}
