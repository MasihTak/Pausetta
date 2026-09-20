<script setup>
const selected = defineModel({ required: true });

defineProps({
  options: { type: Array, required: true },
  label: { type: String, required: true },
  disabled: { type: Boolean, default: false },
  isCompact: { type: Boolean, default: false },
});
</script>

<template>
  <span
    class="select"
    :class="{ 'is-compact': isCompact }"
  >
    <!-- Native select keeps the option list fully native; only the closed state is styled. -->
    <select
      v-model="selected"
      :disabled="disabled"
      :aria-label="label"
    >
      <option
        v-for="option in options"
        :key="option.value"
        :value="option.value"
      >
        {{ option.label }}
      </option>
    </select>
    <svg
      class="select__chevron"
      viewBox="0 0 24 24"
      aria-hidden="true"
    >
      <polyline points="6 9 12 15 18 9" />
    </svg>
  </span>
</template>

<style scoped lang="scss">
.select {
  position: relative;
  display: inline-flex;
  flex-shrink: 0;
}

select {
  appearance: none;
  height: 29px;
  padding: 0 26px 0 12px;
  border: 1px solid var(--field-border);
  border-radius: 999px;
  background: var(--field-bg);
  color: var(--text);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;

  &:disabled {
    cursor: default;
  }

  .is-compact & {
    max-width: 92px;
    padding-right: 24px;
  }
}

.select__chevron {
  position: absolute;
  top: 50%;
  right: 8px;
  width: 9px;
  height: 9px;
  transform: translateY(-50%);
  pointer-events: none;
  fill: none;
  stroke: var(--text3);
  stroke-width: 2.5;
  stroke-linecap: round;
  stroke-linejoin: round;

  .is-compact & {
    right: 7px;
    width: 8px;
    height: 8px;
  }
}
</style>
