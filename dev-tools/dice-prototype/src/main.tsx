import React, { useState, useEffect } from "react";
import { createRoot } from "react-dom/client";
import {
  ConfigProvider,
  App,
  Button,
  InputNumber,
  Select,
  Segmented,
  Popover,
  Switch,
  Collapse,
} from "antd";
import {
  SettingOutlined,
  ReloadOutlined,
  MoonOutlined,
  SunOutlined,
} from "@ant-design/icons";
import {
  antDesignTheme,
  atlasCssVariables,
} from "../../../web/atlas-ui/src/shared/theme/atlasTheme";
import { actors, terrain, formula } from "./fixtures";
import {
  RollResult,
  useRoll,
  type Result,
  type Receipt,
  type Group,
  type Settings,
} from "./rolling";
import "./style.css";
import { AnimationComparison } from "./comparison";

function Prototype() {
  const [dark, setDark] = useState(
    matchMedia("(prefers-color-scheme: dark)").matches,
  );
  const [reduced, setReduced] = useState(
    matchMedia("(prefers-reduced-motion: reduce)").matches,
  );
  const [animate, setAnimate] = useState(true);
  const [delay, setDelay] = useState(0);
  const [fail, setFail] = useState(false);
  const [tab, setTab] = useState("Animation comparison");
  const [history, setHistory] = useState<Receipt[]>([]);
  useEffect(() => {
    const media = matchMedia("(prefers-reduced-motion: reduce)");
    const update = () => setReduced(media.matches);
    media.addEventListener("change", update);
    return () => media.removeEventListener("change", update);
  }, []);
  const settings = { animate, delay, fail, reduced };
  const remember = (r: Receipt) => setHistory((h) => [r, ...h].slice(0, 12));
  const scheme = dark ? "dark" : "light";
  return (
    <ConfigProvider theme={antDesignTheme(scheme)}>
      <App>
        <div className="prototype" style={atlasCssVariables(scheme)}>
          <header className="app-header">
            <strong>Atlas</strong>
            <span className="muted">Dice preview</span>
            <div className="header-tools">
              <Button
                type="text"
                icon={dark ? <SunOutlined /> : <MoonOutlined />}
                aria-label="Toggle appearance"
                onClick={() => setDark(!dark)}
              />
              <Popover
                trigger="click"
                placement="bottomRight"
                title="Preview settings"
                content={
                  <div className="settings">
                    <label>
                      Animate rolls{" "}
                      <Switch checked={animate} onChange={setAnimate} />
                    </label>
                    <label>
                      Response{" "}
                      <Select
                        value={delay}
                        onChange={setDelay}
                        options={[
                          { value: 0, label: "Normal" },
                          { value: 2500, label: "Slow (2.5 seconds)" },
                        ]}
                      />
                    </label>
                    <label>
                      Simulate failure{" "}
                      <Switch checked={fail} onChange={setFail} />
                    </label>
                    <small>
                      Reduced motion follows your device. Retry recovers the
                      same request.
                    </small>
                  </div>
                }
              >
                <Button
                  icon={<SettingOutlined />}
                  aria-label="Preview settings"
                />
              </Popover>
            </div>
          </header>
          <main>
            <div className="page-heading">
              <div>
                <h1>Roll at the table.</h1>
                <p className="muted">
                  Try the same dice in a few different workflows.
                </p>
              </div>
              <Segmented
                aria-label="Roll presentation"
                options={[
                  { label: "Animated", value: true },
                  { label: "Immediate", value: false },
                ]}
                value={animate}
                onChange={setAnimate}
              />
            </div>
            <Segmented
              block
              className="workflow-tabs"
              options={["Initiative", "Attacks & dice", "Roll table", "Animation comparison"]}
              value={tab}
              onChange={setTab}
            />
            <div hidden={tab !== "Initiative"}>
              <Initiative settings={settings} remember={remember} />
            </div>
            <div hidden={tab !== "Attacks & dice"}>
              <DiceLab settings={settings} remember={remember} />
            </div>
            <div hidden={tab !== "Roll table"}>
              <RollTable settings={settings} remember={remember} />
            </div>
            {tab === "Animation comparison" && <AnimationComparison reduced={reduced || !animate} />}
            <Collapse
              className="history"
              ghost
              items={[
                {
                  key: "rolls",
                  label: `Recent rolls${history.length ? ` (${history.length})` : ""}`,
                  children: history.length ? (
                    history.map((r) => (
                      <div key={r.id} className="history-entry">
                        {r.groups.map((g, i) => (
                          <div key={i}>
                            <strong>{g.label}</strong>
                            <span>{g.breakdown}</span>
                            <b>{g.total}</b>
                          </div>
                        ))}
                      </div>
                    ))
                  ) : (
                    <p className="muted">Your rolls will appear here.</p>
                  ),
                },
              ]}
            />
            <footer>
              Prototype only · example creatures and table · no saved encounter
              changes
            </footer>
          </main>
        </div>
      </App>
    </ConfigProvider>
  );
}

