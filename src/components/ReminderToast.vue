<script setup>
import { computed, onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cursorPosition, getCurrentWindow } from "@tauri-apps/api/window";
import AppMark from "./AppMark.vue";
import CategoryIcon from "./CategoryIcon.vue";
import { CATEGORY_BY_KEY } from "../constants/categories.js";
import { useSettingsStore } from "../stores/settings.js";

const store = useSettingsStore();
const categoryKey = ref(window.__PAUSETTA_REMINDER_CATEGORY__);
// Bumped when Rust reuses this window for a new reminder, to replay the card's animations.
const showCount = ref(0);
// Set by Rust in reminder_window.rs, from the same platform check that positions the window.
const isTopAnchored = window.__PAUSETTA_REMINDER_ANCHOR__ === "top";

const AUTO_DISMISS_MS = 14_000;
// Outlasts the eye cue's 20s ring.
const EYE_AUTO_DISMISS_MS = 22_000;
const CURSOR_CHECK_MS = 50;

const card = ref(null);

const category = computed(() => CATEGORY_BY_KEY[categoryKey.value]);
const intervalMinutes = computed(
  () => store.settings?.categories[categoryKey.value].intervalMinutes,
);
const autoDismissMs = computed(() =>
  categoryKey.value === "eye" ? EYE_AUTO_DISMISS_MS : AUTO_DISMISS_MS,
);

let stopListening;
let dismissTimer;
let dismissDeadline;
let remainingMs;
let isHovered = false;
let isTrackingCursor = true;

function startDismissTimer(durationMs) {
  clearTimeout(dismissTimer);
  dismissDeadline = Date.now() + durationMs;
  dismissTimer = setTimeout(dismiss, durationMs);
}

// An unattended reminder closes itself rather than waiting on the desk all afternoon.
function restartDismissTimer() {
  remainingMs = autoDismissMs.value;
  if (!isHovered) startDismissTimer(remainingMs);
}

// The progress bar pauses on the same hover, in CSS.
function pauseDismissTimer() {
  isHovered = true;
  clearTimeout(dismissTimer);
  remainingMs = Math.max(0, dismissDeadline - Date.now());
}

function resumeDismissTimer() {
  isHovered = false;
  startDismissTimer(remainingMs);
}

