<template>
  <!-- Movement: up and away from the chair, mid-stride. -->
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

<style scoped lang="scss">
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
</style>