function Initiative({
  settings,
  remember,
}: {
  settings: Settings;
  remember: (r: Receipt) => void;
}) {
  const [choices, setChoices] = useState(
    actors.map((a) => ({
      id: a.id,
      skill: "perception",
      modifier: a.perception,
    })),
  );
  const [results, setResults] = useState<Record<string, Result>>({});
  const [activeIds, setActiveIds] = useState<string[]>([]);
  const operation = useRoll(settings, (r) => {
    remember(r);
    setResults((old) => ({
      ...old,
      ...Object.fromEntries(r.groups.map((g, i) => [activeIds[i], g])),
    }));
  });
  const run = (ids: string[]) => {
    setActiveIds(ids);
    void operation.roll(
      ids.map((id) => {
        const a = actors.find((a) => a.id === id)!;
        const c = choices.find((c) => c.id === id)!;
        return {
          label: `${a.name} · ${c.skill === "custom" ? "Custom" : c.skill === "stealth" ? "Stealth" : "Perception"}`,
          formula: formula(c.modifier),
        };
      }),
    );
  };
  const missing = actors.filter((a) => !results[a.id]).map((a) => a.id);
  return (
    <section>
      <div className="section-heading">
        <div>
          <h2>Encounter initiative</h2>
          <p className="muted">
            Each creature rolls separately. Existing results stay until you
            reroll them.
          </p>
        </div>
        <Button
          type="primary"
          disabled={operation.pending || !missing.length}
          onClick={() => run(missing)}
        >
          Roll unrolled ({missing.length})
        </Button>
      </div>
      <div className="roster">
        {actors.map((a) => {
          const c = choices.find((c) => c.id === a.id)!;
          const busy = operation.pending && activeIds.includes(a.id);
          return (
            <div className="initiative-row" key={a.id}>
              <div className="actor-name">
                <strong>{a.name}</strong>
                <span className="muted">
                  {c.skill === "stealth"
                    ? a.stealthNote
                    : c.skill === "custom"
                      ? "Enter a modifier"
                      : "Perception"}
                </span>
              </div>
              <div className="initiative-controls">
                <Select
                  aria-label={`${a.name} initiative skill`}
                  disabled={operation.pending}
                  value={c.skill}
                  onChange={(skill) =>
                    setChoices((cs) =>
                      cs.map((x) =>
                        x.id === a.id
                          ? {
                              ...x,
                              skill,
                              modifier:
                                skill === "stealth"
                                  ? a.stealth
                                  : skill === "perception"
                                    ? a.perception
                                    : x.modifier,
                            }
                          : x,
                      ),
                    )
                  }
                  options={[
                    { value: "perception", label: "Perception" },
                    { value: "stealth", label: "Stealth" },
                    { value: "custom", label: "Other skill" },
                  ]}
                />
                <label className="modifier-control"><span>Modifier</span><InputNumber
                  aria-label={`${a.name} modifier`}
                  min={-100}
                  max={100}
                  precision={0}
                  disabled={operation.pending}
                  value={c.modifier}
                  onChange={(v) =>
                    setChoices((cs) =>
                      cs.map((x) =>
                        x.id === a.id ? { ...x, modifier: v ?? 0 } : x,
                      ),
                    )
                  }
                /></label>
                <Button
                  disabled={operation.pending}
                  aria-label={`Roll ${a.name}`}
                  onClick={() => run([a.id])}
                  icon={results[a.id] ? <ReloadOutlined /> : undefined}
                >
                  {results[a.id] ? "Reroll" : "Roll"}
                </Button>
              </div>
              <RollResult
                result={results[a.id]}
                pending={busy}
                animate={settings.animate && !settings.reduced}
                slow={busy && operation.slow}
              />
            </div>
          );
        })}
      </div>
      {operation.error && (
        <div className="local-error" role="alert">
          {operation.error}{" "}
          <Button size="small" onClick={() => operation.roll([], true)}>
            Retry
          </Button>
        </div>
      )}
      {Object.keys(results).length > 0 && (
        <div className="initiative-order">
          <strong>Initiative order</strong>
          {actors
            .filter((a) => results[a.id])
            .sort((a, b) => results[b.id].total - results[a.id].total)
            .map((a) => (
              <span key={a.id}>
                <b>{results[a.id].total}</b> {a.name}
              </span>
            ))}
        </div>
      )}
    </section>
  );
}

