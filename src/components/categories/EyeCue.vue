<template>
  <div class="cue-ring">
    <svg
      width="26"
      height="26"
      viewBox="0 0 30 30"
      aria-hidden="true"
    >
      <circle
        cx="15"
        cy="15"
        r="13"
        class="cue-ring__track"
      />
      <circle
        cx="15"
        cy="15"
        r="13"
        class="cue-ring__progress"
      />
    </svg>
    <span class="cue-ring__label">20s</span>
  </div>
</template>

<style scoped lang="scss">
.cue-ring {
  position: relative;
  flex-shrink: 0;
  width: 26px;
  height: 26px;

  svg {
    transform: rotate(-90deg);
  }
}

.cue-ring__track,
.cue-ring__progress {
  fill: none;
  stroke-width: 3;
}

.cue-ring__track {
  stroke: color-mix(in srgb, var(--cat) 24%, var(--card));
}

.cue-ring__progress {
  stroke: var(--cat-ink);
  stroke-linecap: round;
  stroke-dasharray: 82;
  animation: cue-ring 20s linear forwards;
}

.cue-ring__label {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 8.5px;
  font-weight: 700;
  color: var(--cat-ink);
}

@keyframes cue-ring {
  from {
    stroke-dashoffset: 0;
  }
  to {
    stroke-dashoffset: 82;
  }
}
</style>
