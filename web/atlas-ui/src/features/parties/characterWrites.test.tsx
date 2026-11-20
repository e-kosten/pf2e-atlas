import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { App, ConfigProvider } from "antd";
import type { ReactNode } from "react";
import { AtlasApiError } from "../../api/apiError";
import { RouterTestContext } from "../../test/router";
import { CharacterEditor } from "./CharacterEditor";
import { CharacterPicker } from "./CharacterPicker";
import { EncounterPcAdd } from "./EncounterPcAdd";
import { PathbuilderImport } from "./PathbuilderImport";
const api = vi.hoisted(() => ({
  createCharacter: vi.fn(),
  getParties: vi.fn(),
  getCharacters: vi.fn(),
  getEncounter: vi.fn(),
  addEncounterCharacter: vi.fn(),
  previewPathbuilderPc: vi.fn(),
  savePathbuilderPc: vi.fn(),
}));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));
function setup() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={client}>
        <ConfigProvider theme={{ token: { motion: false } }}>
          <RouterTestContext>
            <App>{children}</App>
          </RouterTestContext>
        </ConfigProvider>
      </QueryClientProvider>
    );
  }
  return { client, wrapper: Wrapper };
}
beforeEach(() => {
  Object.values(api).forEach((mock) => mock.mockReset());
  api.getParties.mockResolvedValue({ parties: [] });
});
afterEach(cleanup);

it("reviews the latest party revision without discarding an inline character draft", async () => {
  api.createCharacter
    .mockRejectedValueOnce(new AtlasApiError(409, "Party changed"))
    .mockResolvedValue({ character_key: "pc-new", available: true });
  api.getParties.mockResolvedValue({
    parties: [{ party_key: "party", name: "Current group", revision: 5, members: [] }],
  });
  const saved = vi.fn();
  render(
    <CharacterEditor
      inline
      party={{ party_key: "party", name: "Old group", revision: 4, members: [] }}
      onClose={vi.fn()}
      onSaved={saved}
    />,
    setup(),
  );
  fireEvent.change(screen.getByLabelText("Name"), { target: { value: "New PC" } });
  fireEvent.click(screen.getByRole("button", { name: "Add to party" }));
  fireEvent.click(await screen.findByRole("button", { name: "Review latest changes" }));
  await screen.findByText(/Latest party Current group/);
  expect(screen.getByLabelText("Name")).toHaveValue("New PC");
  fireEvent.click(screen.getByRole("button", { name: "Add to party" }));
  await waitFor(() => expect(saved).toHaveBeenCalledWith("pc-new"));
  expect(api.createCharacter.mock.calls[1][0]).toEqual(
    expect.objectContaining({
      name: "New PC",
      party: { party_key: "party", expected_revision: 5 },
    }),
  );
});

it("replays a manual PC addition unchanged after a lost response and refreshed encounter", async () => {
  api.addEncounterCharacter
    .mockRejectedValueOnce(new Error("Response lost"))
    .mockResolvedValue({
      encounter_key: "e",
      character_key: "c",
      participant_key: "p",
      available: true,
    });
  const context = setup();
  const view = render(
    <EncounterPcAdd encounter={{ encounterKey: "e", revision: 4 }} onClose={vi.fn()} />,
    context,
  );
  fireEvent.click(screen.getByRole("button", { name: "Enter manually" }));
  fireEvent.change(screen.getByLabelText("Name"), { target: { value: "New PC" } });
  fireEvent.click(screen.getByRole("button", { name: "Add to encounter" }));
  await screen.findByRole("button", { name: "Retry to confirm" });
  view.rerender(
    <EncounterPcAdd encounter={{ encounterKey: "e", revision: 5 }} onClose={vi.fn()} />,
  );
  expect(screen.getByLabelText("Name")).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Retry to confirm" }));
  await waitFor(() => expect(api.addEncounterCharacter).toHaveBeenCalledTimes(2));
  expect(api.addEncounterCharacter.mock.calls[1]).toEqual(
    api.addEncounterCharacter.mock.calls[0],
  );
});

