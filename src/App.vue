<script setup>
import { computed, onMounted, onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import PauseMenu from "./components/PauseMenu.vue";
import AboutView from "./views/AboutView.vue";
import OnboardingView from "./views/OnboardingView.vue";
import SettingsView from "./views/SettingsView.vue";
import { useSettingsStore } from "./stores/settings.js";

const AUTHOR_URL = "https://masih.dev";

const store = useSettingsStore();
// Rust injects the screen when it creates this window (see settings_window.rs); an
// already-open window is told which screen to show with an event instead.
const screen = ref(window.__PAUSETTA_SCREEN__ ?? "settings");
const loadError = ref("");

let stopListening = [];

// Shown once per install; Rust persists the flag in SQLite, so it survives frontend
// rebuilds and only resets if the app's local data is deleted.
const isOnboarding = computed(
  () => Boolean(store.settings) && !store.settings.onboardingCompleted,
);

onMounted(async () => {
  // A failed listener must not stop settings from loading, or the window stays blank.
  const registrations = await Promise.allSettled([
    listen("show-screen", (event) => (screen.value = event.payload)),
    // The tray can pause or resume reminders behind this window's back.
    listen("settings-changed", () =>
      store.load().catch((error) => console.error("[pausetta] could not reload settings", error)),
    ),
  ]);
  stopListening = registrations
    .filter((registration) => registration.status === "fulfilled")
    .map((registration) => registration.value);
  registrations
    .filter((registration) => registration.status === "rejected")
    .forEach((registration) =>
      console.error("[pausetta] could not listen for window events", registration.reason),
    );

  try {
    await store.load();
  } catch (error) {
    loadError.value = String(error);
  }
});

onUnmounted(() => stopListening.forEach((stop) => stop()));

function completeOnboarding() {
  store
    .update((settings) => (settings.onboardingCompleted = true))
    .catch((error) => console.error("[pausetta] could not save onboarding completion", error));
}

function openAuthorSite() {
  openUrl(AUTHOR_URL).catch((error) => console.error("[pausetta] could not open link", error));
}
</script>

<template>
  <OnboardingView
    v-if="isOnboarding"
    @finish="completeOnboarding"
  />

  <div
    v-else
    class="app"
  >
    <header class="app-header">
      <template v-if="screen === 'settings'">
        <PauseMenu v-if="store.settings" />
        <span v-else />
        <button
          type="button"
          class="icon-button"
          aria-label="About"
          @click="screen = 'about'"
        >
          <svg
            width="15"
            height="15"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            aria-hidden="true"
          >
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
          </svg>
        </button>
      </template>
      <button
        v-else
        type="button"
        class="back-button"
        @click="screen = 'settings'"
      >
        <svg
          width="13"
          height="13"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <polyline points="15 18 9 12 15 6" />
        </svg>
        Back
      </button>
    </header>

    <main>
      <AboutView v-if="screen === 'about'" />
      <p
        v-else-if="loadError"
        class="load-error"
        role="alert"
      >
        Couldn't load settings: {{ loadError }}
      </p>
      <template v-else-if="store.settings">
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
        <SettingsView />
      </template>
    </main>

    <footer class="app-footer">
      <span>Built by</span>
      <!-- href keeps link semantics; the webview itself must not navigate, so Rust opens it. -->
      <a
        :href="AUTHOR_URL"
        @click.prevent="openAuthorSite"
      >MasihTak</a>
    </footer>
  </div>
</template>

<style scoped lang="scss">
.app {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  background: var(--bg);
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
  height: 46px;
  padding: 0 16px;
  border-bottom: 1px solid var(--border);
}

// A column so a short screen (About) can fill the window and centre itself, while a
// tall one (Settings) keeps its natural height and scrolls.
main {
  display: flex;
  flex: 1;
  flex-direction: column;
}

.app-footer {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 14px 16px 16px;
  box-shadow: inset 0 1px 0 var(--border);
  font-size: 11.5px;
  color: var(--text3);

  a {
    font-weight: 600;
  }
}

.icon-button,
.back-button {
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--text2);
  cursor: pointer;

  &:hover {
    background: var(--border);
  }
}

.icon-button {
  width: 28px;
  height: 28px;
  border-radius: 6px;
}

.back-button {
  gap: 5px;
  padding: 6px 10px 6px 6px;
  border-radius: 7px;
  font-size: 12.5px;
}

.load-error {
  margin: 0;
  padding: 18px 16px;
  font-size: 13px;
  color: var(--text2);
}

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
