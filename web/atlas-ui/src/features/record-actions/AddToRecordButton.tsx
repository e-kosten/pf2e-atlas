import { isUnconfirmedMutation } from "../../api/apiError";
import {
  MarkdownEditor,
  type MarkdownEditorHandle,
} from "../../shared/documents/MarkdownEditor";
import { useAtlasRouter } from "../../app/router";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { useCreatureScenario } from "../../shared/records/CreatureScenarioChoices";
import {
  EncounterAddDifficulty,
  encounterRecordAddition,
  recordAdditionRequest,
  encounterPreviewBasis,
  type PreparedCreatureAddition,
  type EncounterRecordAddForm,
} from "../encounters/EncounterAddDifficulty";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { difficultySummary } from "../encounters/EncounterDifficulty";
import { EncounterRecordAdjustment } from "../encounters/EncounterRecordAdjustment";
import { recordDetailQueryOptions } from "../../shared/records/recordDetailQuery";
import { AtlasModal as Modal } from "../../shared/ui/overlays/AtlasModal";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  App,
  Button,
  Form,
  Input,
  InputNumber,
  Select,
  Tabs,
  Tooltip,
  Typography,
  type InputRef,
  type RefSelectProps,
} from "antd";
import { ListPlus, Plus } from "lucide-react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import {
  addEncounterRecordParticipant,
  addSavedListItem,
  createEncounterWithAddition,
  previewDraftEncounterAddition,
  createSavedList,
  getEncounter,
  getEncounters,
  previewEncounterDifficulty,
} from "../../api/atlasApi";
import { encounterDestination, listDestination } from "../../app/destinations";
import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";
import type {
  AddEncounterRecordParticipantRequest,
  CreateEncounterWithAdditionRequest,
  RecordSummaryView,
} from "../../generated/atlas";
import { PaneIconButton } from "../../shared/ui/actions/PaneAction";
import { CheckListDirectory } from "../lists/CheckListDirectory";
import { listNameConflictHelp } from "../lists/listNameConflict";
import { invalidateSavedListQueries, useSavedLists } from "../lists/savedListQueries";

type DestinationKind = "list" | "encounter";
type Destination = { key: string; name: string; slug: string };

