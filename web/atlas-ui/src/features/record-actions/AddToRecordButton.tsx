import { AtlasModal as Modal } from "../../shared/ui/overlays/AtlasModal";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  App,
  Button,
  Form,
  Input,
  Select,
  Tabs,
  Tooltip,
  Typography,
  type InputRef,
  type RefSelectProps,
} from "antd";
import { ListPlus, Plus } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import {
  addEncounterRecordParticipant,
  addSavedListItem,
  createEncounter,
  createSavedList,
  getEncounters,
} from "../../api/atlasApi";
import { encounterDestination, listDestination } from "../../app/destinations";
import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";
import type { RecordSummaryView } from "../../generated/atlas";
import { PaneIconButton } from "../../shared/ui/actions/PaneAction";
import { slugify } from "../lists/listUtils";
import { invalidateSavedListQueries, useSavedLists } from "../lists/savedListQueries";

type DestinationKind = "list" | "encounter";
type Destination = { key: string; name: string; slug: string };

export function AddToRecordButton({
  recordKey,
  record,
}: {
  recordKey: string;
  record?: RecordSummaryView;
}) {
  const [open, setOpen] = useState(false);
  const [hoverHint, setHoverHint] = useState(false);
  const [focusHint, setFocusHint] = useState(false);
  const canAddEncounter = record?.kind === "creature" || record?.kind === "hazard";
  const label = canAddEncounter ? "Add to list or encounter" : "Add to list";
  return (
    <>
      <Tooltip
        title={label}
        trigger={["hover"]}
        open={(hoverHint || focusHint) && !open}
        onOpenChange={setHoverHint}
      >
        <PaneIconButton
          icon={<ListPlus size={16} />}
          label={label}
          // Modal returns focus during its close lifecycle. Keep that focus hint
          // controlled instead of invoking Tooltip's synchronous focus handler.
          onFocus={() => setFocusHint(true)}
          onBlur={() => setFocusHint(false)}
          onClick={(event) => {
            event.currentTarget.focus({ preventScroll: true });
            setHoverHint(false);
            setOpen(true);
          }}
        />
      </Tooltip>
      <AddToRecordModal
        open={open}
        recordKey={recordKey}
        record={record}
        canAddEncounter={canAddEncounter}
        onClose={() => setOpen(false)}
      />
    </>
  );
}

