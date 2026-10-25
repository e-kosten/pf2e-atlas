import { render, screen, waitFor, within } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { activityFixture, detailFixture, knownFact } from "../../test/fixtures";
import { RecordReferenceDrawer } from "./RecordReferenceDrawer";

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <ConfigProvider theme={{ token: { motion: false } }}>{children}</ConfigProvider>
);

it("groups identity facts in the Drawer header without duplicating the title or textual action cost", () => {
  const detail = detailFixture("actionspf2e:treat-wounds", "Treat Wounds");
  detail.presentation.classification = "Rule";
  detail.presentation.identity.publication = "Pathfinder Player Core";
  detail.presentation.identity.traits = [
    {
      trait_key: "trait:healing",
      kind: "trait",
      label: "Healing",
      value: "healing",
    },
  ];
  detail.presentation.body = {
    kind: "activity",
    value: {
      ...activityFixture("Treat Wounds"),
      action_cost: knownFact({ glyph: null, label: "passive" }),
    },
  };
  detail.presentation.content = [
    {
      locator: {
        record: detail.record.record_key,
        embedded_path: [],
        field: "/system/description/value",
      },
      role: "description",
      source_fingerprint: null,
      body: {
        kind: "plain",
        text: "Requirements: a healer's toolkit. Spend 10 minutes treating one injured living creature.",
      },
    },
  ];
  render(
    <RecordReferenceDrawer
      anchor={null}
      detail={detail}
      loading={false}
      onClose={vi.fn()}
      onOpenFullPage={vi.fn()}
      onReference={vi.fn()}
    />,
    { wrapper },
  );

  const heading = screen.getByRole("heading", { name: "Treat Wounds" });
  const header = within(heading.closest("header")!);
  expect(screen.getByRole("dialog", { name: "Treat Wounds" })).toBeInTheDocument();
  expect(header.getByText("Rule")).toBeVisible();
  expect(header.getByText("Healing")).toBeVisible();
  const source = header.getByText("Pathfinder Player Core").parentElement;
  expect(source).toBeVisible();
  expect(source).toHaveTextContent("Source: Pathfinder Player Core");
  expect(header.queryByText("passive")).toBeNull();
  expect(screen.getByText("Activation").closest(".record-fact")).toHaveTextContent(
    "Activation passive",
  );
  expect(screen.getAllByText("passive")).toHaveLength(1);
  expect(header.queryByRole("heading", { name: "Description" })).toBeNull();
  expect(screen.queryByRole("heading", { name: "Description" })).toBeNull();
  expect(screen.getByText(/Requirements: a healer's toolkit/)).toBeVisible();
});

it("retains the focused heading when an initially pending document arrives", async () => {
  const props = {
    anchor: null,
    onClose: vi.fn(),
    onOpenFullPage: vi.fn(),
    onReference: vi.fn(),
  };
  const { rerender } = render(
    <RecordReferenceDrawer {...props} detail={undefined} loading />,
    { wrapper },
  );
  const heading = screen.getByRole("heading", { name: "Reference" });
  await waitFor(() => expect(heading).toHaveFocus());
  rerender(
    <RecordReferenceDrawer
      {...props}
      detail={detailFixture("spells:fear", "Fear")}
      loading={false}
    />,
  );
  expect(screen.getByRole("heading", { name: "Fear" })).toBe(heading);
  expect(heading).toHaveFocus();
});
