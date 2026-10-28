<script setup>
import { computed, onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import AppMark from "./AppMark.vue";
import CategoryIcon from "./CategoryIcon.vue";
import ToastCue from "./ToastCue.vue";
import UiIcon from "./UiIcon.vue";
import { CATEGORY_BY_KEY, EYE_COUNTDOWN_SECONDS, SNOOZE_MINUTES } from "../constants/categories.js";
import { useSettingsStore } from "../stores/settings.js";

const store = useSettingsStore();
// A toast shows one reminder for its whole life: Rust opens a new window for the next one.
const categoryKey = window.__PAUSETTA_REMINDER_CATEGORY__;
// Set by Rust in reminder_window.rs, from the same platform check that positions the window.
const isTopAnchored = window.__PAUSETTA_REMINDER_ANCHOR__ === "top";

const AUTO_DISMISS_MS = 14_000;
// Outlasts the eye cue's countdown ring.
const EYE_AUTO_DISMISS_MS = (EYE_COUNTDOWN_SECONDS + 2) * 1000;

const card = ref(null);

const category = CATEGORY_BY_KEY[categoryKey];
const intervalMinutes = computed(
  () => store.settings?.categories[categoryKey].intervalMinutes,
);
const autoDismissMs = categoryKey === "eye" ? EYE_AUTO_DISMISS_MS : AUTO_DISMISS_MS;

let dismissTimer;
let dismissDeadline;
// Only meaningful between a pause (mouseenter) and the resume (mouseleave) that reads it.
let remainingMs;

function startDismissTimer(durationMs) {
  clearTimeout(dismissTimer);
  dismissDeadline = Date.now() + durationMs;
  dismissTimer = setTimeout(dismiss, durationMs);
}

// The progress bar pauses on the same hover, in CSS.
function pauseDismissTimer() {
  clearTimeout(dismissTimer);
  remainingMs = Math.max(0, dismissDeadline - Date.now());
}

function resumeDismissTimer() {
  startDismissTimer(remainingMs);
}

// The window is larger than the card to fit its glow. Rust makes that invisible margin
// click-through, so it needs to know where the card is.
function reportCardBounds() {
  const { left, top, right, bottom } = card.value.getBoundingClientRect();
  invoke("set_toast_card_bounds", { bounds: { left, top, right, bottom } }).catch((error) =>
    console.error("[pausetta] could not report the toast's bounds", error),
  );
}

onMounted(() => {
  store.load().catch((error) => console.error("[pausetta] could not load settings", error));
  // An unattended reminder closes itself rather than waiting on the desk all afternoon.
  startDismissTimer(autoDismissMs);
  // Again once the entrance animation ends: it moves the card, and Rust keeps the last report.
  reportCardBounds();
});

onUnmounted(() => {
  clearTimeout(dismissTimer);
});

function dismiss() {
  invoke("close_reminder").catch((error) => console.error("[pausetta] could not close reminder", error));
}

// Rust owns the schedule, so snoozing is a request to move this category's next trigger.
function snooze() {
  invoke("snooze_reminder", { category: categoryKey }).catch((error) =>
    console.error("[pausetta] could not snooze reminder", error),
  );
}
</script>

<template>
  <div
    class="toast-stage"
    :class="{ 'is-top': isTopAnchored }"
  >
    <article
      ref="card"
      class="toast"
      :style="{ '--cat': `var(--cat-${categoryKey})` }"
      aria-live="polite"
      @animationend.self="reportCardBounds"
      @mouseenter="pauseDismissTimer"
      @mouseleave="resumeDismissTimer"
    >
      <div class="toast__glow" />

      <div class="toast__content">
        <header class="toast__header">
          <span class="toast__mark">
            <AppMark :size="11" />
          </span>
          <span class="toast__app-name">Pausetta</span>
          <span
            v-if="intervalMinutes"
            class="toast__chip"
          >Every {{ intervalMinutes }} min</span>
          <span class="toast__spacer" />
          <button
            type="button"
            class="toast__close"
            aria-label="Close reminder"
            @click="dismiss"
          >
            <UiIcon
              name="close"
              :size="9"
              :stroke-width="2.4"
            />
          </button>
        </header>

        <div class="toast__main">
          <div class="toast__tile">
            <div class="toast__tile-halo" />
            <CategoryIcon
              :category="categoryKey"
              :size="23"
              class="toast__tile-icon"
            />
          </div>

          <div class="toast__text">
            <h1 class="toast__title">
              {{ category.label }}
            </h1>
            <p class="toast__body">
              {{ category.toast.body }}
            </p>

            <ToastCue
              :category="categoryKey"
              :text="category.toast.cue"
            />

            <div class="toast__actions">
              <button
                type="button"
                class="toast__primary"
                @click="dismiss"
              >
                {{ category.toast.action }}
              </button>
              <button
                type="button"
                class="toast__ghost"
                @click="snooze"
              >
                Snooze {{ SNOOZE_MINUTES }} min
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="toast__progress">
        <div
          class="toast__progress-bar"
          :style="{ animationDuration: `${autoDismissMs}ms` }"
        />
      </div>
    </article>
  </div>
</template>

<style scoped lang="scss">
.toast-stage {
  display: flex;
  align-items: flex-end;
  justify-content: flex-end;
  height: 100vh;
  padding: 24px 28px 36px;

  &.is-top {
    align-items: flex-start;
    padding: 12px 28px 36px;
  }
}

.toast {
  --cat-ink: color-mix(in srgb, var(--cat) 86%, #1a0f0c);

  position: relative;
  width: 352px;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--cat) 26%, var(--win-border));
  border-radius: 10px;
  background: var(--card);
  box-shadow: var(--shadow), 0 18px 44px -12px color-mix(in srgb, var(--cat) 42%, transparent);
  animation: toast-in 260ms cubic-bezier(0.2, 0.9, 0.25, 1);

  .is-top & {
    border-radius: 18px;
    animation-name: toast-in-from-top;
  }
}

