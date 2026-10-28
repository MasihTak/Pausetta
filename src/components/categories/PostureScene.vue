<template>
  <!-- Posture: one upright figure at the desk, with an alignment guide through it. -->
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

<style scoped lang="scss">
// The dashed plumb line through the figure is the whole point of the posture scene.
.scene__guide {
  fill: none;
  stroke: color-mix(in srgb, currentColor 40%, transparent);
  stroke-width: 2;
  stroke-linecap: round;
  stroke-dasharray: 3 7;
  animation: guide-fade 4s ease-in-out infinite;
}

.scene__settle {
  animation: settle-up 5s ease-in-out infinite;
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
