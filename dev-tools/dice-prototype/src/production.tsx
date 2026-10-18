import React, { useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { Button, ConfigProvider } from "antd";
import { DieFace } from "../../../web/atlas-ui/src/shared/dice/DieFace";
import { antDesignTheme, atlasCssVariables } from "../../../web/atlas-ui/src/shared/theme/atlasTheme";

/** Fixed outcomes exercise real production drawings, motion, maximum faces and keep state. */
function ProductionDiceComparison() {
  const [pending, setPending] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  useEffect(() => () => clearTimeout(timer.current), []);
  return <ConfigProvider theme={antDesignTheme("light")}><main style={{...atlasCssVariables("light"),fontFamily:"system-ui",padding:24} as React.CSSProperties}>
    <h1>Production dice presentation</h1>
    <p>Fixed maximum-face outcomes; replay changes only animation, never results.</p>
    <Button disabled={pending} onClick={() => {setPending(true);timer.current=setTimeout(()=>setPending(false),650);}}>Replay</Button>
    <div style={{display:"grid",gridTemplateColumns:"repeat(auto-fit,minmax(320px,1fr))",gap:24,marginTop:24}}>
      {(["simple","shaded"] as const).flatMap(drawing => (["spin","tumble","rock"] as const).map(motion => <section key={`${drawing}-${motion}`}>
        <h2>{drawing} · {motion}</h2>
        <div style={{display:"flex",flexWrap:"wrap",gap:8}}>{[4,6,8,10,12,20,78,100,1000].map(sides=><DieFace key={sides} sides={sides} value={sides} pending={pending} drawing={drawing} motion={motion}/>)}</div>
        <p>Fortune: <DieFace sides={20} value={3} kept={pending} pending={pending} drawing={drawing} motion={motion}/><DieFace sides={20} value={20} pending={pending} drawing={drawing} motion={motion}/></p>
      </section>))}
    </div>
  </main></ConfigProvider>;
}
createRoot(document.getElementById("root")!).render(<ProductionDiceComparison/>);
