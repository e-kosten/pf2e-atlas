export type DiceMotion = "spin" | "tumble" | "rock" | "none";

const frames: Record<Exclude<DiceMotion, "none">, Keyframe[]> = {
  spin: [
    { transform: "rotate(0deg)" },
    { transform: "rotate(300deg)", offset: 0.65 },
    { transform: "rotate(360deg)" },
  ],
  tumble: [
    { transform: "none" },
    { transform: "translateY(-4px) rotate(-12deg)", offset: 0.25 },
    { transform: "translateY(-2px) rotate(14deg)", offset: 0.6 },
    { transform: "none" },
  ],
  rock: [
    { transform: "none" },
    { transform: "translateY(-5px) rotate(-18deg) scale(.96)", offset: 0.25 },
    { transform: "translateY(1px) rotate(8deg) scale(1.03)", offset: 0.65 },
    { transform: "none" },
  ],
};

/** Animation has no outcome or request authority. Every strategy settles at identity. */
export function startDiceMotion(element: HTMLElement, motion: DiceMotion) {
  let animation =
    motion === "none" || !element.animate
      ? undefined
      : element.animate(frames[motion], {
          duration: 650,
          easing: "cubic-bezier(.25,.1,.25,1)",
        });
  return {
    settle() {
      if (animation?.playState !== "running") return;
      const transform = getComputedStyle(element).transform;
      animation.cancel();
      animation = element.animate([{ transform }, { transform: "none" }], {
        duration: 160,
        easing: "ease-out",
      });
    },
    cancel() {
      animation?.cancel();
    },
  };
}
