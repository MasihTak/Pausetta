<script setup>
// One hand-drawn vignette per onboarding slide. Every scene inherits its hue from the
// parent's `color`, so a slide only has to set --slide and the whole drawing follows.
// All five scenes are mounted at once in the carousel, so clip-path ids are namespaced.
// Keep tinted fills to small accents: large soft slabs turn to mud at this size.
defineProps({
  category: { type: String, required: true },
});
</script>

<template>
  <svg
    class="scene"
    viewBox="0 0 200 140"
    fill="none"
    aria-hidden="true"
  >
    <ellipse
      class="scene__ground"
      cx="100"
      cy="122"
      rx="58"
      ry="8"
    />

    <!-- Welcome: a mug on the desk, with the brand mark as a low sun behind it. -->
    <template v-if="category === 'welcome'">
      <circle
        class="scene__stroke scene__sun"
        cx="146"
        cy="46"
        r="15"
        stroke-dasharray="18 5.9"
        transform="rotate(-17 146 46)"
      />
      <circle
        class="scene__ink"
        cx="146"
        cy="46"
        r="4"
      />
      <g class="scene__steam">
        <path
          class="scene__wisp scene__wisp--a"
          d="M82 62c-5-6 4-9 0-16"
        />
        <path
          class="scene__wisp scene__wisp--b"
          d="M96 58c-5-7 4-10 0-18"
        />
        <path
          class="scene__wisp scene__wisp--c"
          d="M110 62c-5-6 4-9 0-16"
        />
      </g>
      <path
        class="scene__soft"
        d="M74 76h44v24a13 13 0 0 1-13 13H87a13 13 0 0 1-13-13z"
      />
      <path
        class="scene__stroke"
        d="M74 76h44v24a13 13 0 0 1-13 13H87a13 13 0 0 1-13-13z"
      />
      <path
        class="scene__stroke"
        d="M118 83h6a9 9 0 0 1 0 18h-6"
      />
      <path
        class="scene__stroke"
        d="M62 113h76"
      />
    </template>

    <!-- Eye care: a window onto something far away, with a cloud drifting past. -->
    <template v-else-if="category === 'eye'">
      <clipPath :id="`scene-pane-${category}`">
        <rect
          x="48"
          y="26"
          width="104"
          height="76"
          rx="11"
        />
      </clipPath>
      <rect
        class="scene__wash"
        x="48"
        y="26"
        width="104"
        height="76"
        rx="11"
      />
      <g :clip-path="`url(#scene-pane-${category})`">
        <circle
          class="scene__soft"
          cx="130"
          cy="44"
          r="9"
        />
        <g class="scene__cloud">
          <path
            class="scene__soft"
            d="M56 48c0-5 4-8 8-7 2-6 10-6 12 0 5-1 9 2 9 6s-3 6-7 6H63c-4 0-7-2-7-5z"
          />
        </g>
        <path
          class="scene__soft"
          d="M48 94c16-10 26-2 38-7 13-5 22 3 34 0 10-3 18 4 32 9v8H48z"
        />
        <path
          class="scene__stroke"
          d="M70 94V82"
        />
        <circle
          class="scene__ink"
          cx="70"
          cy="76"
          r="7.5"
        />
      </g>
      <rect
        class="scene__stroke"
        x="48"
        y="26"
        width="104"
        height="76"
        rx="11"
      />
      <path
        class="scene__stroke"
        d="M100 26v76M48 64h104"
      />
    </template>

    <!-- Posture: one upright figure at the desk, with an alignment guide through it. -->
    <template v-else-if="category === 'posture'">
      <path
        class="scene__stroke"
        d="M104 96h64M112 96v24M160 96v24"
      />
      <rect
        class="scene__stroke"
        x="124"
        y="64"
        width="34"
        height="24"
        rx="4"
      />
      <path
        class="scene__stroke"
        d="M141 88v6M132 94h18"
      />
      <path
        class="scene__stroke"
        d="M58 102h36M64 102v18M88 102v18"
      />
      <path
        class="scene__stroke"
        d="M58 102V78"
      />
      <path
        class="scene__guide"
        d="M78 40v64"
      />
      <g class="scene__settle">
        <path
          class="scene__stroke"
          d="M76 100C76 86 77 78 78 68"
        />
        <path
          class="scene__stroke"
          d="M77 80l25 11"
        />
        <circle
          class="scene__ink"
          cx="78"
          cy="60"
          r="8"
        />
      </g>
    </template>

    <!-- Movement: up and away from the chair, mid-stride. -->
    <template v-else-if="category === 'movement'">
      <path
        class="scene__stroke"
        d="M34 104h38M40 104v18M66 104v18M34 104V84"
      />
      <g class="scene__stride">
        <circle
          class="scene__ink"
          cx="112"
          cy="42"
          r="8.5"
        />
        <path
          class="scene__stroke"
          d="M112 52v26"
        />
        <path
          class="scene__stroke"
          d="M112 78l-11 24M112 78l13 22"
        />
        <path
          class="scene__stroke"
          d="M112 60l-16 9M112 60l18 5"
        />
      </g>
      <path
        class="scene__arc scene__arc--a"
        d="M146 46a36 36 0 0 1 0 50"
      />
      <path
        class="scene__arc scene__arc--b"
        d="M158 38a48 48 0 0 1 0 66"
      />
    </template>

    <!-- Hydration: a glass filling, with bubbles rising through it. -->
    <template v-else-if="category === 'hydration'">
      <clipPath :id="`scene-glass-${category}`">
        <path d="M80 40h40l-5 66a9 9 0 0 1-9 8H94a9 9 0 0 1-9-8z" />
      </clipPath>
      <g :clip-path="`url(#scene-glass-${category})`">
        <path
          class="scene__water"
          d="M74 70c9-5 15 4 24 2s17-7 28-3v54H74z"
        />
        <circle
          class="scene__bubble scene__bubble--a"
          cx="94"
          cy="104"
          r="3"
        />
        <circle
          class="scene__bubble scene__bubble--b"
          cx="106"
          cy="108"
          r="2.2"
        />
        <circle
          class="scene__bubble scene__bubble--c"
          cx="100"
          cy="112"
          r="1.6"
        />
      </g>
      <path
        class="scene__stroke"
        d="M80 40h40l-5 66a9 9 0 0 1-9 8H94a9 9 0 0 1-9-8z"
      />
      <path
        class="scene__drop"
        d="M140 52a7 7 0 1 1-14 0c0-4 7-12 7-12s7 8 7 12z"
      />
    </template>
  </svg>
