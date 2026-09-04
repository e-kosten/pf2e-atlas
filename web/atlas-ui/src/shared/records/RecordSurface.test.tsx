import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type {
  CreatureSurfaceContentView,
  RecordSurfaceView,
} from "../../generated/atlas";
import {
  creatureSurfaceFixture,
  encounterRuntimeFixture,
  occurrenceProvenance,
} from "../../test/recordFixtures";
import { RecordSurface } from "./RecordSurface";

const onReference = vi.fn();
const AIR_MEPHIT_KEY = "pathfinder-bestiary:KDRlxdIUADWHI6Vr";
const AIR_SCAMP_KEY = "pathfinder-monster-core:MSm1im7lZA5i82rz";

describe("RecordSurface", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders description before mechanics with record kind in identity metadata", () => {
    const { container } = renderSurface();

    const description = screen.getByRole("heading", { name: "Overview" });
    const mechanics = screen.getByRole("heading", { name: "Actions" });
    expect(
      description.compareDocumentPosition(mechanics) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      screen.getByRole("heading", { name: "Dream-Coven Envoy" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Level 9")).toBeInTheDocument();
    expect(screen.getByText("Creature")).toBeInTheDocument();
    expect(
      container.querySelector(".creature-sheet__snapshot"),
    ).not.toBeInTheDocument();
  });

  it("renders each core statistic exactly once", () => {
    renderSurface();

    for (const label of ["Perception", "HP", "AC", "Fortitude", "Reflex", "Will"]) {
      expect(
        screen
          .getAllByText(label, { exact: true })
          .filter((element) => element.tagName === "DT"),
      ).toHaveLength(1);
    }
    const defenses = screen.getByRole("heading", {
      name: "Defenses & Vitals",
    }).parentElement;
    expect(defenses).not.toBeNull();
    const defenseStats = defenses!.querySelector(".creature-sheet__defense-stats");
    expect(defenseStats).toBeVisible();
    expect(defenseStats?.children).toHaveLength(6);
    for (const label of ["HP", "AC", "Hardness", "Fortitude", "Reflex", "Will"]) {
      const statLabel = within(defenseStats as HTMLElement).getByText(label, {
        exact: true,
      });
      expect(statLabel).toBeInTheDocument();
      expect(statLabel.closest("dl")).toBe(defenseStats);
      expect(statLabel.closest(".record-key-value-list")).toBeNull();
    }
    expect(within(defenses!).getByText("Cold Iron 5")).toBeInTheDocument();
    expect(
      within(defenses!).queryByText("Perception", { exact: true }),
    ).not.toBeInTheDocument();

    const profile = document
      .querySelector('dl[aria-label="Creature profile facts"]')
      ?.closest<HTMLElement>(".creature-sheet__panel--profile");
    expect(profile).not.toBeNull();
    const perceptionLabel = within(profile!)
      .getAllByText("Perception", { exact: true })
      .find((element) => element.tagName === "DT")!;
    expect(perceptionLabel).toBeInTheDocument();
    expect(perceptionLabel.closest(".record-key-value-list")).toHaveAttribute(
      "aria-label",
      "Perception and senses",
    );
    for (const label of ["HP", "AC", "Fortitude", "Reflex", "Will"]) {
      expect(
        within(profile!).queryByText(label, { exact: true }),
      ).not.toBeInTheDocument();
    }
  });

  it("renders compact profile facts, corrected modifiers, and ordered skill variants", () => {
    const { container } = renderSurface();

    expect(
      screen.queryByRole("heading", { name: "Profile & Awareness" }),
    ).not.toBeInTheDocument();
    const profileList = container.querySelector<HTMLElement>(
      'dl[aria-label="Creature profile facts"]',
    )!;
    const profile = profileList.closest<HTMLElement>(
      ".creature-sheet__panel--profile",
    )!;
    expect(screen.getByText("Medium")).toBeInTheDocument();
    expect(screen.getByText("Elite")).toBeInTheDocument();
    expect(screen.getByText("Initiative")).toBeInTheDocument();
    expect(container.querySelector('dl[aria-label="Creature profile"]')).toBeNull();
    for (const label of [
      "Size",
      "Adjustment",
      "Initiative",
      "Movement",
      "Perception & senses",
      "Languages & communication",
    ]) {
      const factLabel = within(profileList)
        .getAllByText(label, { exact: true })
        .find((element) => element.tagName === "DT")!;
      expect(factLabel.parentElement).toHaveClass("record-key-value-list__row");
    }
    const perceptionGroup =
      within(profileList).getByText("Perception & senses").parentElement!;
    const languageGroup = within(profileList).getByText(
      "Languages & communication",
    ).parentElement!;
    expect(perceptionGroup).toHaveClass("creature-sheet__fact-group");
    expect(languageGroup).toHaveClass("creature-sheet__fact-group");
    expect(
      perceptionGroup.querySelector('dl[aria-label="Perception and senses"]'),
    ).toHaveClass("creature-sheet__fact-group-rows");
    expect(
      languageGroup.querySelector('dl[aria-label="Languages and communication"]'),
    ).toHaveClass("creature-sheet__fact-group-rows");
    expect(profileList).toHaveClass("creature-sheet__compact-fact-grid");
    expect(profile.querySelector(".ant-tag")).toBeNull();
    const overview = screen.getByRole("heading", { name: "Overview" });
    const overviewCallout = overview.closest<HTMLElement>(
      ".creature-sheet__narrative",
    )!;
    const factsGrid = profile.closest<HTMLElement>(".creature-sheet__facts-grid")!;
    expect(overviewCallout.nextElementSibling).toBe(factsGrid);
    expect(overviewCallout).not.toContainElement(profileList);

    const abilities = screen.getByRole("heading", {
      name: "Ability Modifiers",
    }).parentElement!;
    const defenses = screen.getByRole("heading", {
      name: "Defenses & Vitals",
    }).parentElement!;
    expect(abilities.parentElement).toBe(defenses.parentElement);
    expect(abilities.parentElement).toHaveClass(
      "creature-sheet__facts-column--primary",
    );
    for (const [label, value] of [
      ["Str", "+4"],
      ["Dex", "+3"],
      ["Con", "+4"],
      ["Int", "+5"],
      ["Wis", "+4"],
      ["Cha", "+5"],
    ]) {
      expect(within(abilities).getByText(label)).toBeInTheDocument();
      expect(within(abilities).getAllByText(value).length).toBeGreaterThan(0);
    }

    const skills = screen.getByRole("heading", { name: "Skills" }).parentElement!;
    expect(within(skills).getByText("Occultism")).toBeInTheDocument();
    expect(within(skills).getByText("Dreams")).toBeInTheDocument();
    expect(within(skills).getByText("Nightmares")).toBeInTheDocument();
    expect(within(skills).getByText("dreams")).toBeInTheDocument();
    expect(within(skills).queryByText("Source key")).not.toBeInTheDocument();
    expect(within(skills).queryByText("occultism")).not.toBeInTheDocument();
    const variants = skills.querySelectorAll(".creature-sheet__skill-variants li");
    expect(variants).toHaveLength(2);
    expect(variants[0]).toHaveTextContent("Dreams+24dreams");
    expect(variants[1]).toHaveTextContent("Nightmares+23nightmares");
    expect(within(skills).getByRole("list", { name: "Skills" })).toHaveClass(
      "creature-sheet__skill-grid",
    );
    expect(within(skills).getByText("Occultism").parentElement).toHaveClass(
      "creature-sheet__skill-heading",
    );
    expect(
      within(within(skills).getByText("Occultism").parentElement!).getByText("+22"),
    ).toBeInTheDocument();
    expect(skills.querySelectorAll(".creature-sheet__skill-cell")).toHaveLength(1);
    expect(skills.querySelector(".record-key-value-list")).toBeNull();
  });

  it("keeps all movement modes together in the compact profile grid", () => {
    renderSurface();

    const profile = document
      .querySelector('dl[aria-label="Creature profile facts"]')!
      .closest<HTMLElement>(".creature-sheet__panel--profile")!;
    const movementLabel = within(profile)
      .getAllByText("Movement", { exact: true })
      .find((element) => element.tagName === "DT")!;
    const movement = movementLabel.parentElement!;
    expect(movement).toHaveClass("record-key-value-list__row");
    const movementValues = within(movement).getByRole("list", { name: "Movement" });
    expect(movementValues.children).toHaveLength(2);
    expect(within(movementValues).getByText("Speed")).toBeInTheDocument();
    expect(within(movementValues).getByText("Fly")).toBeInTheDocument();
    expect(within(movementValues).getByText("25 feet")).toBeInTheDocument();
    expect(within(movementValues).getByText("40 feet")).toBeInTheDocument();
    for (const item of movementValues.children) {
      expect(item).toHaveClass("creature-sheet__compact-multi-value-item");
    }
    expect(screen.queryByRole("heading", { name: "Movement" })).not.toBeInTheDocument();
  });

  it("uses one wrap-safe list pattern for typed senses, languages, and movement", () => {
    const surface = detailedSurfaceFixture();
    if (surface.presentation.presentation_type !== "creature") {
      throw new Error("Fixture must be a creature surface");
    }
    surface.presentation.body.awareness = {
      ...surface.presentation.body.awareness!,
      senses: [
        {
          component_id: "darkvision",
          authored_order: 0,
          kind: "darkvision",
        },
        {
          component_id: "scent",
          authored_order: 1,
          kind: "scent",
          acuity: "imprecise",
          range_feet: 30,
        },
      ],
      languages: ["common"],
      language_details: "Telepathy 100 feet",
    };

    render(<RecordSurface onReference={onReference} surface={surface} />);

    const lists = [
      screen.getByRole("list", { name: "Senses" }),
      screen.getByRole("list", { name: "Languages" }),
      screen.getByRole("list", { name: "Movement" }),
    ];
    for (const list of lists) {
      expect(list).toHaveClass("creature-sheet__compact-multi-value-list");
      for (const item of list.children) {
        expect(item.tagName).toBe("LI");
        expect(item).toHaveClass("creature-sheet__compact-multi-value-item");
      }
    }
    expect(lists[0].children).toHaveLength(2);
    expect(lists[1].children).toHaveLength(1);
    expect(lists[2].children).toHaveLength(2);
    expect(lists[0]).toHaveTextContent("DarkvisionScent—imprecise, 30 feet");
    expect(lists[1]).toHaveTextContent("Common");
    expect(lists[1]).not.toHaveTextContent("Telepathy 100 feet");
    expect(lists[2]).toHaveTextContent("Speed—25 feetFly—40 feet");
    const languageGroup = screen.getByText("Languages & communication").parentElement!;
    const languageDetails = within(languageGroup).getByText("Telepathy 100 feet");
    expect(languageDetails.tagName).toBe("DD");
    expect(languageDetails.closest("li")).toBeNull();
    expect(languageDetails.parentElement).toHaveClass(
      "creature-sheet__fact-group-note",
    );
    expect(
      document.querySelector(".creature-sheet__compact-multi-value-list .ant-tag"),
    ).toBeNull();
  });

  it("keeps structured senses and exact perception Details distinct", () => {
    const surface = detailedSurfaceFixture();
    if (surface.presentation.presentation_type !== "creature") {
      throw new Error("Fixture must be a creature surface");
    }
    surface.presentation.body.awareness = {
      ...surface.presentation.body.awareness!,
      details: "fog vision",
      senses: [
        {
          component_id: "darkvision",
          authored_order: 0,
          kind: "darkvision",
        },
      ],
    };
    surface.presentation.body.activities = [
      ...(surface.presentation.body.activities ?? []),
      {
        occurrence_id: "fog-vision",
        authored_order: 99,
        provenance: occurrenceProvenance,
        activity_type: "action",
        label: "Fog Vision",
        action_cost: { cost_type: "passive" },
        content: [
          passiveActivityContent("The air scamp ignores concealment from fog."),
        ],
      },
    ];

    render(<RecordSurface onReference={onReference} surface={surface} />);

    const profile = document
      .querySelector('dl[aria-label="Creature profile facts"]')!
      .closest<HTMLElement>(".creature-sheet__panel--profile")!;
    const perceptionGroup =
      within(profile).getByText("Perception & senses").parentElement!;
    const fogVision = within(perceptionGroup).getByText("fog vision");
    expect(fogVision.tagName).toBe("DD");
    expect(fogVision.parentElement).toHaveClass("creature-sheet__fact-group-note");
    expect(within(fogVision.parentElement!).getByText("Details")).toBeInTheDocument();
    const senses = within(perceptionGroup).getByRole("list", { name: "Senses" });
    expect(senses.children).toHaveLength(1);
    expect(senses).toHaveTextContent("Darkvision");
    expect(senses).not.toHaveTextContent("·");
    expect(within(perceptionGroup).queryByText("Fog Vision")).not.toBeInTheDocument();
    const features = screen.getByRole("heading", {
      name: "Passives",
    }).parentElement!;
    expect(within(features).getByText("Fog Vision")).toBeInTheDocument();
  });

  it("keeps sparse fact sections in the natural two-column flow", () => {
    const surface = detailedSurfaceFixture();
    if (surface.presentation.presentation_type !== "creature") {
      throw new Error("Fixture must be a creature surface");
    }
    surface.presentation.body.defenses = {
      ...surface.presentation.body.defenses!,
      weaknesses: undefined,
    };
    surface.presentation.body.abilities = undefined;
    surface.presentation.body.skills = [
      {
        component_id: "stealth",
        authored_order: 0,
        kind: "stealth",
        label: "Stealth",
        modifier: 5,
      },
    ];

    const { container } = render(
      <RecordSurface onReference={onReference} surface={surface} />,
    );
    const sections = container.querySelectorAll(
      ".creature-sheet__facts-grid .creature-sheet__panel",
    );
    expect(sections).toHaveLength(3);
    expect(sections[0]).toHaveClass("creature-sheet__panel--defenses");
    expect(sections[1]).toHaveClass("creature-sheet__panel--profile");
    expect(sections[2]).toHaveClass("creature-sheet__panel--skills");
    expect(sections[0]?.parentElement).toHaveClass(
      "creature-sheet__facts-column--primary",
    );
    expect(sections[1]?.parentElement).toHaveClass(
      "creature-sheet__facts-column--secondary",
    );
    expect(sections[2]?.parentElement).toHaveClass(
      "creature-sheet__facts-column--secondary",
    );
  });

  it("renders Areelu structured values and exact authored Details", () => {
    const surface = detailedSurfaceFixture();
    if (surface.presentation.presentation_type !== "creature") {
      throw new Error("Fixture must be a creature surface");
    }
    surface.metadata.title = "Areelu Vorlesh";
    surface.presentation.body.awareness = {
      ...surface.presentation.body.awareness!,
      perception: 34,
      details: "darkvision, truesight",
      senses: [],
      languages: [
        "aklo",
        "chthonian",
        "common",
        "diabolic",
        "draconic",
        "dwarven",
        "elven",
        "empyrean",
        "fey",
        "halfling",
        "hallit",
        "jotun",
        "necril",
        "sakvroth",
        "gnomish",
        "orcish",
      ],
      language_details: "Telepathy 100 feet",
    };

    render(<RecordSurface onReference={onReference} surface={surface} />);

    const perceptionGroup = screen.getByText("Perception & senses").parentElement!;
    expect(within(perceptionGroup).getByText("+34")).toBeInTheDocument();
    expect(within(perceptionGroup).queryByText("Senses")).not.toBeInTheDocument();
    const perceptionDetails = within(perceptionGroup).getByText(
      "darkvision, truesight",
    );
    expect(perceptionDetails.parentElement).toHaveClass(
      "creature-sheet__fact-group-note",
    );

    const languageGroup = screen.getByText("Languages & communication").parentElement!;
    const languages = within(languageGroup).getByRole("list", { name: "Languages" });
    expect(languages.children).toHaveLength(16);
    expect(within(languages).getByText("Aklo")).toBeInTheDocument();
    expect(within(languages).getByText("Orcish")).toBeInTheDocument();
    expect(languages).not.toHaveTextContent("Telepathy 100 feet");
    expect(
      within(languageGroup).getByText("Telepathy 100 feet").parentElement,
    ).toHaveClass("creature-sheet__fact-group-note");
  });

  it("renders Giant Rat typed senses and omits empty Details", () => {
    const surface = detailedSurfaceFixture();
    if (surface.presentation.presentation_type !== "creature") {
      throw new Error("Fixture must be a creature surface");
    }
    surface.metadata.title = "Giant Rat";
    surface.presentation.body.awareness = {
      ...surface.presentation.body.awareness!,
      perception: 5,
      details: "",
      senses: [
        {
          component_id: "sense:low-light-vision:0",
          authored_order: 0,
          kind: "low-light-vision",
        },
        {
          component_id: "sense:scent:0",
          authored_order: 1,
          kind: "scent",
          acuity: "imprecise",
          range_feet: 30,
        },
      ],
      languages: [],
      language_details: "",
    };

    render(<RecordSurface onReference={onReference} surface={surface} />);

    const perceptionGroup = screen.getByText("Perception & senses").parentElement!;
    expect(within(perceptionGroup).getByText("+5")).toBeInTheDocument();
    const senses = within(perceptionGroup).getByRole("list", { name: "Senses" });
    expect(senses.children).toHaveLength(2);
    expect(within(senses).getByText("Low Light Vision")).toBeInTheDocument();
    expect(within(senses).getByText("Scent")).toBeInTheDocument();
    expect(within(senses).getByText("imprecise, 30 feet")).toBeInTheDocument();
    expect(within(perceptionGroup).queryByText("Details")).not.toBeInTheDocument();
    expect(screen.queryByText("Languages & communication")).not.toBeInTheDocument();
  });

  it("shows details-only groups without fabricating structured rows", () => {
    const surface = detailedSurfaceFixture();
    if (surface.presentation.presentation_type !== "creature") {
      throw new Error("Fixture must be a creature surface");
    }
    surface.presentation.body.awareness = {
      provenance: surface.presentation.body.awareness!.provenance,
      details: "heat ripples",
      language_details: "Telepathy 100 feet",
    };

    render(<RecordSurface onReference={onReference} surface={surface} />);

    const perceptionGroup = screen.getByText("Perception & senses").parentElement!;
    expect(within(perceptionGroup).getByText("heat ripples")).toBeInTheDocument();
    expect(within(perceptionGroup).queryByText("Perception")).not.toBeInTheDocument();
    expect(within(perceptionGroup).queryByText("Senses")).not.toBeInTheDocument();
    const languageGroup = screen.getByText("Languages & communication").parentElement!;
    expect(within(languageGroup).getByText("Telepathy 100 feet")).toBeInTheDocument();
    expect(within(languageGroup).queryByText("Languages")).not.toBeInTheDocument();
  });

  it("uses semantic lists with responsive fact and skill grid hooks", () => {
    const { container } = renderSurface();

    const facts = container.querySelector<HTMLElement>(
      'dl[aria-label="Creature profile facts"]',
    )!;
    expect(facts.tagName).toBe("DL");
    expect(facts).toHaveClass("creature-sheet__compact-fact-grid");
    expect(facts.querySelectorAll(":scope > .record-key-value-list__row")).toHaveLength(
      6,
    );
    expect(facts.querySelectorAll(":scope > .creature-sheet__fact-group")).toHaveLength(
      2,
    );

    const skills = screen.getByRole("list", { name: "Skills" });
    expect(skills.tagName).toBe("UL");
    expect(skills).toHaveClass("creature-sheet__skill-grid");
    expect(
      skills.querySelectorAll(":scope > .creature-sheet__skill-cell"),
    ).toHaveLength(1);
  });

  it("starts action and ability disclosures open and lets each collapse and reopen", async () => {
    const { container } = renderSurface();

    expect(screen.queryByText("View rules")).not.toBeInTheDocument();
    const activityHeading = screen
      .getByText("Dream Bargain")
      .closest<HTMLElement>(".creature-sheet__activity-heading");
    expect(activityHeading).not.toBeNull();
    expect(within(activityHeading!).getByText("Two actions")).toBeInTheDocument();
    expect(activityHeading?.children).toHaveLength(2);
    const actionControl = screen.getByRole("button", { name: /Dream Bargain/ });
    expect(actionControl).toHaveAttribute("aria-expanded", "true");
    expect(actionControl).toHaveAccessibleName("Dream Bargain, Two actions");
    const action = actionControl.closest<HTMLElement>(".ant-collapse-item")!;
    const actionTrait = within(action).getByText("Mental");
    expect(actionTrait).toBeVisible();
    expect(within(action).getByText("Occult")).toBeVisible();
    expect(screen.getByText("Will DC 28")).toBeVisible();
    expect(container.querySelector(".creature-sheet__activity hr")).toBeInTheDocument();

    fireEvent.click(actionControl);
    expect(actionControl).toHaveAttribute("aria-expanded", "false");
    await waitFor(() => expect(actionTrait).not.toBeVisible());
    await waitFor(() => expect(screen.getByText("Will DC 28")).not.toBeVisible());
    fireEvent.click(actionControl);
    expect(actionControl).toHaveAttribute("aria-expanded", "true");
    await waitFor(() => expect(actionTrait).toBeVisible());
    await waitFor(() => expect(screen.getByText("Will DC 28")).toBeVisible());

    const abilityControl = screen.getByRole("button", { name: /Dream Haunting/ });
    expect(abilityControl).toHaveAttribute("aria-expanded", "true");
    expect(abilityControl).toHaveAccessibleName("Dream Haunting, Passive");
    const ability = abilityControl.closest<HTMLElement>(".ant-collapse-item")!;
    const abilityTrait = within(ability).getByText("Occult");
    expect(abilityTrait).toBeVisible();
    expect(screen.getByText("The haunting follows the sleeper.")).toBeVisible();
    fireEvent.click(abilityControl);
    expect(abilityControl).toHaveAttribute("aria-expanded", "false");
    await waitFor(() => expect(abilityTrait).not.toBeVisible());
    await waitFor(() =>
      expect(screen.getByText("The haunting follows the sleeper.")).not.toBeVisible(),
    );
    fireEvent.click(abilityControl);
    expect(abilityControl).toHaveAttribute("aria-expanded", "true");
    await waitFor(() => expect(abilityTrait).toBeVisible());
    await waitFor(() =>
      expect(screen.getByText("The haunting follows the sleeper.")).toBeVisible(),
    );
  });

  it("opens typed spell content from compact occurrence links", async () => {
    renderSurface();

    expect(screen.getByRole("heading", { name: "Spells" })).toBeInTheDocument();
    expect(screen.queryByText("Details", { exact: true })).not.toBeInTheDocument();
    const innateGroup = screen.getByRole("button", {
      name: /Occult Innate Spells/,
    });
    const covenGroup = screen.getByRole("button", { name: /Coven Spells/ });
    const standaloneGroup = screen.getByRole("button", { name: /Standalone/ });
    expect(innateGroup).toHaveAttribute("aria-expanded", "true");
    expect(covenGroup).toHaveAttribute("aria-expanded", "true");
    expect(standaloneGroup).toHaveAttribute("aria-expanded", "true");
    expect(screen.getAllByText("5th")).toHaveLength(2);
    fireEvent.click(innateGroup);
    expect(innateGroup).toHaveAttribute("aria-expanded", "false");
    expect(covenGroup).toHaveAttribute("aria-expanded", "true");
    expect(standaloneGroup).toHaveAttribute("aria-expanded", "true");
    fireEvent.click(innateGroup);
    expect(innateGroup).toHaveAttribute("aria-expanded", "true");
    const innateEntry = innateGroup.closest<HTMLElement>(".ant-collapse-item")!;
    const innateRoster = innateEntry.querySelector<HTMLElement>(
      ".creature-sheet__spell-links",
    )!;
    expect(innateRoster.querySelectorAll(".creature-sheet__spell-link")).toHaveLength(
      2,
    );
    expect(
      within(innateRoster).getByRole("link", { name: "Dream Message" }),
    ).toBeVisible();
    expect(within(innateRoster).getByRole("link", { name: "Nightmare" })).toBeVisible();
    expect(
      innateRoster.querySelector(".creature-sheet__spell-separator"),
    ).toHaveTextContent(",");
    const innateLink = screen.getAllByRole("link", { name: "Dream Message" })[0]!;
    expect(innateLink).toHaveAttribute("aria-haspopup", "dialog");
    expect(innateLink).toHaveAttribute("aria-expanded", "false");
    expect(
      screen.queryByText("The innate message reaches a sleeper."),
    ).not.toBeInTheDocument();
    fireEvent.pointerDown(innateLink);
    fireEvent.focus(innateLink);
    fireEvent.click(innateLink);
    expect(innateLink).toHaveAttribute("aria-expanded", "true");
    const innatePopover = screen.getByRole("dialog", {
      name: "Dream Message spell details",
    });
    expect(
      within(innatePopover).getByText("The innate message reaches a sleeper."),
    ).toBeInTheDocument();
    expect(within(innatePopover).getByText(/^5th ·/)).toBeInTheDocument();
    fireEvent.keyDown(innateLink, { key: "Escape" });
    expect(innateLink).toHaveAttribute("aria-expanded", "false");
    await waitFor(() => expect(document.activeElement).toBe(innateLink));
    fireEvent.click(innateLink);
    fireEvent.click(
      screen.getByRole("button", {
        name: "Open spell record",
      }),
    );
    expect(onReference.mock.calls[onReference.mock.calls.length - 1]?.[0]).toBe(
      "spells:dream-message",
    );

    expect(screen.getAllByRole("link", { name: "Dream Message" })).toHaveLength(2);
    const covenSpell = screen.getAllByRole("link", { name: "Dream Message" })[1]!;
    fireEvent.click(covenSpell);
    const covenPopover = screen.getByRole("dialog", {
      name: "Dream Message spell details",
    });
    expect(
      within(covenPopover).getByText("The coven repeats the same authored spell."),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("The innate message reaches a sleeper."),
    ).not.toBeInTheDocument();

    fireEvent.click(standaloneGroup);
    expect(standaloneGroup).toHaveAttribute("aria-expanded", "false");
    expect(innateGroup).toHaveAttribute("aria-expanded", "true");
    expect(covenGroup).toHaveAttribute("aria-expanded", "true");
    fireEvent.click(standaloneGroup);
    expect(standaloneGroup).toHaveAttribute("aria-expanded", "true");
    const standalone = standaloneGroup.closest<HTMLElement>(".ant-collapse-item")!;
    expect(within(standalone).getByText("8th")).toBeInTheDocument();
    expect(
      within(standalone).getAllByRole("link", { name: "Control Weather" }),
    ).toHaveLength(2);
    const standaloneLinks = within(standalone).getAllByRole("link", {
      name: "Control Weather",
    });
    expect(standaloneLinks[0]).toHaveAttribute("aria-expanded", "false");
    expect(standaloneLinks[1]).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByText("You alter the weather.")).not.toBeInTheDocument();

    fireEvent.click(standaloneLinks[0]!);
    const standalonePopover = screen.getByRole("dialog", {
      name: "Control Weather spell details",
    });
    expect(within(standalonePopover).getByText("8th")).toBeInTheDocument();
    expect(
      within(standalonePopover).getByText("You alter the weather."),
    ).toBeInTheDocument();
    expect(screen.queryByText("A second ritual occurrence.")).not.toBeInTheDocument();

    fireEvent.mouseDown(document.body);
    await waitFor(() =>
      expect(standaloneLinks[0]).toHaveAttribute("aria-expanded", "false"),
    );

    fireEvent.click(standaloneLinks[1]!);
    expect(
      within(
        screen.getByRole("dialog", { name: "Control Weather spell details" }),
      ).getByText("A second ritual occurrence."),
    ).toBeInTheDocument();
  });

  it("aligns creature and encounter resources without changing encounter vitals", () => {
    const { unmount } = renderSurface();
    const recordResources = screen.getByRole("heading", {
      name: "Resources",
    }).parentElement;
    expect(recordResources).not.toBeNull();
    const recordRow = within(recordResources!).getByText("Dream tokens").parentElement;
    expect(recordRow).toHaveClass("record-key-value-list__row");
    expect(within(recordRow!).getByText("3")).toBeInTheDocument();
    unmount();

    const surface = detailedSurfaceFixture();
    surface.profile = "encounter_participant";
    const runtime = encounterRuntimeFixture({ temporaryHp: 2 });
    const numberFact = (label: string, value: number) => ({
      label,
      base_value: value,
      adjusted_value: value,
      provenance: { source: { source_type: "canonical_record" as const } },
    });
    runtime.resources = [
      {
        resource_id: "dream-token",
        label: "Dream tokens",
        current: numberFact("Current dream tokens", 1),
        maximum: numberFact("Maximum dream tokens", 3),
      },
    ];
    surface.encounter = runtime;
    render(<RecordSurface onReference={onReference} surface={surface} />);

    const encounterResources = screen.getByRole("heading", {
      name: "Resources",
    }).parentElement;
    const encounterRow = within(encounterResources!).getByText(
      "Dream tokens",
    ).parentElement;
    expect(encounterRow).toHaveClass("record-key-value-list__row");
    expect(encounterRow).toHaveTextContent("Dream tokens1/3");
    expect(
      screen.getByText("+2 temporary").closest(".record-surface__runtime-vitals"),
    ).not.toBeNull();
  });

  it("renders residual Heartstone content once in Overview without reclaiming occurrence content", () => {
    renderSurface();

    const overview = screen
      .getByRole("heading", { name: "Overview" })
      .closest("section");
    expect(overview).not.toBeNull();
    expect(within(overview!).getAllByText("Heartstone")).toHaveLength(1);
    expect(within(overview!).queryByText("Dream Bargain")).not.toBeInTheDocument();
    expect(
      screen.queryByText("The heartstone lets the envoy use ethereal jaunt."),
    ).not.toBeInTheDocument();

    fireEvent.click(within(overview!).getByRole("button", { name: "Heartstone" }));
    expect(
      screen.getAllByText("The heartstone lets the envoy use ethereal jaunt."),
    ).toHaveLength(1);

    fireEvent.click(screen.getByText("Dream Bargain"));
    expect(screen.getAllByText("The envoy offers a bargain.")).toHaveLength(1);
    expect(
      within(overview!).queryByText("The envoy offers a bargain."),
    ).not.toBeInTheDocument();
  });

  it("renders hardness, shield facts, and save details without duplicating owners", () => {
    renderSurface();

    const defenses = screen.getByRole("heading", {
      name: "Defenses & Vitals",
    }).parentElement!;
    expect(within(defenses).getByText("8")).toBeInTheDocument();
    const shield = within(defenses).getByRole("heading", {
      name: "Shield",
    }).parentElement!;
    expect(within(shield).getByText("+2")).toBeInTheDocument();
    expect(within(shield).getByText("5")).toBeInTheDocument();
    expect(within(shield).getByText("20")).toBeInTheDocument();
    expect(within(shield).getByText("10")).toBeInTheDocument();
    expect(within(defenses).getByText("+1 against curses")).toBeInTheDocument();
    expect(within(defenses).getByText("+2 against traps")).toBeInTheDocument();
    expect(within(defenses).getByText("+1 against magic")).toBeInTheDocument();
  });

  it("renders user-facing action details without raw effect or category slugs", () => {
    const { container } = renderSurface();
    const actions = screen.getByRole("heading", { name: "Actions" }).parentElement!;
    const features = screen.getByRole("heading", {
      name: "Passives",
    }).parentElement!;

    const claw = screen
      .getByText("Claw")
      .closest<HTMLElement>(".creature-sheet__activity")!;
    expect(actions).toContainElement(claw);
    expect(features).not.toContainElement(claw);
    expect(within(claw).getByText("One action")).toHaveClass("sr-only");
    expect(within(claw).queryByText("abyssal-plague")).not.toBeInTheDocument();
    expect(within(claw).queryByText("Attack effects")).not.toBeInTheDocument();
    expect(within(claw).queryByText("offensive")).not.toBeInTheDocument();

    const bargain = screen
      .getByText("Dream Bargain")
      .closest<HTMLElement>(".creature-sheet__activity")!;
    for (const value of [
      "Two actions",
      "1 per minute",
      "The envoy can see the target.",
      "One dream token",
      "3 maximum",
      "Effect: Dream veil",
    ]) {
      expect(within(bargain).getByText(value)).toBeInTheDocument();
    }
    expect(within(bargain).queryByText("Category")).not.toBeInTheDocument();
    expect(within(bargain).queryByText("offensive")).not.toBeInTheDocument();
    expect(within(bargain).queryByText("PT1M")).not.toBeInTheDocument();
    expect(actions).toContainElement(bargain);
    const haunting = screen
      .getByText("Dream Haunting")
      .closest<HTMLElement>(".creature-sheet__activity")!;
    expect(features).toContainElement(haunting);
    expect(actions).not.toContainElement(haunting);
    expect(within(haunting).getByText("Passive")).toBeVisible();
    expect(haunting).toHaveClass("creature-sheet__activity--expandable");

    const ambush = screen
      .getByText("Spell Ambush")
      .closest<HTMLElement>(".creature-sheet__activity")!;
    expect(features).toContainElement(ambush);
    expect(actions).not.toContainElement(ambush);
    expect(within(ambush).getByText("Passive")).toBeVisible();
    expect(within(ambush).queryByRole("button")).not.toBeInTheDocument();
    expect(
      ambush.querySelector(":scope > .creature-sheet__activity-summary"),
    ).toBeInTheDocument();
    expect(bargain).toHaveClass("creature-sheet__activity--expandable");
    expect(bargain.querySelector(".ant-collapse-expand-icon")).toBeInTheDocument();
    expect(container.querySelectorAll(".action-glyph__mark")).toHaveLength(2);
  });

  it("renders ranked slots, occurrence context, ritual DC, gear, and Lore", () => {
    renderSurface();

    expect(screen.getByText("2 slots")).toBeInTheDocument();
    expect(screen.getByText("1 use")).toBeInTheDocument();
    expect(screen.queryByText(/slot5:0/)).not.toBeInTheDocument();
    expect(screen.queryByText(/qg3r6OKHjX8qHiNS/)).not.toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Rituals" })).toBeInTheDocument();
    expect(screen.getByText("Ritual DC")).toBeInTheDocument();
    expect(screen.getByText("31")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Equipment & Gear" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Silver key")).toBeInTheDocument();
    expect(
      screen.getByText("Quantity 1 · Level 9 · Usage held · 2 maximum uses"),
    ).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Lore" })).toBeInTheDocument();
    expect(screen.getByText("Dream Lore")).toBeInTheDocument();
    expect(screen.getByText("+20")).toBeInTheDocument();
  });

  it("keeps optional absence and known-empty data silent", () => {
    const surface = detailedSurfaceFixture();
    if (surface.presentation.presentation_type !== "creature") {
      throw new Error("Fixture must be a creature surface");
    }
    surface.presentation.body.unavailable_domains = {
      skills: { causes: [] },
      equipment: { causes: [] },
    };
    surface.presentation.body.unmodeled_skills = [];

    render(<RecordSurface onReference={onReference} surface={surface} />);

    expect(
      screen.queryByRole("button", { name: "Data availability" }),
    ).not.toBeInTheDocument();
  });

  it("renders only legitimate availability causes with exact hostile skill copy", () => {
    const hostileKey = '<img src=x onerror="alert(1)">\n+20';
    const surface = detailedSurfaceFixture();
    if (surface.presentation.presentation_type !== "creature") {
      throw new Error("Fixture must be a creature surface");
    }
    const provenance = {
      owner: "canonical_creature" as const,
      field: "skills" as const,
    };
    surface.presentation.body.unavailable_domains = {
      defenses: {
        causes: [
          {
            state: "missing",
            field: "shield_hardness",
            provenance,
            message: "Shield hardness is unavailable.",
          },
        ],
      },
      spellcasting: {
        causes: [
          {
            state: "null",
            field: "spell_slot_maximum",
            provenance,
            message: "Spell slot maximum is unavailable.",
          },
        ],
      },
      skills: {
        causes: [
          {
            state: "unsupported",
            field: "unmodeled_skill",
            component_id: "opaque:skill:internal",
            provenance,
            unmodeled_skill: {
              component_id: "opaque:skill:internal",
              authored_order: 0,
              authored_key: hostileKey,
              base: { state: "null" },
              reason: "unknown_authored_key",
            },
            message: "The source supplied an unrecognized skill key.",
          },
        ],
      },
    };

    const { container } = render(
      <RecordSurface onReference={onReference} surface={surface} />,
    );
    const control = screen.getByRole("button", { name: "Data availability" });
    expect(control).toHaveAttribute("aria-expanded", "false");
    fireEvent.click(control);

    expect(screen.getByText("Missing")).toBeInTheDocument();
    expect(screen.getByText("Null")).toBeInTheDocument();
    expect(screen.getByText("Unsupported")).toBeInTheDocument();
    expect(screen.getByText("Shield hardness is unavailable.")).toBeInTheDocument();
    expect(screen.getByText("Spell slot maximum is unavailable.")).toBeInTheDocument();
    expect(screen.getByText("Unmodeled skill entry")).toBeInTheDocument();
    const authoredKey = container.querySelector(".creature-sheet__authored-value code");
    expect(authoredKey).toBeInTheDocument();
    expect(authoredKey?.textContent).toBe(hostileKey);
    expect(
      screen.getAllByText("The source supplied an unrecognized skill key.", {
        exact: true,
      }),
    ).toHaveLength(1);
    expect(container.querySelector("img")).not.toBeInTheDocument();
    expect(screen.queryByText("opaque:skill:internal")).not.toBeInTheDocument();
    expect(screen.queryByText(/meaning was inferred/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/not modeled/i)).not.toBeInTheDocument();
  });

  it("renders compact teaser and all six B3 scan facts in a responsive semantic grid", () => {
    const surface = detailedSurfaceFixture();
    surface.profile = "search_compact";

    const { container } = render(
      <RecordSurface onReference={onReference} surface={surface} />,
    );

    expect(
      screen.getByText("A dream-coven envoy that bargains before battle."),
    ).toBeVisible();
    const facts = container.querySelector(".record-surface-search__facts")!;
    expect(facts.tagName).toBe("DL");
    for (const label of ["Perception", "AC", "HP", "Fort", "Ref", "Will"]) {
      expect(
        within(facts as HTMLElement).getByText(label, { exact: true }),
      ).toBeVisible();
    }
    expect(
      container.querySelector(".record-surface--search-compact"),
    ).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Overview" })).not.toBeInTheDocument();
  });

  it("keeps the remastered counterpart action inside the legacy notice", () => {
    const airMephit = detailedSurfaceFixture();
    airMephit.metadata.title = "Air Mephit";
    airMephit.metadata.edition = {
      status: "legacy",
      counterparts: [
        {
          role: "remastered_counterpart",
          record_key: AIR_SCAMP_KEY,
          title: "Air Scamp",
        },
      ],
    };
    const { container } = render(
      <RecordSurface onReference={onReference} surface={airMephit} />,
    );
    const notice = screen
      .getByText("This record uses legacy rules.")
      .closest<HTMLElement>(".ant-alert")!;
    const action = within(notice).getByRole("button", {
      name: "View remastered Air Scamp",
    });
    expect(action).toBeVisible();
    expect(container.querySelector(".creature-sheet__related-edition")).toBeNull();
    fireEvent.click(action);
    expect(onReference).toHaveBeenLastCalledWith(AIR_SCAMP_KEY);
  });

  it("places the legacy counterpart in remastered header metadata", () => {
    const airScamp = detailedSurfaceFixture();
    airScamp.metadata.title = "Air Scamp";
    airScamp.metadata.edition = {
      status: "remaster",
      counterparts: [
        {
          role: "legacy_counterpart",
          record_key: AIR_MEPHIT_KEY,
          title: "Air Mephit",
        },
      ],
    };
    const { container } = render(
      <RecordSurface onReference={onReference} surface={airScamp} />,
    );
    expect(
      screen.queryByText("This record uses remastered rules."),
    ).not.toBeInTheDocument();
    expect(screen.queryByText("Remastered", { exact: true })).not.toBeInTheDocument();
    expect(screen.queryByText("Legacy", { exact: true })).not.toBeInTheDocument();
    expect(container.querySelector(".creature-sheet__edition-notice")).toBeNull();
    const header = screen
      .getByRole("heading", { name: "Air Scamp" })
      .closest<HTMLElement>(".creature-sheet__header")!;
    const related = within(header)
      .getByText("Related edition")
      .closest<HTMLElement>(".creature-sheet__related-edition")!;
    const action = within(related).getByRole("button", {
      name: "View legacy Air Mephit",
    });
    expect(action).toHaveClass("ant-btn-link");
    fireEvent.click(action);
    expect(onReference).toHaveBeenLastCalledWith(AIR_MEPHIT_KEY);
  });

  it("places record identity only in the inline provenance disclosure", () => {
    renderSurface();

    expect(screen.queryByText("concept:f1-record")).not.toBeInTheDocument();
    expect(screen.queryByText("Skill source keys")).not.toBeInTheDocument();
    expect(screen.queryByText("occultism")).not.toBeInTheDocument();
    fireEvent.click(screen.getByText("References & Source"));
    expect(screen.getByText("Record ID")).toBeInTheDocument();
    expect(screen.getByText("concept:f1-record")).toBeInTheDocument();
    expect(screen.getByText("Skill source keys")).toBeInTheDocument();
    expect(screen.getByText("occultism")).toBeInTheDocument();
    expect(screen.queryByText("source contract")).not.toBeInTheDocument();
    expect(screen.queryByText("upstream commit")).not.toBeInTheDocument();
  });

  it("uses aligned key-value grids for natural definition groups", () => {
    const { container } = renderSurface();

    fireEvent.click(screen.getByText("References & Source"));
    for (const label of [
      "Immunities",
      "Weaknesses",
      "Resistances",
      "Senses",
      "Languages",
      "Record ID",
    ]) {
      const row = screen.getByText(label, { exact: true }).parentElement;
      expect(row).toHaveClass("record-key-value-list__row");
      expect(row?.children).toHaveLength(2);
    }
    expect(screen.getByText("Record ID").closest(".record-key-value-list")).toHaveClass(
      "record-key-value-list--provenance",
    );
    expect(
      container.querySelectorAll(".record-key-value-list").length,
    ).toBeGreaterThanOrEqual(3);
    for (const label of ["AC", "HP", "Fortitude", "Reflex", "Will"]) {
      for (const element of screen.getAllByText(label, { exact: true })) {
        expect(element.closest(".record-key-value-list")).toBeNull();
      }
    }
    const perceptionLabel = screen
      .getAllByText("Perception", { exact: true })
      .find((element) => element.tagName === "DT")!;
    expect(perceptionLabel.closest(".record-key-value-list")).toHaveAttribute(
      "aria-label",
      "Perception and senses",
    );
  });

  it("keeps record-styled description secondary in the encounter profile", () => {
    const surface = detailedSurfaceFixture();
    surface.profile = "encounter_participant";
    surface.encounter = encounterRuntimeFixture();

    render(<RecordSurface onReference={onReference} surface={surface} />);

    const combatSnapshot = screen.getByRole("heading", {
      name: "Combat Snapshot",
    });
    const description = screen.getByRole("heading", {
      name: "Description & Lore",
    });
    const narrative = description.closest(".creature-sheet__narrative");
    expect(narrative).not.toBeNull();
    expect(narrative).toHaveTextContent("A quick goblin scout");
    expect(
      combatSnapshot.compareDocumentPosition(description) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  });

  it("separates runtime labels from values and exposes typed spell details", async () => {
    const onSpellCast = vi.fn();
    const surface = detailedSurfaceFixture();
    surface.profile = "encounter_participant";
    const runtime = encounterRuntimeFixture();
    const provenance = {
      source: { source_type: "canonical_record" as const },
    };
    runtime.movement = {
      speeds: [
        {
          movement_type: "land",
          label: "Land Speed",
          base_value_feet: 25,
          adjusted_value_feet: 25,
          provenance,
        },
      ],
    };
    runtime.skills = [
      {
        skill_id: "skill:occultism",
        label: "Occultism",
        kind: { kind: "standard", slug: "occultism" },
        modifier: {
          label: "Occultism",
          base_value: 20,
          adjusted_value: 20,
          provenance,
        },
      },
    ];
    if (runtime.action_budget) {
      runtime.action_budget.can_act = {
        available: false,
        reason: "A condition prevents actions.",
      };
    }
    runtime.activities = [
      {
        activity_id: "runtime-claw",
        label: "Runtime Claw",
        kind: "strike",
        usage: "unlimited",
        content: [
          spellContent(
            "runtime-claw-content",
            "Runtime Claw",
            "The creature makes a vicious claw attack.",
          ),
        ],
        action_cost: {
          value: { kind: "actions", count: 1 },
          provenance,
        },
        provenance,
      },
      {
        activity_id: "nightmare-aura",
        label: "Nightmare Aura",
        kind: "other",
        usage: "unlimited",
        action_cost: {
          value: { kind: "passive" },
          provenance,
        },
        provenance,
      },
    ];
    runtime.spellcasting = [
      {
        entry_id: "occult-innate",
        authored_order: 0,
        label: "Occult Innate Spells",
        tradition: "occult",
        preparation: "innate",
        attack: {
          roll_id: "occult-innate-attack",
          label: "Spell attack",
          base_value: 20,
          adjusted_value: 20,
          surface: "attack_roll",
          provenance,
        },
        dc: {
          roll_id: "occult-innate-dc",
          label: "Spell DC",
          base_value: 28,
          adjusted_value: 28,
          surface: "dc",
          provenance,
        },
        spells: [
          {
            occurrence_id: "dream-message",
            authored_order: 0,
            label: "Dream Message",
            target_record_key: "spells:dream-message",
            rank: 5,
            traits: ["concentrate", "mental"],
            content: [
              spellContent(
                "runtime-dream-message",
                "Dream Message",
                "The message reaches a sleeper.",
              ),
            ],
            activity: {
              activity_id: "dream-message",
              label: "Dream Message",
              kind: "spell",
              usage: "unlimited",
              action_cost: {
                value: { kind: "actions", count: 2 },
                provenance,
              },
              provenance,
            },
            cast: {
              spend_target: {
                target_type: "innate_use",
                entry_id: "occult-innate",
                spell_occurrence_id: "dream-message",
              },
              available: true,
              state: {
                state_type: "tracked",
                maximum: 2,
                initial_remaining: 2,
                remaining: 2,
              },
            },
            provenance,
          },
        ],
      },
    ];
    surface.encounter = runtime;

    render(
      <RecordSurface
        onReference={onReference}
        onSpellCast={onSpellCast}
        surface={surface}
      />,
    );

    const movement = screen.getByLabelText("Movement");
    expect(movement).toHaveClass("record-key-value-list");
    const movementRow = movement.querySelector(".record-key-value-list__row");
    expect(movementRow?.children[0]?.tagName).toBe("DT");
    expect(movementRow?.children[1]?.tagName).toBe("DD");
    expect(screen.getByRole("list", { name: "Skills" })).toHaveClass(
      "creature-sheet__skill-grid",
    );
    expect(screen.getByText("Land Speed").nextElementSibling).toHaveTextContent(
      "25 ft",
    );
    expect(screen.getByText("Occultism").parentElement).toHaveTextContent("+20");
    expect(screen.getByRole("list", { name: "Languages" })).toHaveClass(
      "creature-sheet__compact-multi-value-list",
    );
    expect(screen.getByText("Cannot act").parentElement).toHaveTextContent(
      "A condition prevents actions.",
    );
    expect(
      screen.queryByRole("button", { name: "Actions adjustment details" }),
    ).not.toBeInTheDocument();

    const actions = screen.getByRole("heading", { name: "Actions" });
    const spellcasting = screen.getByRole("heading", { name: "Spellcasting" });
    const features = screen.getByRole("heading", { name: "Passives" });
    expect(actions.closest("section")).toHaveTextContent("Runtime Claw");
    expect(actions.closest("section")).not.toHaveTextContent("Nightmare Aura");
    expect(features.closest("section")).toHaveTextContent("Nightmare Aura");
    expect(features.closest("section")).not.toHaveTextContent("Runtime Claw");
    expect(
      actions.compareDocumentPosition(spellcasting) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      spellcasting.compareDocumentPosition(features) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();

    const actionDisclosure = screen.getByRole("button", {
      name: /Runtime Claw/,
    });
    expect(actionDisclosure).toHaveAttribute("aria-expanded", "true");
    fireEvent.click(actionDisclosure);
    expect(actionDisclosure).toHaveAttribute("aria-expanded", "false");
    fireEvent.click(actionDisclosure);
    expect(actionDisclosure).toHaveAttribute("aria-expanded", "true");

    const spellcastingDisclosure = screen.getByRole("button", {
      name: /Occult Innate Spells/,
    });
    const spellcastingSection = spellcasting.closest("section");
    if (!spellcastingSection) throw new Error("Spellcasting section was not rendered");
    expect(
      within(spellcastingSection).getAllByText("DC", { exact: true }),
    ).toHaveLength(1);
    expect(
      within(spellcastingSection).getAllByText("Spell attack", { exact: true }),
    ).toHaveLength(1);
    expect(spellcastingDisclosure).not.toHaveTextContent("DC 28");
    expect(spellcastingDisclosure).not.toHaveTextContent("attack +20");
    expect(spellcastingDisclosure).toHaveAttribute("aria-expanded", "true");
    fireEvent.click(spellcastingDisclosure);
    expect(spellcastingDisclosure).toHaveAttribute("aria-expanded", "false");
    fireEvent.click(spellcastingDisclosure);
    expect(spellcastingDisclosure).toHaveAttribute("aria-expanded", "true");
    const spellTrigger = screen.getByRole("link", { name: "Dream Message" });
    const runtimeSpell = spellTrigger.closest(".encounter-runtime-spell");
    expect(runtimeSpell?.querySelector(".action-glyph__mark")).toHaveTextContent("2");
    expect(runtimeSpell).toHaveTextContent("Two actions");
    expect(within(runtimeSpell as HTMLElement).getByText("Two actions")).toHaveClass(
      "sr-only",
    );
    expect(spellTrigger).toHaveAttribute("aria-expanded", "false");
    fireEvent.click(spellTrigger);
    expect(spellTrigger).toHaveAttribute("aria-expanded", "true");
    const preview = await screen.findByRole("dialog", {
      name: "Dream Message spell details",
    });
    const previewHeader = preview
      .closest(".ant-popover")
      ?.querySelector(".preview-popover__header");
    expect(previewHeader?.querySelector(".action-glyph__mark")).toHaveTextContent("2");
    expect(within(previewHeader as HTMLElement).getByText("Two actions")).toHaveClass(
      "sr-only",
    );
    expect(
      within(preview).getByText("The message reaches a sleeper."),
    ).toBeInTheDocument();
    expect(within(preview).getByText("5th · Concentrate · Mental")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Open spell record" }),
    ).toBeInTheDocument();
    expect(within(preview).getByText("2 of 2 remaining")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Cast Dream Message" }));
    expect(onSpellCast).toHaveBeenCalledWith({
      spell_occurrence_id: "dream-message",
      spend_target: {
        target_type: "innate_use",
        entry_id: "occult-innate",
        spell_occurrence_id: "dream-message",
      },
      operation: "cast_one",
    });
    expect(
      screen.getByRole("button", {
        name: "Restore one use of Dream Message",
      }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", {
        name: "Restore one use of Dream Message",
      }),
    ).toHaveAccessibleDescription("Already at the creation baseline");
  });

  it("renders backend-authored spell availability and submits typed cast and restore operations", () => {
    const surface = detailedSurfaceFixture();
    surface.profile = "encounter_participant";
    const runtime = encounterRuntimeFixture();
    const provenance = {
      source: { source_type: "canonical_record" as const },
    };
    runtime.spellcasting = [
      {
        entry_id: "occult-innate",
        authored_order: 0,
        label: "Occult Innate Spells",
        spells: [
          {
            occurrence_id: "limited",
            authored_order: 0,
            label: "Limited Spell",
            cast: {
              spend_target: {
                target_type: "innate_use",
                entry_id: "occult-innate",
                spell_occurrence_id: "limited",
              },
              available: false,
              state: {
                state_type: "tracked",
                maximum: 2,
                initial_remaining: 2,
                remaining: 0,
              },
              blocked_reason: "exhausted",
            },
            provenance,
          },
          {
            occurrence_id: "at-will",
            authored_order: 1,
            label: "At-Will Spell",
            cast: {
              spend_target: { target_type: "at_will" },
              available: true,
              state: { state_type: "at_will" },
            },
            provenance,
          },
          {
            occurrence_id: "focus",
            authored_order: 2,
            label: "Focus Spell",
            cast: {
              spend_target: {
                target_type: "focus_pool",
                resource_id: "resource:focus",
              },
              available: false,
              state: { state_type: "unavailable", reason: "missing_current" },
              blocked_reason: "state_unavailable",
            },
            provenance,
          },
        ],
      },
    ];
    surface.encounter = runtime;
    const onSpellCast = vi.fn();

    render(
      <RecordSurface
        onReference={onReference}
        onSpellCast={onSpellCast}
        surface={surface}
      />,
    );
    expect(
      screen.getByRole("button", { name: /Occult Innate Spells/ }),
    ).toHaveAttribute("aria-expanded", "true");

    expect(screen.getByText("0 of 2 remaining")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Limited Spell" }));
    let preview = screen.getByRole("dialog", {
      name: "Limited Spell spell details",
    });
    expect(within(preview).getByText("0 of 2 remaining")).toBeInTheDocument();
    const limitedCast = screen.getByRole("button", { name: "Cast Limited Spell" });
    expect(limitedCast).toBeDisabled();
    expect(limitedCast).toHaveAccessibleDescription(
      "Cast unavailable at 0 remaining uses",
    );
    expect(screen.queryByText("No uses remaining")).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", {
        name: "Restore one use of Limited Spell",
      }),
    ).toBeEnabled();
    fireEvent.click(
      screen.getByRole("button", {
        name: "Restore one use of Limited Spell",
      }),
    );
    expect(onSpellCast).toHaveBeenLastCalledWith({
      spell_occurrence_id: "limited",
      spend_target: {
        target_type: "innate_use",
        entry_id: "occult-innate",
        spell_occurrence_id: "limited",
      },
      operation: "restore_one",
    });
    fireEvent.click(screen.getByRole("button", { name: "Close spell preview" }));

    fireEvent.click(screen.getByRole("button", { name: "At-Will Spell" }));
    preview = screen.getByRole("dialog", {
      name: "At-Will Spell spell details",
    });
    expect(within(preview).getByText("At will")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Cast At-Will Spell" }));
    expect(onSpellCast).toHaveBeenLastCalledWith({
      spell_occurrence_id: "at-will",
      spend_target: { target_type: "at_will" },
      operation: "cast_one",
    });
    expect(screen.getAllByText("At will").length).toBeGreaterThan(0);
    expect(
      screen.getByRole("button", {
        name: "Restore one use of At-Will Spell",
      }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", {
        name: "Restore one use of At-Will Spell",
      }),
    ).toHaveAccessibleDescription("At-will spells do not consume uses");
    fireEvent.click(screen.getByRole("button", { name: "Close spell preview" }));

    fireEvent.click(screen.getByRole("button", { name: "Focus Spell" }));
    preview = screen.getByRole("dialog", { name: "Focus Spell spell details" });
    expect(
      within(preview).getByText(/Unavailable: current uses were not provided/),
    ).toBeInTheDocument();
    const focusCast = screen.getByRole("button", { name: "Cast Focus Spell" });
    expect(focusCast).toBeDisabled();
    expect(focusCast).toHaveAccessibleDescription("Casting state unavailable");
    expect(
      screen.getByRole("button", {
        name: "Restore one use of Focus Spell",
      }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", {
        name: "Restore one use of Focus Spell",
      }),
    ).toHaveAccessibleDescription("Tracked uses are unavailable");
  });

  it("keeps static record profiles free of encounter mutation slots", () => {
    const surface = detailedSurfaceFixture();
    surface.encounter = encounterRuntimeFixture();

    render(
      <RecordSurface
        onReference={onReference}
        onSpellCast={vi.fn()}
        slots={{
          conditions: <button>Mutate conditions</button>,
          header: <button>Mutate participant state</button>,
          header_actions: <button>Mutate variant</button>,
          notes: <textarea aria-label="Participant note" />,
          vitals: <button>Mutate HP</button>,
        }}
        surface={surface}
      />,
    );

    expect(screen.queryByText("Mutate conditions")).not.toBeInTheDocument();
    expect(screen.queryByText("Mutate participant state")).not.toBeInTheDocument();
    expect(screen.queryByText("Mutate variant")).not.toBeInTheDocument();
    expect(screen.queryByText("Mutate HP")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Participant note")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /^Cast / })).not.toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Defenses & Vitals" })).toBeVisible();
  });
});

