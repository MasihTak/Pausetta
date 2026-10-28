<script setup>
import { EYE_COUNTDOWN_SECONDS } from "../../constants/eye.js";
</script>

<template>
  <div
    class="cue-ring"
    :style="{ '--eye-countdown': `${EYE_COUNTDOWN_SECONDS}s` }"
  >
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
        pathLength="1"
        class="cue-ring__progress"
      />
    </svg>
    <span class="cue-ring__label">{{ EYE_COUNTDOWN_SECONDS }}s</span>
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
  // pathLength="1" on the circle makes the dash length independent of the radius.
  stroke-dasharray: 1;
  animation: cue-ring var(--eye-countdown) linear forwards;
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
    stroke-dashoffset: 1;
  }
}
</style>