function AddToRecordModal({
  recordKey,
  record,
  onClose,
  open,
  canAddEncounter,
}: {
  open: boolean;
  canAddEncounter: boolean;
  recordKey: string;
  record?: RecordSummaryView;
  onClose: () => void;
}) {
  const [destinationKind, setKind] = useState<DestinationKind>("list");
  const kind = canAddEncounter ? destinationKind : "list";
  const [selected, setSelected] = useState<Destination | null>(null);
  const [creating, setCreating] = useState(false);
  const [created, setCreated] = useState(false);
  const [form] = Form.useForm<{ name: string; note?: string }>();
  const choiceInput = useRef<RefSelectProps>(null);
  const nameInput = useRef<InputRef>(null);
  const newButton = useRef<HTMLAnchorElement | HTMLButtonElement>(null);
  useEffect(() => {
    if (!open) return;
    // Ant Select synchronously updates its popup on focus; run after React's
    // commit so moving focus cannot flush an update inside a lifecycle.
    const frame = requestAnimationFrame(() => {
      if (creating) nameInput.current?.focus({ preventScroll: true });
      else {
        choiceInput.current?.focus();
        if (!choiceInput.current?.nativeElement.contains(document.activeElement))
          newButton.current?.focus({ preventScroll: true });
      }
    });
    return () => cancelAnimationFrame(frame);
  }, [open, creating, kind]);
  const queryClient = useQueryClient();
  const { message } = App.useApp();
  const inFlight = useRef(false);
  const createdDestination = useRef<Destination | null>(null);
  const lists = useSavedLists({ enabled: open && kind === "list" });
  const encounters = useQuery({
    queryKey: ["encounters"],
    queryFn: getEncounters,
    enabled: open && kind === "encounter",
  });
  const catalog = kind === "list" ? lists : encounters;
  const destinations: Destination[] =
    kind === "list"
      ? (lists.data?.lists ?? []).map((list) => ({
          key: list.list_key,
          name: list.name,
          slug: list.slug,
        }))
      : (encounters.data?.encounters ?? [])
          .filter((encounter) => encounter.status !== "archived")
          .map((encounter) => ({
            key: encounter.encounter_key,
            name: encounter.name,
            slug: encounter.slug,
          }));
  const submit = useMutation({
    retry: false,
    mutationFn: async ({ name, note }: { name?: string; note?: string }) => {
      let destination = selected;
      if (creating && name) {
        if (kind === "list") {
          const view = await createSavedList({ name, slug: slugify(name), tags: [] });
          destination = {
            key: view.list.list_key,
            slug: view.list.slug,
            name: view.list.name,
          };
        } else {
          const view = await createEncounter({ name });
          destination = {
            key: view.encounter.encounter_key,
            slug: view.encounter.slug,
            name: view.encounter.name,
          };
        }
        // A later add failure must retry this destination, never create it again.
        createdDestination.current = destination;
        void queryClient.invalidateQueries({
          queryKey: [kind === "list" ? "saved-lists" : "encounters"],
        });
      }
      if (!destination) throw new Error("Choose a destination.");
      if (kind === "list") {
        const receipt = await addSavedListItem({
          list_ref: destination.key,
          record_ref: recordKey,
          ...(note ? { note } : {}),
        });
        void invalidateSavedListQueries(queryClient, [
          destination.key,
          destination.slug,
          receipt.slug,
          receipt.list_key,
        ]);
        return { destination, alreadyPresent: receipt.outcome === "already_present" };
      }
      const receipt = await addEncounterRecordParticipant({
        encounter_ref: destination.key,
        record_ref: recordKey,
        quantity: 1,
      });
      void queryClient.invalidateQueries({ queryKey: ["encounters"] });
      for (const ref of new Set([
        destination.key,
        destination.slug,
        receipt.encounter_key,
      ])) {
        void queryClient.invalidateQueries({ queryKey: ["encounter", ref] });
      }
      return { destination, alreadyPresent: false };
    },
    onSuccess: ({ destination, alreadyPresent }) => {
      const target =
        kind === "list"
          ? listDestination({ slug: destination.slug, selectedRecordKey: null })
          : encounterDestination({ slug: destination.slug });
      void message.success({
        // Feedback chrome must not intercept the next action beneath it.
        styles: { root: { pointerEvents: "none" } },
        content: (
          <>
            {alreadyPresent ? "Already in " : "Added to "}
            <Typography.Link
              style={{ pointerEvents: "auto" }}
              href={destinationHref(target)}
              onClick={(event) => {
                if (!isPrimaryActivation(event)) return;
                event.preventDefault();
                navigateDestination(target);
              }}
            >
              {destination.name}
            </Typography.Link>
          </>
        ),
      });
      onClose();
    },
    onError: () => {
      if (createdDestination.current) {
        setSelected(createdDestination.current);
        setCreating(false);
        setCreated(true);
      }
    },
    onSettled: () => {
      inFlight.current = false;
    },
  });
  const run = (name?: string) => {
    if (inFlight.current) return;
    inFlight.current = true;
    const note = form.getFieldValue("note");
    submit.mutate({ name, note: note?.trim() ? note : undefined });
  };
  const changeMode = (next: boolean) => {
    if (inFlight.current) return;
    createdDestination.current = null;
    setCreating(next);
    setCreated(false);
    submit.reset();
  };
  return (
    <Modal
      open={open}
      afterClose={() => {
        createdDestination.current = null;
        setKind("list");
        setSelected(null);
        setCreating(false);
        setCreated(false);
        form.resetFields();
        submit.reset();
      }}
      title={canAddEncounter ? "Add to…" : "Add to list"}
      transitionName=""
      maskTransitionName=""
      confirmLoading={submit.isPending}
      cancelButtonProps={{ disabled: submit.isPending }}
      okButtonProps={{ disabled: !creating && !selected }}
      closable={{ disabled: submit.isPending }}
      keyboard={!submit.isPending}
      mask={{ closable: !submit.isPending }}
      okText={creating ? "Create & add" : "Add"}
      onCancel={() => {
        if (!inFlight.current) onClose();
      }}
      onOk={() => {
        if (creating) form.submit();
        else if (selected) run();
      }}
      footer={(_, { OkBtn, CancelBtn }) => (
        <div className="record-destination-picker__footer">
          {creating && (
            <Button disabled={submit.isPending} onClick={() => changeMode(false)}>
              Choose existing {kind}
            </Button>
          )}
          <div className="record-destination-picker__footer-actions">
            <CancelBtn />
            <OkBtn />
          </div>
        </div>
      )}
    >
      {record && <Typography.Paragraph>{record.title}</Typography.Paragraph>}
      {canAddEncounter && (
        <Tabs
          activeKey={kind}
          animated={false}
          onChange={(value) => {
            if (inFlight.current) return;
            createdDestination.current = null;
            setKind(value as DestinationKind);
            setSelected(null);
            setCreating(false);
            setCreated(false);
            form.resetFields();
            submit.reset();
          }}
          items={[
            { key: "list", label: "Lists", disabled: submit.isPending },
            { key: "encounter", label: "Encounters", disabled: submit.isPending },
          ]}
        />
      )}
      <div className="record-destination-picker">
        <Form
          className="record-destination-picker__form"
          form={form}
          layout="vertical"
          disabled={submit.isPending}
          onFinish={({ name }) => {
            if (creating) run(name.trim());
          }}
        >
          {creating ? (
            <Form.Item
              style={{ marginBottom: 0 }}
              name="name"
              label={kind === "list" ? "New list name" : "New encounter name"}
              rules={[
                { required: true, whitespace: true, message: "Enter a name." },
                {
                  validator: (_, value: string | undefined) =>
                    kind !== "list" || value === undefined || slugify(value).length > 0
                      ? Promise.resolve()
                      : Promise.reject(new Error("Use at least one letter or number.")),
                },
              ]}
            >
              <Input ref={nameInput} autoComplete="off" />
            </Form.Item>
          ) : (
            <>
              <div className="record-destination-picker__choices">
                <Select
                  className="record-destination-picker__select"
                  ref={choiceInput}
                  aria-label={kind === "list" ? "Saved list" : "Encounter"}
                  disabled={submit.isPending || (!destinations.length && !selected)}
                  loading={catalog.isFetching}
                  showSearch={{ optionFilterProp: "label" }}
                  placeholder={
                    kind === "list" ? "Choose a list" : "Choose an encounter"
                  }
                  value={selected?.key ?? null}
                  options={(selected &&
                  !destinations.some((item) => item.key === selected.key)
                    ? [selected, ...destinations]
                    : destinations
                  ).map((item) => ({ label: item.name, value: item.key }))}
                  onChange={(key) => {
                    if (inFlight.current) return;
                    createdDestination.current = null;
                    setSelected(destinations.find((item) => item.key === key) ?? null);
                    setCreated(false);
                    submit.reset();
                  }}
                />
                <Button
                  ref={newButton}
                  icon={<Plus size={16} />}
                  disabled={submit.isPending}
                  onClick={() => changeMode(true)}
                >
                  New {kind}
                </Button>
              </div>
              {!catalog.isLoading &&
                !catalog.error &&
                destinations.length === 0 &&
                !selected && (
                  <Typography.Text type="secondary">
                    {kind === "list"
                      ? "No lists yet. Create a list to add this record."
                      : "No active encounters. Create an encounter to add this record."}
                  </Typography.Text>
                )}
            </>
          )}
          {kind === "list" && (
            <Form.Item name="note" label="Note" style={{ marginBottom: 0 }}>
              <Input.TextArea
                autoSize={{ minRows: 3, maxRows: 6 }}
                styles={{
                  textarea: {
                    transitionProperty:
                      "background-color, border-color, box-shadow, color",
                  },
                }}
              />
            </Form.Item>
          )}
        </Form>
        {catalog.error && !creating && (
          <ErrorNotice
            error={catalog.error}
            title={
              kind === "list"
                ? catalog.data
                  ? "Saved lists could not refresh"
                  : "Saved lists could not load"
                : catalog.data
                  ? "Encounters could not refresh"
                  : "Encounters could not load"
            }
            description={
              catalog.data ? "Previously loaded choices remain available." : undefined
            }
            action={
              <Button
                size="small"
                loading={catalog.isFetching}
                disabled={submit.isPending}
                onClick={() => void catalog.refetch()}
              >
                Retry
              </Button>
            }
          />
        )}
        {submit.error && (
          <ErrorNotice
            error={submit.error}
            outcome={created ? "saved" : undefined}
            title={
              created
                ? `${selected?.name} was created. Adding the record could not be confirmed.`
                : creating
                  ? `${kind === "list" ? "List" : "Encounter"} could not be created`
                  : "Record could not be added"
            }
          />
        )}
      </div>
    </Modal>
  );
}