const examples: {
  name: string;
  note: string;
  groups: Group[];
  sides: number[][];
}[] = [
  {
    name: "Crossbow",
    note: "Kobold Scout · attack check",
    groups: [{ label: "Crossbow attack", formula: "1d20+9" }],
    sides: [[20]],
  },
  {
    name: "Damage",
    note: "Example weapon · separate damage types",
    groups: [
      { label: "Slashing", formula: "2d6+4" },
      { label: "Fire", formula: "1d6" },
    ],
    sides: [[6, 6], [6]],
  },
  {
    name: "Mixed dice",
    note: "One component, different die sizes",
    groups: [{ label: "Mixed dice", formula: "1d8+1d6+4" }],
    sides: [[8, 6]],
  },
  {
    name: "Keep higher",
    note: "Roll twice · keep one d20",
    groups: [{ label: "Keep higher", formula: "2d20K1+8" }],
    sides: [[20, 20]],
  },
  {
    name: "Keep lower",
    note: "Roll twice · keep one d20",
    groups: [{ label: "Keep lower", formula: "2d20k1+8" }],
    sides: [[20, 20]],
  },
  {
    name: "Eight dice",
    note: "Larger pool · 8d6",
    groups: [{ label: "Fire", formula: "8d6" }],
    sides: [Array(8).fill(6)],
  },
];
function DiceLab({
  settings,
  remember,
}: {
  settings: Settings;
  remember: (r: Receipt) => void;
}) {
  return (
    <section>
      <div className="section-heading">
        <div>
          <h2>Attacks & damage</h2>
          <p className="muted">
            Attack checks and damage are separate. No target or HP changes.
          </p>
        </div>
      </div>
      <div className="lab-grid">
        {examples.map((example) => (
          <DiceExample
            key={example.name}
            example={example}
            settings={settings}
            remember={remember}
          />
        ))}
      </div>
    </section>
  );
}
function DiceExample({
  example,
  settings,
  remember,
}: {
  example: (typeof examples)[number];
  settings: Settings;
  remember: (r: Receipt) => void;
}) {
  const [result, setResult] = useState<Receipt>();
  const op = useRoll(settings, (r) => {
    setResult(r);
    remember(r);
  });
  return (
    <article className="dice-example">
      <div className="example-heading">
        <div>
          <h3>{example.name}</h3>
          <span className="muted">{example.note}</span>
        </div>
        <Button disabled={op.pending} onClick={() => op.roll(example.groups)}>
          Roll
        </Button>
      </div>
      {example.groups.map((g, i) => (
        <div className="component" key={g.label}>
          <div className="component-label">
            <span>{example.groups.length > 1 ? g.label : ""}</span>
            <code>
              {g.formula
                .replace("K1", " keep higher")
                .replace("k1", " keep lower")}
            </code>
          </div>
          <RollResult
            result={result?.groups[i]}
            pending={op.pending}
            sides={example.sides[i]}
            animate={settings.animate && !settings.reduced}
            slow={op.slow}
          />
        </div>
      ))}
      {result && example.groups.length > 1 && !op.pending && (
        <div className="damage-total">
          Total <strong>{result.total}</strong>
          <span className="muted">
            {result.groups
              .map((g) => `${g.total} ${g.label.toLowerCase()}`)
              .join(" + ")}
          </span>
        </div>
      )}
      {op.error && (
        <div role="alert" className="local-error">
          {op.error}{" "}
          <Button size="small" onClick={() => op.roll([], true)}>
            Retry
          </Button>
        </div>
      )}
    </article>
  );
}
function RollTable({
  settings,
  remember,
}: {
  settings: Settings;
  remember: (r: Receipt) => void;
}) {
  const [result, setResult] = useState<Result>();
  const op = useRoll(settings, (r) => {
    remember(r);
    setResult(r.groups[0]);
  });
  const selected = terrain.find(
    (r) => result && r.low <= result.total && r.high >= result.total,
  );
  return (
    <section>
      <div className="section-heading">
        <div>
          <h2>Random terrain type</h2>
          <p className="muted">Foundry table · 1d20 · results may repeat</p>
        </div>
        <Button
          type="primary"
          disabled={op.pending}
          onClick={() =>
            op.roll([{ label: "Random terrain type", formula: "1d20" }])
          }
        >
          Roll table
        </Button>
      </div>
      <div className="table-result">
        <RollResult
          result={result}
          pending={op.pending}
          animate={settings.animate && !settings.reduced}
          slow={op.slow}
        />
        <strong aria-live="polite">{!op.pending && selected?.text}</strong>
      </div>
      {op.error && (
        <div role="alert" className="local-error">
          {op.error}
          <Button size="small" onClick={() => op.roll([], true)}>
            Retry
          </Button>
        </div>
      )}
      <div className="table-rows">
        {terrain.map((r) => (
          <div
            key={r.low}
            className={selected === r && !op.pending ? "matched" : ""}
          >
            <span>{r.low === r.high ? r.low : `${r.low}–${r.high}`}</span>
            <span>{r.text}</span>
          </div>
        ))}
      </div>
    </section>
  );
}
createRoot(document.getElementById("root")!).render(<Prototype />);
