<script setup>
import { onMounted, ref } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import AppMark from "../components/AppMark.vue";

const REPO_URL = "https://github.com/MasihTak/Pausetta";

const version = ref("");

onMounted(async () => {
  try {
    version.value = await getVersion();
  } catch (error) {
    // Outside Tauri (plain `pnpm dev`) there is no version to show.
    console.warn("[pausetta] could not read app version", error);
  }
});

function openRepo() {
  openUrl(REPO_URL).catch((error) => console.error("[pausetta] could not open link", error));
}
</script>

<template>
  <div class="about">
    <div class="about__mark">
      <div class="about__glow" />
      <div class="about__tile">
        <AppMark
          :size="28"
          class="about__logo"
        />
      </div>
    </div>

    <h1 class="about__name">
      Pausetta
    </h1>
    <p class="about__purpose">
      Gentle reminders to look after your eyes, posture, movement, and hydration while you work.
    </p>

    <div class="settings-card about__facts">
      <div
        v-if="version"
        class="about__fact"
      >
        <span class="about__fact-label">Version</span>
        <span class="about__fact-value">{{ version }}</span>
      </div>
      <div class="about__fact">
        <span class="about__fact-label">License</span>
        <span class="about__fact-value">GPL-3.0</span>
      </div>
      <div class="about__fact">
        <span class="about__fact-label">Source</span>
        <!-- href keeps link semantics; the webview itself must not navigate, so Rust opens it. -->
        <a
          class="about__fact-value about__fact-link"
          :href="REPO_URL"
          @click.prevent="openRepo"
        >GitHub</a>
      </div>
    </div>

    <p class="about__note">
      No account, no telemetry, no network. Everything stays on your machine.
    </p>
    <p class="about__note">
      Intervals are sensible defaults based on available research, not medical advice.
    </p>
  </div>
</template>

<style scoped lang="scss">
.about {
  display: flex;
  flex: 1 0 auto;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 38px 26px 30px;
  text-align: center;
}

.about__mark {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 72px;
  height: 72px;
}

.about__glow {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  pointer-events: none;
  background: radial-gradient(circle, color-mix(in srgb, var(--accent) 22%, transparent), transparent 70%);
  animation: breathe 7s ease-in-out infinite;
}

.about__tile {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 58px;
  height: 58px;
  border-radius: 19px;
  background: var(--card);
  box-shadow:
    inset 0 0 0 1px var(--border),
    0 8px 20px -12px rgba(24, 28, 26, 0.4);
}

.about__logo {
  color: var(--accent);
}

.about__name {
  margin: 18px 0 0;
  font-size: 22px;
  font-weight: 600;
  letter-spacing: -0.018em;
  color: var(--text);
}

.about__purpose {
  max-width: 296px;
  margin: 8px 0 0;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text2);
  text-wrap: pretty;
}

.about__facts {
  width: 100%;
  margin-top: 26px;
}

.about__fact {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 13px 16px;

  // Matches .settings-row: an inset shadow keeps the row's full padding.
  &:not(:last-child) {
    box-shadow: inset 0 -1px 0 var(--border);
  }
}

.about__fact-label {
  font-size: 12.5px;
  color: var(--text3);
}

.about__fact-value {
  font-size: 12.5px;
  font-weight: 500;
  color: var(--text);
}

.about__fact-link {
  font-weight: 600;
}

.about__note {
  max-width: 280px;
  margin: 20px 0 0;
  font-size: 11.5px;
  line-height: 1.55;
  color: var(--text3);
  text-wrap: pretty;

  & + & {
    margin-top: 10px;
  }
}

@keyframes breathe {
  0%,
  100% {
    transform: scale(1);
    opacity: 0.5;
  }
  50% {
    transform: scale(1.35);
    opacity: 0.9;
  }
}
</style>
