<template>
  <div
    class="cue-bars"
    aria-hidden="true"
  >
    <span
      v-for="barIndex in 4"
      :key="barIndex"
      :style="{ animationDelay: `${(barIndex - 1) * 150}ms` }"
    />
  </div>
</template>

<style scoped lang="scss">
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

@keyframes cue-bar {
  0%,
  100% {
    transform: scaleY(0.3);
  }
  50% {
    transform: scaleY(1);
  }
}
</style>
