<script setup>
import { computed, nextTick, onMounted, onUnmounted, ref, useTemplateRef, watch } from "vue";
import UiIcon from "./UiIcon.vue";
import { useSettingsStore } from "../stores/settings.js";

const store = useSettingsStore();
const isMenuOpen = ref(false);
const menuElement = useTemplateRef("menu");
const triggerElement = useTemplateRef("trigger");

// Display-only clock for the "time left" label; scheduling itself lives in Rust.
const now = ref(Date.now());
let clockTimer;

// Without this, a fresh pause is measured against a clock up to 30s old ("1 hr 1 min left").
watch(
  () => store.settings.pause,
  () => (now.value = Date.now()),
);

const activePause = computed(() => {
  const pause = store.settings.pause;
  if (!pause) return null;
  const millisecondsLeft = Date.parse(pause.until) - now.value;
  return millisecondsLeft > 0 ? { kind: pause.kind, millisecondsLeft } : null;
});

const isPaused = computed(() => activePause.value !== null);

function formatTimeLeft(milliseconds) {
  const totalMinutes = Math.ceil(milliseconds / 60_000);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (hours === 0) return `${minutes} min`;
  return minutes === 0 ? `${hours} hr` : `${hours} hr ${minutes} min`;
}

const buttonLabel = computed(() => {
  if (!activePause.value) return "Pause";
  if (activePause.value.kind === "today") return "Paused for today";
  return `Paused · ${formatTimeLeft(activePause.value.millisecondsLeft)} left`;
});

async function openMenu() {
  isMenuOpen.value = true;
  await nextTick();
  menuElement.value?.querySelector("button")?.focus();
}

// The open menu holds focus, so closing it must hand focus back or it falls to the page top.
function closeMenu() {
  isMenuOpen.value = false;
  triggerElement.value?.focus();
}

function toggleMenu() {
  if (isMenuOpen.value) {
    closeMenu();
    return;
  }
  openMenu();
}

function choose(action) {
  closeMenu();
  action();
}

function closeOnEscape(event) {
  if (event.key === "Escape" && isMenuOpen.value) closeMenu();
}

onMounted(() => {
  clockTimer = setInterval(() => (now.value = Date.now()), 30_000);
  window.addEventListener("keydown", closeOnEscape);
});

onUnmounted(() => {
  clearInterval(clockTimer);
  window.removeEventListener("keydown", closeOnEscape);
});
</script>

<template>
  <div class="pause">
    <button
      ref="trigger"
      type="button"
      class="pause__button"
      :class="{ 'is-paused': isPaused }"
      :aria-expanded="isMenuOpen"
      @click="toggleMenu"
    >
      <UiIcon name="pause" />
      <span>{{ buttonLabel }}</span>
      <UiIcon
        name="chevron-down"
        :size="9"
        :stroke-width="2.5"
      />
    </button>

    <template v-if="isMenuOpen">
      <div
        class="pause__backdrop"
        @click="closeMenu"
      />
      <div
        ref="menu"
        class="pause__menu"
      >
        <button
          type="button"
          class="pause__item"
          @click="choose(() => store.pause('oneHour'))"
        >
          Pause for 1 hour
        </button>
        <button
          type="button"
          class="pause__item"
          @click="choose(() => store.pause('today'))"
        >
          Pause for today
        </button>
        <button
          v-if="isPaused"
          type="button"
          class="pause__item is-resume"
          @click="choose(store.resume)"
        >
          Resume
        </button>
      </div>
    </template>
  </div>
</template>

<style scoped lang="scss">
.pause {
  position: relative;
}

.pause__button {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 11px;
  border: 1px solid var(--border-strong);
  border-radius: 7px;
  background: var(--card);
  color: var(--text);
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;

  // Reads as "active", not alarming.
  &.is-paused {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 12%, var(--card));
    color: var(--accent);
  }
}

.pause__backdrop {
  position: fixed;
  inset: 0;
  z-index: 40;
}

.pause__menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  z-index: 50;
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 172px;
  padding: 6px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--card);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.16);
}

.pause__item {
  padding: 8px 10px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text);
  font-size: 12.5px;
  text-align: left;
  cursor: pointer;

  &:hover {
    background: var(--border);
  }

  &.is-resume {
    color: var(--accent);
    font-weight: 600;
  }
}
</style>
