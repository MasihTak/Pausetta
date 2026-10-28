<script setup>
// The frame around one hand-drawn vignette per onboarding slide. Every scene inherits its
// hue from the parent's `color`, so a slide only has to set --slide and the whole drawing
// follows. Keep tinted fills to small accents: large soft slabs turn to mud at this size.
// All five scenes are mounted at once in the carousel, so their clip-path ids are unique.
defineProps({
  scene: { type: Object, required: true },
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

    <component :is="scene" />
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

.scene :deep(.scene__soft) {
  fill: color-mix(in srgb, currentColor 18%, transparent);
}

.scene :deep(.scene__stroke) {
  fill: none;
  stroke: currentColor;
  stroke-width: 2.4;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.scene :deep(.scene__ink) {
  fill: currentColor;
}
</style>