function isCursorOverCard(cursor, windowPosition, scaleFactor) {
  const bounds = card.value?.getBoundingClientRect();
  if (!bounds) return false;
  const x = (cursor.x - windowPosition.x) / scaleFactor;
  const y = (cursor.y - windowPosition.y) / scaleFactor;
  return x >= bounds.left && x < bounds.right && y >= bounds.top && y < bounds.bottom;
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

// The window is larger than the card to fit its glow. That invisible margin must not
// swallow clicks meant for the apps underneath, and a click-through window receives no
// mouse events, so the cursor is polled to switch click-through off over the card.
async function letClicksThroughOutsideCard() {
  const appWindow = getCurrentWindow();
  const [windowPosition, scaleFactor] = await Promise.all([
    appWindow.innerPosition(),
    appWindow.scaleFactor(),
  ]);
  let isIgnoringCursor;
  while (isTrackingCursor) {
    const cursor = await cursorPosition();
    const shouldIgnoreCursor = !isCursorOverCard(cursor, windowPosition, scaleFactor);
    if (shouldIgnoreCursor !== isIgnoringCursor) {
      await appWindow.setIgnoreCursorEvents(shouldIgnoreCursor);
      isIgnoringCursor = shouldIgnoreCursor;
    }
    await sleep(CURSOR_CHECK_MS);
  }
}

onMounted(async () => {
  store.load().catch((error) => console.error("[pausetta] could not load settings", error));
  restartDismissTimer();
  letClicksThroughOutsideCard().catch((error) =>
    console.error("[pausetta] could not make the toast's margin click-through", error),
  );
  stopListening = await listen("reminder-changed", (event) => {
    categoryKey.value = event.payload;
    showCount.value += 1;
    restartDismissTimer();
  });
});

onUnmounted(() => {
  isTrackingCursor = false;
  clearTimeout(dismissTimer);
  stopListening?.();
});

function dismiss() {
  invoke("close_reminder").catch((error) => console.error("[pausetta] could not close reminder", error));
}

// Rust owns the schedule, so snoozing is a request to move this category's next trigger.
function snooze() {
  invoke("snooze_reminder", { category: categoryKey.value }).catch((error) =>
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
      :key="`${categoryKey}-${showCount}`"
      ref="card"
      class="toast"
      :style="{ '--cat': `var(--cat-${categoryKey})` }"
      aria-live="polite"
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
            <svg
              width="9"
              height="9"
              viewBox="0 0 10 10"
              aria-hidden="true"
            >
              <line
                x1="0"
                y1="0"
                x2="10"
                y2="10"
                stroke="currentColor"
                stroke-width="1.4"
              />
              <line
                x1="10"
                y1="0"
                x2="0"
                y2="10"
                stroke="currentColor"
                stroke-width="1.4"
              />
            </svg>
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

            <div class="toast__cue">
              <div
                v-if="categoryKey === 'eye'"
                class="cue-ring"
              >
                <svg
                  width="26"
                  height="26"
                  viewBox="0 0 30 30"
                  aria-hidden="true"
                >
                  <circle
                    cx="15"
                    cy="15"
                    r="13"
                    class="cue-ring__track"
                  />
                  <circle
                    cx="15"
                    cy="15"
                    r="13"
                    class="cue-ring__progress"
                  />
                </svg>
                <span class="cue-ring__label">20s</span>
              </div>
              <div
                v-else-if="categoryKey === 'posture'"
                class="cue-posture"
                aria-hidden="true"
              >
                <span style="width: 16px" />
                <span style="width: 11px; opacity: 0.75" />
                <span style="width: 20px; opacity: 0.5" />
              </div>
              <div
                v-else-if="categoryKey === 'movement'"
                class="cue-bars"
                aria-hidden="true"
              >
                <span
                  v-for="barIndex in 4"
                  :key="barIndex"
                  :style="{ animationDelay: `${(barIndex - 1) * 150}ms` }"
                />
              </div>
              <div
                v-else-if="categoryKey === 'hydration'"
                class="cue-glass"
                aria-hidden="true"
              >
                <span />
              </div>
              <span class="toast__cue-text">{{ category.toast.cue }}</span>
            </div>

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
                Snooze 10 min
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

.toast__cue {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 11px;
  padding: 9px 11px;
  border: 1px solid color-mix(in srgb, var(--cat) 20%, transparent);
  border-radius: 12px;
  background: color-mix(in srgb, var(--cat) 9%, var(--card));
}

.toast__cue-text {
  flex: 1;
  min-width: 0;
  font-size: 11.5px;
  font-weight: 500;
  line-height: 1.4;
  color: var(--text2);
  text-wrap: pretty;
}

.cue-ring {
  position: relative;
  flex-shrink: 0;
  width: 26px;
  height: 26px;

  svg {
    transform: rotate(-90deg);
  }
}

.cue-ring__track,
.cue-ring__progress {
  fill: none;
  stroke-width: 3;
}

.cue-ring__track {
  stroke: color-mix(in srgb, var(--cat) 24%, var(--card));
}

.cue-ring__progress {
  stroke: var(--cat-ink);
  stroke-linecap: round;
  stroke-dasharray: 82;
  animation: cue-ring 20s linear forwards;
}

.cue-ring__label {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 8.5px;
  font-weight: 700;
  color: var(--cat-ink);
}

.cue-posture {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  flex-shrink: 0;
  width: 26px;

  span {
    height: 3px;
    border-radius: 2px;
    background: var(--cat-ink);
  }
}

.cue-bars {
  display: flex;
  align-items: flex-end;
  gap: 3px;
  flex-shrink: 0;
  width: 26px;
  height: 22px;

  span {
    width: 4px;
    height: 22px;
    border-radius: 2px;
    background: var(--cat-ink);
    transform-origin: bottom;
    animation: cue-bar 900ms ease-in-out infinite;
  }
}

.cue-glass {
  position: relative;
  flex-shrink: 0;
  width: 18px;
  height: 24px;
  margin: 0 4px;
  overflow: hidden;
  border: 2px solid var(--cat-ink);
  border-top: none;
  border-radius: 2px 2px 7px 7px;

  span {
    position: absolute;
    right: 0;
    bottom: 0;
    left: 0;
    height: 80%;
    background: var(--cat-ink);
    opacity: 0.85;
    animation: cue-fill 2.4s ease-in-out infinite alternate;
  }
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
  animation: progress-sweep 14s linear reverse forwards;

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

  .toast__cue {
    border-color: color-mix(in srgb, var(--cat) 26%, transparent);
    background: color-mix(in srgb, var(--cat) 14%, var(--card));
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

@keyframes cue-ring {
  from {
    stroke-dashoffset: 0;
  }
  to {
    stroke-dashoffset: 82;
  }
}

@keyframes cue-bar {
  0%,
  100% {
    transform: scaleY(0.3);
  }
  50% {
    transform: scaleY(1);
  }
}

@keyframes cue-fill {
  0% {
    height: 24%;
  }
  100% {
    height: 82%;
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