function renderSurface() {
  return render(
    <RecordSurface onReference={onReference} surface={detailedSurfaceFixture()} />,
  );
}

function detailedSurfaceFixture(): RecordSurfaceView {
  const surface = creatureSurfaceFixture({
    level: 9,
    recordKey: "concept:f1-record",
    title: "Dream-Coven Envoy",
    traits: ["fiend", "hag", "unholy"],
  });
  if (surface.presentation.presentation_type !== "creature") {
    throw new Error("Fixture must be a creature surface");
  }
  surface.metadata.edition = {
    status: "legacy",
    counterparts: [],
  };
  surface.presentation.body = {
    ...surface.presentation.body,
    teaser: "A dream-coven envoy that bargains before battle.",
    size: {
      value: "medium",
      provenance: { owner: "canonical_creature", field: "size" },
    },
    adjustment: {
      value: "elite",
      provenance: { owner: "canonical_creature", field: "adjustment" },
    },
    initiative: {
      statistic: "perception",
      provenance: { owner: "canonical_creature", field: "initiative" },
    },
    defenses: {
      ...surface.presentation.body.defenses!,
      hardness: 8,
      shield: {
        armor_class_bonus: 2,
        broken_threshold: 10,
        hardness: 5,
        maximum_hit_points: 20,
      },
      immunities: [
        {
          component_id: "sleep",
          authored_order: 0,
          kind: "sleep",
        },
      ],
      resistances: [
        {
          component_id: "fire",
          authored_order: 0,
          kind: "fire",
          amount: 5,
        },
      ],
      weaknesses: [
        {
          component_id: "cold-iron",
          authored_order: 0,
          kind: "cold iron",
          amount: 5,
        },
      ],
    },
    saves: {
      ...surface.presentation.body.saves!,
      fortitude: {
        ...surface.presentation.body.saves!.fortitude!,
        details: "+1 against curses",
      },
      reflex: {
        ...surface.presentation.body.saves!.reflex!,
        details: "+2 against traps",
      },
      all_saves_note: "+1 against magic",
    },
    awareness: {
      ...surface.presentation.body.awareness!,
      senses: [
        {
          component_id: "darkvision",
          authored_order: 0,
          kind: "darkvision",
        },
      ],
      languages: ["Aklo", "Common"],
    },
    abilities: {
      strength: 4,
      dexterity: 3,
      constitution: 4,
      intelligence: 5,
      wisdom: 4,
      charisma: 5,
      provenance: { owner: "canonical_creature", field: "legacy_abilities" },
    },
    movement: [
      {
        component_id: "land",
        authored_order: 0,
        mode: "land",
        label: "Speed",
        speed_feet: 25,
      },
      {
        component_id: "fly",
        authored_order: 1,
        mode: "fly",
        speed_feet: 40,
      },
    ],
    skills: [
      {
        component_id: "occultism",
        authored_order: 0,
        kind: "occultism",
        label: "Occultism",
        modifier: 22,
        variants: [
          {
            component_id: "dreams",
            authored_order: 0,
            label: "Dreams",
            modifier: 24,
            predicates: [{ predicate_type: "term", term: "dreams" }],
          },
          {
            component_id: "nightmares",
            authored_order: 1,
            label: "Nightmares",
            modifier: 23,
            predicates: [{ predicate_type: "term", term: "nightmares" }],
          },
        ],
        source_entries: [
          {
            authored_order: 0,
            authored_key: "occultism",
            modifier: { state: "value", value: 22 },
          },
        ],
      },
    ],
    activities: [
      ...(surface.presentation.body.activities ?? []).map((activity) => ({
        ...activity,
        attack_effects: ["abyssal-plague"],
        category: "offensive",
      })),
      {
        occurrence_id: "dream-bargain",
        authored_order: 1,
        provenance: occurrenceProvenance,
        activity_type: "action",
        label: "Dream Bargain",
        action_cost: { cost_type: "actions", count: 2 },
        traits: ["mental", "occult"],
        category: "offensive",
        frequency: { maximum: 1, period: "PT1M", display: "1 per minute" },
        requirements: "The envoy can see the target.",
        cost: "One dream token",
        uses: { maximum: 3 },
        self_effect: { label: "Effect", value: "Dream veil" },
        content: [activityContent()],
      },
      {
        occurrence_id: "dream-haunting",
        authored_order: 2,
        provenance: occurrenceProvenance,
        activity_type: "action",
        label: "Dream Haunting",
        action_cost: { cost_type: "passive" },
        traits: ["occult"],
        category: "offensive",
        content: [passiveActivityContent()],
      },
      {
        occurrence_id: "spell-ambush",
        authored_order: 3,
        provenance: occurrenceProvenance,
        activity_type: "action",
        label: "Spell Ambush",
        action_cost: { cost_type: "passive" },
        category: "offensive",
      },
    ],
    spellcasting: [
      {
        occurrence_id: "innate",
        authored_order: 0,
        provenance: occurrenceProvenance,
        label: "Occult Innate Spells",
        tradition: "occult",
        preparation: "innate",
        difficulty_class: 28,
        attack_modifier: 20,
        slots: [{ rank: 5, maximum: 2 }],
        spells: [
          {
            occurrence_id: "dream-message",
            authored_order: 0,
            provenance: occurrenceProvenance,
            label: "Dream Message",
            target_record_key: "spells:dream-message",
            rank: 5,
            context: {
              contextual_label: "At will",
              group: "innate",
              location: "qg3r6OKHjX8qHiNS",
              slot: "slot5:0",
              uses: { maximum: 1 },
            },
            content: [
              spellContent(
                "innate-dream-message",
                "Dream Message",
                "The innate message reaches a sleeper.",
              ),
            ],
          },
          {
            occurrence_id: "nightmare",
            authored_order: 1,
            provenance: occurrenceProvenance,
            label: "Nightmare",
            target_record_key: "spells:nightmare",
            rank: 5,
            context: { location: "qg3r6OKHjX8qHiNS" },
            content: [
              spellContent(
                "innate-nightmare",
                "Nightmare",
                "The nightmare follows the target into sleep.",
              ),
            ],
          },
        ],
      },
      {
        occurrence_id: "coven",
        authored_order: 1,
        provenance: occurrenceProvenance,
        label: "Coven Spells",
        tradition: "occult",
        spells: [
          {
            occurrence_id: "coven-dream-message",
            authored_order: 1,
            provenance: occurrenceProvenance,
            label: "Dream Message",
            target_record_key: "spells:dream-message",
            rank: 5,
            content: [
              spellContent(
                "coven-dream-message",
                "Dream Message",
                "The coven repeats the same authored spell.",
              ),
            ],
          },
        ],
      },
    ],
    standalone_spells: [
      {
        occurrence_id: "control-weather-repeat",
        authored_order: 2,
        provenance: occurrenceProvenance,
        label: "Control Weather",
        target_record_key: "spells:control-weather",
        rank: 8,
        content: [
          spellContent(
            "control-weather-repeat",
            "Control Weather",
            "A second ritual occurrence.",
          ),
        ],
      },
      {
        occurrence_id: "control-weather",
        authored_order: 1,
        provenance: occurrenceProvenance,
        label: "Control Weather",
        target_record_key: "spells:control-weather",
        rank: 8,
        content: [
          spellContent("control-weather", "Control Weather", "You alter the weather."),
        ],
      },
    ],
    resources: [
      {
        component_id: "dream-token",
        authored_order: 0,
        kind: "uses",
        label: "Dream tokens",
        maximum: 3,
      },
    ],
    rituals: {
      difficulty_class: 31,
      provenance: { owner: "canonical_creature", field: "embedded_entities" },
    },
    equipment: [
      {
        occurrence_id: "silver-key",
        authored_order: 0,
        provenance: occurrenceProvenance,
        label: "Silver key",
        traits: ["magical"],
        level: 9,
        usage: "held",
        quantity: 1,
        uses: { maximum: 2 },
      },
    ],
    lore: [
      {
        occurrence_id: "dream-lore",
        authored_order: 0,
        provenance: occurrenceProvenance,
        label: "Dream Lore",
        modifier: 20,
      },
    ],
    content: [
      ...(surface.presentation.body.content ?? []),
      spellContent(
        "heartstone",
        "Heartstone",
        "The heartstone lets the envoy use ethereal jaunt.",
      ),
    ],
    provenance: {
      source_path: "fixture.json",
      source_contract_version: "hidden-contract",
      source_system_version: "hidden-system",
      source_upstream_commit: "hidden-commit",
    },
  };
  return surface;
}

