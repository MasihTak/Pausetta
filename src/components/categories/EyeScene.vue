<template>
  <!-- Eye care: a window onto something far away, with a cloud drifting past. -->
  <clipPath id="scene-pane-eye">
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
  <g clip-path="url(#scene-pane-eye)">
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

<style scoped lang="scss">
.scene__wash {
  fill: color-mix(in srgb, currentColor 8%, transparent);
}

.scene__cloud {
  animation: cloud-drift 18s linear infinite;
}

@keyframes cloud-drift {
  from {
    transform: translateX(-16px);
  }
  to {
    transform: translateX(104px);
  }
}
</style>
