<template>
  <!-- Welcome: a mug on the desk, with the brand mark as a low sun behind it. -->
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

<style scoped lang="scss">
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
</style>