function spellContent(
  contentKey: string,
  label: string,
  text: string,
): CreatureSurfaceContentView {
  return {
    content_key: contentKey,
    role: "embedded_capability",
    authored_order: 0,
    label,
    blocks: [
      {
        block_type: "paragraph",
        spans: [{ span_type: "text", text }],
      },
    ],
    content_hash: contentKey,
    visibility: "public",
    provenance: {
      source_record_key: "concept:f1-record",
      relative_source_path: "fixture.json",
      field_family: "fixture.spell",
    },
  };
}

function activityContent(): CreatureSurfaceContentView {
  return {
    content_key: "dream-bargain-content",
    role: "embedded_capability",
    authored_order: 0,
    label: "Dream Bargain",
    blocks: [
      {
        block_type: "paragraph",
        spans: [{ span_type: "text", text: "The envoy offers a bargain." }],
      },
      {
        block_type: "paragraph",
        spans: [
          {
            span_type: "check",
            display: "Will DC 28",
            statistic: "will",
            difficulty_class: 28,
          },
        ],
      },
      { block_type: "divider" },
    ],
    content_hash: "dream-bargain",
    visibility: "public",
    provenance: {
      source_record_key: "concept:f1-record",
      relative_source_path: "fixture.json",
      field_family: "fixture.activity",
    },
  };
}

function passiveActivityContent(
  text = "The haunting follows the sleeper.",
): CreatureSurfaceContentView {
  return {
    content_key: "dream-haunting-content",
    role: "embedded_capability",
    authored_order: 0,
    label: "Dream Haunting",
    blocks: [
      {
        block_type: "paragraph",
        spans: [{ span_type: "text", text }],
      },
    ],
    content_hash: "dream-haunting",
    visibility: "public",
    provenance: {
      source_record_key: "concept:f1-record",
      relative_source_path: "fixture.json",
      field_family: "fixture.activity",
    },
  };
}
