<template>
  <!-- Hydration: a glass filling, with bubbles rising through it. -->
  <clipPath id="scene-glass-hydration">
    <path d="M80 40h40l-5 66a9 9 0 0 1-9 8H94a9 9 0 0 1-9-8z" />
  </clipPath>
  <g clip-path="url(#scene-glass-hydration)">
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

<style scoped lang="scss">
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
</style>
