<script setup>
defineProps({
  size: { type: Number, default: 28 },
});
</script>

<template>
  <!-- The brand mark: a paused ring with a gap, around a solid centre. -->
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 64 64"
    fill="none"
    aria-hidden="true"
  >
    <circle
      cx="32"
      cy="32"
      r="22"
      stroke="currentColor"
      stroke-width="5"
      stroke-linecap="round"
      stroke-dasharray="26 8.55"
      transform="rotate(-17 32 32)"
    />
    <circle
      cx="32"
      cy="32"
      r="5.5"
      fill="currentColor"
    />
  </svg>
</template>