export function AddToRecordButton({
  recordKey,
  record,
  encounterOnly = false,
}: {
  recordKey: string;
  record?: RecordSummaryView;
  encounterOnly?: boolean;
}) {
  const paused = useSavedWritesPaused();
  const [open, setOpen] = useState(false);
  const [hoverHint, setHoverHint] = useState(false);
  const [focusHint, setFocusHint] = useState(false);
  const canAddEncounter = record?.kind === "creature" || record?.kind === "hazard";
  const label = encounterOnly
    ? "Add to encounter"
    : canAddEncounter
      ? "Add to list or encounter"
      : "Add to list";
  return (
    <>
      <Tooltip
        title={label}
        trigger={["hover"]}
        open={(hoverHint || focusHint) && !open}
        onOpenChange={setHoverHint}
      >
        <PaneIconButton
          disabled={paused}
          icon={<ListPlus size={16} />}
          label={label}
          outlined
          tooltip={false}
          title=""
          // Modal returns focus during its close lifecycle. Keep that focus hint
          // controlled instead of invoking Tooltip's synchronous focus handler.
          onFocus={() => setFocusHint(true)}
          onBlur={() => setFocusHint(false)}
          onKeyDown={(event) => {
            if (event.key !== "Escape" || open || (!hoverHint && !focusHint)) return;
            event.stopPropagation();
            setHoverHint(false);
            setFocusHint(false);
          }}
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
        encounterOnly={encounterOnly}
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
  encounterOnly,
}: {
  open: boolean;
  canAddEncounter: boolean;
  encounterOnly: boolean;
  recordKey: string;
  record?: RecordSummaryView;
  onClose: () => void;
}) {
  const atlasRouter = useAtlasRouter();
  const paused = useSavedWritesPaused();
  const [destinationKind, setKind] = useState<DestinationKind>("list");
  const kind = encounterOnly ? "encounter" : canAddEncounter ? destinationKind : "list";
  const [selection, setSelected] = useState<Destination | null>(null);
  const [creating, setCreating] = useState(false);
  const [createdList, setCreatedList] = useState(false);
  const [listCreateUncertain, setListCreateUncertain] = useState(false);
  const [directoryChecked, setDirectoryChecked] = useState(false);
  const [noteDraft, setNoteDraft] = useState("");
  const [noteOpen, setNoteOpen] = useState(false);
  const noteEditor = useRef<MarkdownEditorHandle>(null);
  const focusNote = useRef(false);
  useLayoutEffect(() => {
    if (noteOpen && focusNote.current) {
      focusNote.current = false;
      noteEditor.current?.focus();
    }
  }, [noteOpen]);
  const [form] = Form.useForm<EncounterRecordAddForm & { name?: string }>();
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
  const [unconfirmed, setUnconfirmed] = useState<{
    destination: Destination;
    request: AddEncounterRecordParticipantRequest;
  } | null>(null);
  const [unconfirmedCreation, setUnconfirmedCreation] =
    useState<CreateEncounterWithAdditionRequest | null>(null);
  const frozen = Boolean(unconfirmed || unconfirmedCreation);
  const hasUnsavedDraft = () =>
    !frozen &&
    Boolean(
      noteDraft.trim() ||
      (kind === "list" && creating && form.getFieldValue("name")?.trim()),
    );
  const createdDestination = useRef<Destination | null>(null);
  const lists = useSavedLists({ enabled: open && kind === "list" });
  const encounters = useQuery({
    queryKey: ["encounters"],
    queryFn: getEncounters,
    enabled: open && kind === "encounter",
  });
  const selected =
    kind === "encounter" &&
    !frozen &&
    encounters.data &&
    !encounters.data.encounters.some(
      (row) => row.encounter_key === selection?.key && !row.archived,
    )
      ? null
      : selection;
  const encounterKey =
    open && kind === "encounter" && !creating ? selected?.key : undefined;
  const encounter = useQuery({
    queryKey: ["encounter", encounterKey],
    queryFn: () => getEncounter(encounterKey!),
    enabled: Boolean(encounterKey),
  });
  const creature = record?.kind === "creature";
  const detail = useQuery(
    recordDetailQueryOptions(
      open && kind === "encounter" && creature ? recordKey : null,
    ),
  );
  const { useId } = useCreatureScenario(recordKey);
  const [preparedCreature, setPreparedCreature] =
    useState<PreparedCreatureAddition | null>(null);
  const scenario = detail.data?.record.creature_display?.uses.length
    ? {
        artifact_generation: detail.data.artifact_generation,
        ...(useId === undefined ? {} : { use_id: useId }),
      }
    : undefined;
  Form.useWatch("quantity", form);
  Form.useWatch("adjustment", form);
  const previewBasis =
    encounterKey && encounter.data
      ? encounterPreviewBasis(encounterKey, encounter.data.revision)
      : undefined;
  const currentAddition = () =>
    encounterRecordAddition(
      recordKey,
      creature,
      form,
      scenario,
      preparedCreature,
      previewBasis,
    );
  const waitingForCreaturePreview = Boolean(
    encounterKey && scenario && !currentAddition().creature_scenario?.expected,
  );
  const resolvedAdjustment =
    detail.data?.record.record_key === recordKey
      ? detail.data.encounter_adjustment
      : undefined;
  const adjustmentBlocked = kind === "encounter" && creature && !resolvedAdjustment;
  useLayoutEffect(() => {
    if (
      open &&
      kind === "encounter" &&
      creature &&
      !form.isFieldTouched("adjustment")
    ) {
      form.setFields([
        { name: "adjustment", value: resolvedAdjustment, touched: false },
      ]);
    }
  }, [open, kind, creature, form, resolvedAdjustment]);
  const newEncounter = open && creating && kind === "encounter";
  const draftAddition = encounterRecordAddition(recordKey, creature, form, scenario);
  const draftValid =
    Number.isInteger(draftAddition.quantity) &&
    draftAddition.quantity >= 1 &&
    draftAddition.quantity <= 50;
  const preview = useQuery({
    meta: referenceQueryMeta,
    queryKey: ["draft-encounter-addition", draftAddition],
    queryFn: ({ signal }) => previewDraftEncounterAddition(draftAddition, signal),
    enabled: newEncounter && draftValid && !adjustmentBlocked && !frozen,
    retry: false,
  });
  const reviewed =
    draftValid && !preview.isFetching && !preview.isError ? preview.data : undefined;
  const waitingForDraft =
    newEncounter &&
    (!reviewed || (scenario && !reviewed.projected.prepared_creature_choice));
  const listBlocked = listCreateUncertain && !directoryChecked;
  const catalog = kind === "list" ? lists : encounters;
  const destinations: Destination[] =
    kind === "list"
      ? (lists.data?.lists ?? []).map((list) => ({
          key: list.list_key,
          name: list.name,
          slug: list.slug,
        }))
      : (encounters.data?.encounters ?? [])
          .filter((encounter) => !encounter.archived)
          .map((encounter) => ({
            key: encounter.encounter_key,
            name: encounter.name,
            slug: encounter.slug,
          }));
  const choices =
    selected && !destinations.some((item) => item.key === selected.key)
      ? [selected, ...destinations]
      : destinations;
  const options = choices.map((item) => ({
    value: item.key,
    label: item.name,
  }));
  const submit = useMutation({
    retry: false,
    mutationFn: async ({ name, note }: { name?: string; note?: string }) => {
      let destination = unconfirmed?.destination ?? selected;
      if (unconfirmedCreation || (newEncounter && name)) {
        let request = unconfirmedCreation;
        if (!request) {
          if (!reviewed || !name || waitingForDraft)
            throw new Error("Review the encounter addition first.");
          const addition = encounterRecordAddition(recordKey, creature, form, scenario);
          if (JSON.stringify(addition) !== JSON.stringify(draftAddition))
            throw new Error("Review the updated addition first.");
          if (addition.creature_scenario && reviewed.projected.prepared_creature_choice)
            addition.creature_scenario.expected =
              reviewed.projected.prepared_creature_choice;
          request = {
            name,
            addition: recordAdditionRequest(addition),
            expected_artifact_generation: reviewed.artifact_generation,
          };
        }
        try {
          const receipt = await createEncounterWithAddition(request);
          setUnconfirmedCreation(null);
          void queryClient.invalidateQueries({ queryKey: ["encounters"] });
          return {
            destination: {
              key: receipt.encounter_key,
              slug: receipt.encounter_key,
              name: request.name,
            },
            alreadyPresent: false,
            deleted: !receipt.available,
          };
        } catch (error) {
          if (isUnconfirmedMutation(error)) setUnconfirmedCreation(request);
          else {
            setUnconfirmedCreation(null);
            if (creature) void detail.refetch();
            void preview.refetch();
          }
          throw error;
        }
      }
      if (creating && kind === "list" && name) {
        setDirectoryChecked(false);
        try {
          const view = await createSavedList({ name, tags: [] });
          setListCreateUncertain(false);
          destination = {
            key: view.list.list_key,
            slug: view.list.slug,
            name: view.list.name,
          };
        } catch (error) {
          setListCreateUncertain(isUnconfirmedMutation(error));
          throw error;
        }
        createdDestination.current = destination;
        void queryClient.invalidateQueries({ queryKey: ["saved-lists"] });
      }
      if (!destination) throw new Error("Choose a destination.");
      if (kind === "list") {
        const receipt = await addSavedListItem(destination.key, {
          record_ref: recordKey,
          ...(note ? { note } : {}),
        });
        void invalidateSavedListQueries(queryClient, [
          destination.key,
          destination.slug,
          receipt.slug,
          receipt.list_key,
        ]);
        return {
          destination,
          alreadyPresent: receipt.outcome === "already_present",
          deleted: false,
        };
      }
      const request = unconfirmed?.request ?? recordAdditionRequest(currentAddition());
      let receipt;
      try {
        receipt = await addEncounterRecordParticipant(destination.key, request);
      } catch (error) {
        setUnconfirmed(isUnconfirmedMutation(error) ? { destination, request } : null);
        throw error;
      }
      setUnconfirmed(null);
      void queryClient.invalidateQueries({ queryKey: ["encounters"] });
      for (const ref of new Set([
        destination.key,
        destination.slug,
        receipt.encounter_key,
      ])) {
        void queryClient.invalidateQueries({ queryKey: ["encounter", ref] });
      }
      return { destination, alreadyPresent: false, deleted: false };
    },
    onSuccess: (result) => {
      if (result.deleted) {
        void message.info("The encounter was created and has since been deleted.");
        onClose();
        return;
      }
      const { destination, alreadyPresent } = result;
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
                navigateDestination(target, undefined, atlasRouter);
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
        setCreatedList(true);
      }
    },
    onSettled: () => {
      inFlight.current = false;
    },
  });
  const nameConflict =
    creating && kind === "list" ? listNameConflictHelp(submit.error) : undefined;
  const run = (name?: string) => {
    if (inFlight.current) return;
    inFlight.current = true;
    submit.mutate({ name, note: noteDraft.trim() ? noteDraft : undefined });
  };
  const changeMode = (next: boolean) => {
    if (inFlight.current) return;
    createdDestination.current = null;
    setCreating(next);
    setCreatedList(false);
    setListCreateUncertain(false);
    setDirectoryChecked(false);
    submit.reset();
  };
  const cancel = () => {
    if (!inFlight.current) onClose();
  };
  return (
    <Modal
      open={open}
      afterClose={() => {
        // Reopening an unconfirmed add must retain its original destination and ID.
        if (frozen) return;
        createdDestination.current = null;
        setKind("list");
        setSelected(null);
        setCreating(false);
        setCreatedList(false);
        setListCreateUncertain(false);
        setDirectoryChecked(false);
        setNoteOpen(false);
        setNoteDraft("");
        form.resetFields();
        submit.reset();
      }}
      title={
        encounterOnly ? "Add to encounter" : canAddEncounter ? "Add to…" : "Add to list"
      }
      transitionName=""
      maskTransitionName=""
      confirmLoading={submit.isPending}
      cancelButtonProps={{ disabled: submit.isPending }}
      okButtonProps={{
        disabled:
          paused ||
          (!frozen &&
            (listBlocked ||
              waitingForDraft ||
              adjustmentBlocked ||
              waitingForCreaturePreview ||
              (!creating && !selected))),
      }}
      closable={{ disabled: submit.isPending }}
      keyboard={!submit.isPending}
      mask={{ closable: !submit.isPending }}
      okText={
        frozen
          ? "Retry to confirm"
          : creating
            ? listCreateUncertain
              ? "Create another list & add"
              : "Create & add"
            : "Add"
      }
      confirmDiscard={{
        when: hasUnsavedDraft,
        onDiscard: cancel,
      }}
      onCancel={cancel}
      onOk={() => {
        if (paused) return;
        if (frozen) run();
        else if (creating || kind === "encounter") form.submit();
        else if (selected) run();
      }}
      footer={(_, { OkBtn, CancelBtn }) => (
        <div className="record-destination-picker__footer">
          {creating && !frozen && (
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
      {record && (
        <Typography.Text className="record-destination-picker__record">
          {record.title}
        </Typography.Text>
      )}
      {canAddEncounter && !encounterOnly && (
        <Tabs
          activeKey={kind}
          animated={false}
          onChange={(value) => {
            if (inFlight.current) return;
            createdDestination.current = null;
            setKind(value as DestinationKind);
            setSelected(null);
            setCreating(false);
            setCreatedList(false);
            setListCreateUncertain(false);
            setDirectoryChecked(false);
            form.resetFields();
            submit.reset();
          }}
          items={[
            {
              key: "list",
              label: "Lists",
              disabled: submit.isPending || frozen,
            },
            {
              key: "encounter",
              label: "Encounters",
              disabled: submit.isPending || frozen,
            },
          ]}
        />
      )}
      <div className="record-destination-picker">
        <Form
          className="record-destination-picker__form"
          form={form}
          layout="vertical"
          disabled={paused || submit.isPending || frozen}
          onFinish={({ name }) => {
            if (
              frozen ||
              listBlocked ||
              waitingForDraft ||
              adjustmentBlocked ||
              (encounterKey &&
                scenario &&
                !currentAddition().creature_scenario?.expected)
            )
              return;
            if (creating && name) run(name.trim());
            else if (selected) run();
          }}
        >
          {creating ? (
            <Form.Item
              style={{ marginBottom: 0 }}
              name="name"
              label={kind === "list" ? "New list name" : "New encounter name"}
              help={nameConflict}
              validateStatus={nameConflict ? "error" : undefined}
              rules={[{ required: true, whitespace: true, message: "Enter a name." }]}
            >
              <Input
                ref={nameInput}
                autoComplete="off"
                aria-invalid={nameConflict ? true : undefined}
                onChange={() => {
                  if (nameConflict) submit.reset();
                }}
              />
            </Form.Item>
          ) : (
            <>
              <div className="record-destination-picker__choices">
                <Select
                  className="record-destination-picker__select"
                  ref={choiceInput}
                  aria-label={kind === "list" ? "Saved list" : "Encounter"}
                  disabled={
                    submit.isPending || frozen || (!destinations.length && !selected)
                  }
                  loading={catalog.isFetching}
                  showSearch={{ optionFilterProp: "label" }}
                  placeholder={
                    kind === "list" ? "Choose a list" : "Choose an encounter"
                  }
                  value={selected?.key ?? null}
                  options={options}
                  onChange={(key) => {
                    if (inFlight.current) return;
                    createdDestination.current = null;
                    setSelected(destinations.find((item) => item.key === key) ?? null);
                    setCreatedList(false);
                    setListCreateUncertain(false);
                    setDirectoryChecked(false);
                    submit.reset();
                  }}
                />
                <Button
                  ref={newButton}
                  icon={<Plus size={16} />}
                  disabled={submit.isPending || frozen}
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
          {kind === "encounter" && (
            <div className="record-destination-picker__encounter-fields">
              {creature && (
                <EncounterRecordAdjustment
                  loading={detail.isPending}
                  disabled={adjustmentBlocked || frozen}
                />
              )}
              <Form.Item
                name="quantity"
                label="Quantity"
                initialValue={1}
                preserve={false}
                rules={[
                  {
                    required: true,
                    type: "integer",
                    min: 1,
                    max: 50,
                    message: "Enter a whole number from 1 to 50.",
                  },
                ]}
              >
                <InputNumber min={1} max={50} />
              </Form.Item>
            </div>
          )}
          {kind === "list" && (
            <div className="record-destination-picker__note">
              <div className="record-destination-picker__note-controls">
                <Button
                  type="link"
                  size="small"
                  aria-expanded={noteOpen}
                  onClick={() => {
                    focusNote.current = !noteOpen;
                    setNoteOpen(!noteOpen);
                  }}
                >
                  {noteOpen
                    ? "Hide note"
                    : noteDraft.trim()
                      ? "Edit note (included)"
                      : "Add a note"}
                </Button>
              </div>
              {noteOpen && (
                <MarkdownEditor
                  id="note"
                  label="Note"
                  expandedTitle="List entry note"
                  editorRef={noteEditor}
                  value={noteDraft}
                  onChange={setNoteDraft}
                />
              )}
            </div>
          )}
        </Form>
        {newEncounter && !frozen && (
          <section aria-label="Encounter addition preview" aria-live="polite">
            {reviewed ? (
              <>
                <strong>
                  {draftAddition.quantity === 1 && "1 × "}
                  {reviewed.projected.opposition[0]?.name}
                </strong>
                <div>
                  {reviewed.projected.party_level === null ||
                  reviewed.projected.party_size === null
                    ? "Set the party level and size to see XP and difficulty."
                    : difficultySummary(reviewed.projected)}
                </div>
              </>
            ) : preview.error && draftValid && !adjustmentBlocked ? (
              <ErrorNotice
                error={preview.error}
                title="Addition preview could not load"
                action={
                  <Button
                    size="small"
                    onClick={() => {
                      if (scenario) void detail.refetch();
                      void preview.refetch();
                    }}
                  >
                    Retry
                  </Button>
                }
              />
            ) : draftValid && !adjustmentBlocked ? (
              "Preparing addition…"
            ) : null}
          </section>
        )}
        {encounter.data && encounterKey && (
          <EncounterAddDifficulty
            encounterKey={encounterKey}
            revision={encounter.data.revision}
            artifactGeneration={encounter.data.artifact_generation}
            value={encounter.data.difficulty}
            onPreview={(request) => previewEncounterDifficulty(encounterKey, request)}
            form={form}
            selectedRecordKey={recordKey}
            creature={creature}
            scenario={scenario}
            onPreparedCreature={setPreparedCreature}
            recordName={record?.title ?? recordKey}
            disabled={submit.isPending || adjustmentBlocked || frozen}
          />
        )}
        {adjustmentBlocked && detail.error && (
          <ErrorNotice
            error={detail.error}
            title="Creature adjustment could not load"
            action={<Button onClick={() => void detail.refetch()}>Retry</Button>}
          />
        )}
        {encounter.error && (
          <ErrorNotice
            error={encounter.error}
            title="Encounter difficulty could not load"
          />
        )}
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
        {submit.error && !nameConflict && (
          <ErrorNotice
            error={submit.error}
            outcome={
              createdList
                ? "saved"
                : frozen || listCreateUncertain
                  ? "uncertain"
                  : undefined
            }
            description={
              listCreateUncertain
                ? "Check the list directory before creating another list."
                : undefined
            }
            action={
              listCreateUncertain && (
                <CheckListDirectory onChecked={() => setDirectoryChecked(true)} />
              )
            }
            title={
              createdList
                ? `${selected?.name} was created. The record could not be added.`
                : listCreateUncertain
                  ? "List creation could not be confirmed"
                  : unconfirmedCreation
                    ? "Encounter creation could not be confirmed"
                    : unconfirmed
                      ? "Addition could not be confirmed"
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
