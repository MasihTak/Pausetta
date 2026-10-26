<script setup>
import { useSettingsStore } from "../stores/settings.js";

const store = useSettingsStore();
</script>

<template>
  <div
    v-if="store.saveError"
    class="save-error"
    role="alert"
  >
    <span>Couldn't save your change: {{ store.saveError }}</span>
    <button
      type="button"
      class="save-error__dismiss"
      @click="store.dismissSaveError"
    >
      Dismiss
    </button>
  </div>
</template>

<style scoped lang="scss">
// Sticky so it stays in view when the change was made far down the scrolled Settings list.
.save-error {
  position: sticky;
  top: 0;
  z-index: 1;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--border-strong);
  background: var(--card);
  font-size: 12.5px;
  color: var(--text);

  span {
    flex: 1;
  }
}

.save-error__dismiss {
  flex-shrink: 0;
  padding: 4px 6px;
  border: 0;
  background: none;
  font-weight: 600;
  color: var(--accent);
  cursor: pointer;

  &:hover {
    color: var(--accent-hover);
  }
}
</style>
