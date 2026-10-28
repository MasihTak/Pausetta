<script setup>
defineProps({
  title: { type: String, required: true },
  description: { type: String, default: "" },
  isDimmed: { type: Boolean, default: false },
});
</script>

<template>
  <div class="settings-row">
    <div class="settings-row__head">
      <span class="settings-row__title">{{ title }}</span>
      <slot name="control" />
    </div>
    <div
      class="settings-row__detail"
      :class="{ 'is-dimmed': isDimmed }"
    >
      <span class="settings-row__description">{{ description }}</span>
      <slot name="detail" />
    </div>
    <slot />
  </div>
</template>
