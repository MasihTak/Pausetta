<script setup>
defineProps({
  category: { type: String, required: true },
  size: { type: Number, default: 16 },
});
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <template v-if="category === 'eye'">
      <path d="M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7-10-7-10-7z" />
      <circle
        cx="12"
        cy="12"
        r="3"
      />
    </template>
    <template v-else-if="category === 'posture'">
      <circle
        cx="12"
        cy="4.5"
        r="2"
      />
      <path d="M12 6.5v3.5c0 2-2.2 2.5-2.2 4.5s2.2 2.2 2.2 4v2.5" />
    </template>
    <template v-else-if="category === 'movement'">
      <path d="M9 3.5a1.6 1.6 0 1 0 0 3.2 1.6 1.6 0 0 0 0-3.2z" />
      <path d="M6.8 21l1-6-2-1.5.6-4.3 2.9-1.4 2.2 2.4 2.8.6" />
      <path d="M17 8.2a1.4 1.4 0 1 0 0 2.8 1.4 1.4 0 0 0 0-2.8z" />
      <path d="M14.6 21l1-5.4-2.2-2" />
    </template>
    <template v-else-if="category === 'hydration'">
      <path d="M12 3.2s6.4 7.3 6.4 12A6.4 6.4 0 1 1 5.6 15.2c0-4.7 6.4-12 6.4-12z" />
    </template>
  </svg>
</template>
