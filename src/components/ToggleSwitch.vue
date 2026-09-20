<script setup>
const isOn = defineModel({ type: Boolean, required: true });

defineProps({
  label: { type: String, required: true },
});
</script>

<template>
  <button
    type="button"
    role="switch"
    class="toggle"
    :class="{ 'is-on': isOn }"
    :aria-checked="isOn"
    :aria-label="label"
    @click="isOn = !isOn"
  >
    <span class="toggle__knob" />
  </button>
</template>

<style scoped lang="scss">
.toggle {
  position: relative;
  flex-shrink: 0;
  width: 42px;
  height: 24px;
  padding: 0;
  border: none;
  border-radius: 999px;
  background: var(--track-off);
  box-shadow:
    inset 0 0 0 1px var(--border-strong),
    inset 0 1px 2px rgba(24, 28, 26, 0.07);
  cursor: pointer;
  transition:
    background-color 240ms cubic-bezier(0.3, 0.85, 0.3, 1),
    box-shadow 240ms ease;

  &.is-on {
    background: var(--switch-on);
    box-shadow:
      inset 0 0 0 1px var(--switch-on-ring),
      inset 0 1px 1px rgba(255, 255, 255, 0.18),
      0 3px 10px -4px var(--switch-glow);
  }
}

.toggle__knob {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--knob);
  box-shadow: 0 1px 3px rgba(24, 28, 26, 0.26);
  transition: left 240ms cubic-bezier(0.3, 0.85, 0.3, 1);

  .is-on & {
    left: 21px;
  }
}
</style>