.toast__glow {
  position: absolute;
  top: -58px;
  right: -40px;
  width: 180px;
  height: 180px;
  border-radius: 50%;
  pointer-events: none;
  background: radial-gradient(circle, color-mix(in srgb, var(--cat) 30%, transparent), transparent 70%);
}

.toast__content {
  position: relative;
  padding: 14px 15px 13px;
}

.toast__header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.toast__mark {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 17px;
  height: 17px;
  border-radius: 5px;
  background: color-mix(in srgb, var(--accent) 20%, var(--card));
  color: var(--accent);
}

.toast__app-name {
  font-size: 10.5px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--text3);
}

.toast__chip {
  padding: 2.5px 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--cat) 15%, var(--card));
  color: var(--cat-ink);
  font-size: 10.5px;
  font-weight: 600;
  letter-spacing: 0.01em;
}

.toast__spacer {
  flex: 1;
}

.toast__close {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text3);
  cursor: pointer;

  &:hover {
    background: var(--border);
    color: var(--text);
  }
}

.toast__main {
  display: flex;
  align-items: flex-start;
  gap: 13px;
}

.toast__tile {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 44px;
  height: 44px;
  overflow: hidden;
  border-radius: 14px;
  background: linear-gradient(
    150deg,
    color-mix(in srgb, var(--cat) 86%, #ffffff),
    color-mix(in srgb, var(--cat) 92%, #2a1410)
  );
  box-shadow:
    0 6px 16px -4px color-mix(in srgb, var(--cat) 55%, transparent),
    inset 0 1px 0 rgba(255, 255, 255, 0.28);
  color: #ffffff;
}

.toast__tile-halo {
  position: absolute;
  inset: 0;
  pointer-events: none;
  background: radial-gradient(circle at 30% 22%, rgba(255, 255, 255, 0.34), transparent 62%);
}

.toast__tile-icon {
  position: relative;
}

.toast__text {
  flex: 1;
  min-width: 0;
}

.toast__title {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
  letter-spacing: -0.01em;
  color: var(--text);
}

.toast__body {
  margin: 3px 0 0;
  font-size: 12.5px;
  line-height: 1.5;
  color: var(--text2);
  text-wrap: pretty;
}

.toast__actions {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-top: 12px;
}

.toast__primary,
.toast__ghost {
  padding: 7px 13px;
  border-radius: 9px;
  font-size: 12.5px;
  cursor: pointer;
}

.toast__primary {
  border: none;
  background: color-mix(in srgb, var(--cat) 94%, #2a1410);
  box-shadow: 0 4px 12px -4px color-mix(in srgb, var(--cat) 60%, transparent);
  color: #ffffff;
  font-weight: 600;
}

.toast__ghost {
  padding-inline: 12px;
  border: 1px solid var(--border-strong);
  background: transparent;
  color: var(--text2);
  font-weight: 500;
}

.toast__progress {
  width: 100%;
  height: 3px;
  background: color-mix(in srgb, var(--cat) 16%, transparent);
}

.toast__progress-bar {
  width: 100%;
  height: 100%;
  background: var(--cat);
  // The duration is bound inline to the same value as the dismiss timer.
  animation: progress-sweep linear reverse forwards;

  .toast:hover & {
    animation-play-state: paused;
  }
}

@media (prefers-color-scheme: dark) {
  .toast {
    --cat-ink: var(--cat);
  }

  .toast__glow {
    background: radial-gradient(circle, color-mix(in srgb, var(--cat) 26%, transparent), transparent 70%);
  }

  .toast__chip {
    background: color-mix(in srgb, var(--cat) 22%, var(--card));
  }

  .toast__tile {
    background: linear-gradient(
      150deg,
      color-mix(in srgb, var(--cat) 84%, #15110e),
      color-mix(in srgb, var(--cat) 52%, #15110e)
    );
  }

  .toast__primary {
    background: var(--cat);
    color: #15110e;
  }
}

@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateY(14px) scale(0.97);
  }
}

@keyframes toast-in-from-top {
  from {
    opacity: 0;
    transform: translateY(-14px) scale(0.97);
  }
}

@keyframes progress-sweep {
  from {
    transform: translateX(-100%);
  }
  to {
    transform: translateX(0);
  }
}
</style>
