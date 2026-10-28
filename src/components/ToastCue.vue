<script setup>
import { EYE_COUNTDOWN_SECONDS } from "../constants/categories.js";

defineProps({
  category: { type: String, required: true },
  text: { type: String, required: true },
});
</script>

<template>
  <div class="toast__cue">
    <div
      v-if="category === 'eye'"
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
    <div
      v-else-if="category === 'posture'"
      class="cue-posture"
      aria-hidden="true"
    >
      <span />
      <span />
      <span />
    </div>
    <div
      v-else-if="category === 'movement'"
      class="cue-bars"
      aria-hidden="true"
    >
      <span
        v-for="barIndex in 4"
        :key="barIndex"
        :style="{ animationDelay: `${(barIndex - 1) * 150}ms` }"
      />
    </div>
    <div
      v-else-if="category === 'hydration'"
      class="cue-glass"
      aria-hidden="true"
    >
      <span />
    </div>
    <span class="toast__cue-text">{{ text }}</span>
  </div>
</template>

<style scoped lang="scss">
.toast__cue {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 11px;
  padding: 9px 11px;
  border: 1px solid color-mix(in srgb, var(--cat) 20%, transparent);
  border-radius: 12px;
  background: color-mix(in srgb, var(--cat) 9%, var(--card));
}

.toast__cue-text {
  flex: 1;
  min-width: 0;
  font-size: 11.5px;
  font-weight: 500;
  line-height: 1.4;
  color: var(--text2);
  text-wrap: pretty;
}

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

.cue-posture {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  flex-shrink: 0;
  width: 26px;

  span {
    height: 3px;
    border-radius: 2px;
    background: var(--cat-ink);
  }
}

.cue-bars {
  display: flex;
  align-items: flex-end;
  gap: 3px;
  flex-shrink: 0;
  width: 26px;
  height: 22px;

  span {
    width: 4px;
    height: 22px;
    border-radius: 2px;
    background: var(--cat-ink);
    transform-origin: bottom;
    animation: cue-bar 900ms ease-in-out infinite;
  }
}

.cue-glass {
  position: relative;
  flex-shrink: 0;
  width: 18px;
  height: 24px;
  margin: 0 4px;
  overflow: hidden;
  border: 2px solid var(--cat-ink);
  border-top: none;
  border-radius: 2px 2px 7px 7px;

  span {
    position: absolute;
    right: 0;
    bottom: 0;
    left: 0;
    height: 80%;
    background: var(--cat-ink);
    opacity: 0.85;
    animation: cue-fill 2.4s ease-in-out infinite alternate;
  }
}

// The three bars narrow toward the bottom, like a slouch straightening.
.cue-posture span {
  &:nth-child(1) {
    width: 16px;
  }

  &:nth-child(2) {
    width: 11px;
    opacity: 0.75;
  }

  &:nth-child(3) {
    width: 20px;
    opacity: 0.5;
  }
}

@media (prefers-color-scheme: dark) {
  .toast__cue {
    border-color: color-mix(in srgb, var(--cat) 26%, transparent);
    background: color-mix(in srgb, var(--cat) 14%, var(--card));
  }
}

@keyframes cue-ring {
  from {
    stroke-dashoffset: 0;
  }
  to {
    stroke-dashoffset: 1;
  }
}

@keyframes cue-bar {
  0%,
  100% {
    transform: scaleY(0.3);
  }
  50% {
    transform: scaleY(1);
  }
}

@keyframes cue-fill {
  0% {
    height: 24%;
  }
  100% {
    height: 82%;
  }
}
</style>