</template>

<style scoped lang="scss">
.scene {
  display: block;
  width: 100%;
  height: auto;
}

.scene__ground {
  fill: color-mix(in srgb, currentColor 10%, transparent);
}

.scene__wash {
  fill: color-mix(in srgb, currentColor 8%, transparent);
}

.scene__soft {
  fill: color-mix(in srgb, currentColor 18%, transparent);
}

.scene__stroke {
  fill: none;
  stroke: currentColor;
  stroke-width: 2.4;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.scene__ink {
  fill: currentColor;
}

.scene__sun {
  animation: sun-turn 28s linear infinite;
  transform-origin: 146px 46px;
}

.scene__wisp {
  fill: none;
  stroke: color-mix(in srgb, currentColor 45%, transparent);
  stroke-width: 2.2;
  stroke-linecap: round;
  animation: wisp-rise 4.5s ease-in-out infinite;

  &--b {
    animation-duration: 5.4s;
    animation-delay: 600ms;
  }

  &--c {
    animation-duration: 5s;
    animation-delay: 1200ms;
  }
}

.scene__water {
  fill: color-mix(in srgb, currentColor 38%, transparent);
  animation: water-swell 5s ease-in-out infinite alternate;
}

.scene__drop {
  fill: color-mix(in srgb, currentColor 55%, transparent);
  animation: drop-fall 4s ease-in-out infinite;
}

.scene__bubble {
  fill: color-mix(in srgb, currentColor 55%, transparent);
  animation: bubble-rise 3.4s ease-in infinite;

  &--b {
    animation-duration: 4.2s;
    animation-delay: 700ms;
  }

  &--c {
    animation-duration: 3.8s;
    animation-delay: 1500ms;
  }
}

.scene__cloud {
  animation: cloud-drift 18s linear infinite;
}

// The dashed plumb line through the figure is the whole point of the posture scene.
.scene__guide {
  fill: none;
  stroke: color-mix(in srgb, currentColor 40%, transparent);
  stroke-width: 2;
  stroke-linecap: round;
  stroke-dasharray: 3 7;
  animation: guide-fade 4s ease-in-out infinite;
}

.scene__arc {
  fill: none;
  stroke: color-mix(in srgb, currentColor 45%, transparent);
  stroke-width: 2.4;
  stroke-linecap: round;
  stroke-dasharray: 10 12;
  animation: arc-pulse 2.6s ease-in-out infinite;

  &--b {
    animation-delay: 300ms;
  }
}

.scene__stride {
  animation: stride-bob 2.2s ease-in-out infinite;
}

.scene__settle {
  animation: settle-up 5s ease-in-out infinite;
}

@keyframes sun-turn {
  to {
    transform: rotate(360deg);
  }
}

@keyframes wisp-rise {
  0%,
  100% {
    opacity: 0.25;
    transform: translateY(2px);
  }
  50% {
    opacity: 0.9;
    transform: translateY(-4px);
  }
}

@keyframes water-swell {
  from {
    transform: translateY(6px);
  }
  to {
    transform: translateY(-2px);
  }
}

@keyframes drop-fall {
  0%,
  60% {
    opacity: 0;
    transform: translateY(-6px);
  }
  75% {
    opacity: 1;
  }
  100% {
    opacity: 0;
    transform: translateY(14px);
  }
}

@keyframes bubble-rise {
  0% {
    opacity: 0;
    transform: translateY(0);
  }
  20% {
    opacity: 1;
  }
  100% {
    opacity: 0;
    transform: translateY(-38px);
  }
}

@keyframes cloud-drift {
  from {
    transform: translateX(-16px);
  }
  to {
    transform: translateX(104px);
  }
}

@keyframes guide-fade {
  0%,
  100% {
    opacity: 0.3;
  }
  50% {
    opacity: 1;
  }
}

@keyframes arc-pulse {
  0%,
  100% {
    opacity: 0.35;
    transform: translateX(0);
  }
  50% {
    opacity: 0.9;
    transform: translateX(4px);
  }
}

@keyframes stride-bob {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-4px);
  }
}

@keyframes settle-up {
  0%,
  100% {
    transform: translateY(1.5px);
  }
  50% {
    transform: translateY(-1.5px);
  }
}
</style>
