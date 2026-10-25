export function AtlasMark() {
  return (
    <svg
      className="atlas-mark"
      viewBox="0 0 160 160"
      aria-hidden="true"
      focusable="false"
    >
      <circle className="atlas-mark__ring" cx="80" cy="80" r="58" />
      <circle className="atlas-mark__ring-inner" cx="80" cy="80" r="50" />

      <g className="atlas-mark__ticks">
        <path d="M80 31v7M80 122v7M31 80h7M122 80h7" />
        <path d="m45.4 45.4 5 5M109.6 109.6l5 5M114.6 45.4l-5 5M50.4 109.6l-5 5" />
      </g>

      <g className="atlas-mark__secondary-points">
        <path d="M119 41 91.3 80 80 68.7Z" />
        <path d="M119 119 80 91.3 91.3 80Z" />
        <path d="M41 119 68.7 80 80 91.3Z" />
        <path d="M41 41 80 68.7 68.7 80Z" />
      </g>

      <g className="atlas-mark__cardinal-points">
        <path d="M80 6 94.5659 70 80 66 65.4341 70Z" />
        <path d="M154 80 90 93.042 94 80 90 66.958Z" />
        <path d="M80 154 65.4341 90 80 94 94.5659 90Z" />
        <path d="M6 80 70 66.958 66 80 70 93.042Z" />
      </g>
      <g className="atlas-mark__cardinal-shading">
        <path d="M80 6v60L65.4341 70Z" />
        <path d="M154 80H94L90 66.958Z" />
        <path d="M80 154V94L94.5659 90Z" />
        <path d="M6 80H66L70 93.042Z" />
      </g>

      <g transform="translate(59 55.8752) scale(.42)">
        <path
          className="atlas-mark__die-rim"
          d="M50 0 100 28.72 100 86.16 50 114.88 0 86.16 0 28.72Z"
        />
        <path
          className="atlas-mark__die-field"
          d="M50 5.744 95 31.592 95 83.288 50 109.136 5 83.288 5 31.592Z"
        />
        <path
          className="atlas-mark__die-face atlas-mark__die-face--0"
          d="M50 25.490115 77.811529 73.414943 22.188471 73.414943Z"
        />
        <path
          className="atlas-mark__die-face atlas-mark__die-face--1"
          d="M50 25.490115 77.811529 73.414943 95 31.592Z"
        />
        <path
          className="atlas-mark__die-face atlas-mark__die-face--2"
          d="M50 25.490115 5 31.592 50 5.744Z"
        />
        <path
          className="atlas-mark__die-face atlas-mark__die-face--3"
          d="M50 25.490115 5 31.592 22.188471 73.414943Z"
        />
        <path
          className="atlas-mark__die-face atlas-mark__die-face--4"
          d="M50 25.490115 50 5.744 95 31.592Z"
        />
        <path
          className="atlas-mark__die-face atlas-mark__die-face--5"
          d="M77.811529 73.414943 50 109.136 95 83.288Z"
        />
        <path
          className="atlas-mark__die-face atlas-mark__die-face--6"
          d="M77.811529 73.414943 50 109.136 22.188471 73.414943Z"
        />
        <path
          className="atlas-mark__die-face atlas-mark__die-face--7"
          d="M77.811529 73.414943 95 83.288 95 31.592Z"
        />
        <path
          className="atlas-mark__die-face atlas-mark__die-face--8"
          d="M5 31.592 22.188471 73.414943 5 83.288Z"
        />
        <path
          className="atlas-mark__die-face atlas-mark__die-face--9"
          d="M50 109.136 22.188471 73.414943 5 83.288Z"
        />
      </g>
    </svg>
  );
}
