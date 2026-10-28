<script setup>
// Small interface glyphs; the per-category glyphs live in CategoryIcon.vue.
defineProps({
  name: { type: String, required: true },
  size: { type: Number, default: 13 },
  strokeWidth: { type: Number, default: 2 },
});
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    :stroke-width="strokeWidth"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <template v-if="name === 'info'">
      <circle
        cx="12"
        cy="12"
        r="9"
      />
      <line
        x1="12"
        y1="11"
        x2="12"
        y2="16.5"
      />
      <circle
        cx="12"
        cy="7.7"
        r="0.9"
        fill="currentColor"
        stroke="none"
      />
    </template>
    <polyline
      v-else-if="name === 'chevron-left'"
      points="15 18 9 12 15 6"
    />
    <polyline
      v-else-if="name === 'chevron-down'"
      points="6 9 12 15 18 9"
    />
    <template v-else-if="name === 'pause'">
      <line
        x1="8"
        y1="5"
        x2="8"
        y2="19"
      />
      <line
        x1="16"
        y1="5"
        x2="16"
        y2="19"
      />
    </template>
    <template v-else-if="name === 'close'">
      <line
        x1="3"
        y1="3"
        x2="21"
        y2="21"
      />
      <line
        x1="21"
        y1="3"
        x2="3"
        y2="21"
      />
    </template>
  </svg>
</template>