it("replays an existing character addition using its originally reviewed revisions", async () => {
  const character = {
    character_key: "c",
    name: "Aster",
    revision: 2,
    imported: false,
    parties: [],
  };
  api.getCharacters.mockResolvedValue({ characters: [character] });
  api.addEncounterCharacter
    .mockRejectedValueOnce(new Error("Response lost"))
    .mockResolvedValue({
      encounter_key: "e",
      character_key: "c",
      participant_key: "p",
      available: true,
    });
  const context = setup();
  const view = render(
    <CharacterPicker
      encounter={{ encounterKey: "e", revision: 4 }}
      onClose={vi.fn()}
    />,
    context,
  );
  fireEvent.click(await screen.findByRole("button", { name: "Add" }));
  await screen.findByRole("button", { name: "Retry to confirm" });
  context.client.setQueryData(["characters"], {
    characters: [{ ...character, revision: 3 }],
  });
  view.rerender(
    <CharacterPicker
      encounter={{ encounterKey: "e", revision: 5 }}
      onClose={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Retry to confirm" }));
  await waitFor(() => expect(api.addEncounterCharacter).toHaveBeenCalledTimes(2));
  expect(api.addEncounterCharacter.mock.calls[1]).toEqual(
    api.addEncounterCharacter.mock.calls[0],
  );
});

it("reviews retained rule corrections and refreshes a stale revision without losing the import", async () => {
  const ability = {
    ability_key: "counterspell",
    label: "Counterspell",
    kind: "feat",
    manual: true,
    automatic_match_available: false,
    ambiguous: false,
    record_key: "feats:prepared",
    record_title: "Counterspell (Prepared)",
  };
  const sections = [{ kind: "feat", label: "Feats", abilities: [ability] }];
  const preview = {
    name: "Aster",
    level: 7,
    artifact_generation: "generation",
    character_revision: 2,
    previous_name: "Aster",
    previous_level: 7,
    retained_manual_count: 1,
    resolved_count: 1,
    unresolved_count: 0,
    abilities: [ability],
    sections,
  };
  api.previewPathbuilderPc
    .mockResolvedValueOnce(preview)
    .mockResolvedValue({ ...preview, character_revision: 3 });
  api.savePathbuilderPc
    .mockRejectedValueOnce(new AtlasApiError(409, "Character changed"))
    .mockResolvedValue({ character_key: "c", available: true });
  const saved = vi.fn();
  render(
    <PathbuilderImport
      character={{
        character_key: "c",
        character_revision: 1,
        name: "Aster",
        level: 7,
        parties: [],
        imported: true,
        abilities: [ability],
        sections,
        resolved_count: 1,
        unresolved_count: 0,
      }}
      onClose={vi.fn()}
      onSaved={saved}
    />,
    setup(),
  );
  fireEvent.change(screen.getByLabelText("Pathbuilder JSON"), {
    target: { value: "{}" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Continue" }));
  await screen.findByText("1 saved link retained");
  expect(api.previewPathbuilderPc).toHaveBeenCalledWith("{}", "c");
  expect(screen.getByText("Counterspell (Prepared)")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Refresh character" }));
  fireEvent.click(await screen.findByRole("button", { name: "Review latest changes" }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Refresh character" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Refresh character" }));
  await waitFor(() => expect(saved).toHaveBeenCalledWith("c"));
  expect(api.savePathbuilderPc.mock.calls[0][0].expected_character_revision).toBe(2);
  expect(api.savePathbuilderPc.mock.calls[1][0]).toEqual(
    expect.objectContaining({
      json: "{}",
      character_key: "c",
      expected_character_revision: 3,
    }),
  );
});

it("keeps importer cancellation disabled until the submitted save finishes", async () => {
  api.previewPathbuilderPc.mockResolvedValue({
    name: "Aster",
    level: 7,
    artifact_generation: "g",
    abilities: [],
    sections: [],
    resolved_count: 0,
    unresolved_count: 0,
    retained_manual_count: 0,
  });
  let finish!: (value: { character_key: string; available: boolean }) => void;
  api.savePathbuilderPc.mockReturnValue(
    new Promise((resolve) => {
      finish = resolve;
    }),
  );
  const closed = vi.fn();
  render(<PathbuilderImport onClose={closed} />, setup());
  fireEvent.change(screen.getByLabelText("Pathbuilder JSON"), {
    target: { value: "{}" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Continue" }));
  fireEvent.click(await screen.findByRole("button", { name: "Add character" }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled(),
  );
  expect(screen.queryByRole("button", { name: "Close" })).not.toBeInTheDocument();
  expect(closed).not.toHaveBeenCalled();
  finish({ character_key: "c", available: true });
  await waitFor(() => expect(closed).toHaveBeenCalledOnce());
});
