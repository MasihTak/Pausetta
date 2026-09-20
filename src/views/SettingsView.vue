<script setup>
import CategoryRow from "../components/CategoryRow.vue";
import SelectField from "../components/SelectField.vue";
import ToggleSwitch from "../components/ToggleSwitch.vue";
import { CATEGORIES, HALF_HOUR_TIME_OPTIONS, IDLE_OPTIONS } from "../constants/categories.js";
import { useSettingModel } from "../stores/settings.js";

const isQuietHoursEnabled = useSettingModel(
  (settings) => settings.quietHours.enabled,
  (settings, enabled) => (settings.quietHours.enabled = enabled),
);
const quietHoursStart = useSettingModel(
  (settings) => settings.quietHours.start,
  (settings, time) => (settings.quietHours.start = time),
);
const quietHoursEnd = useSettingModel(
  (settings) => settings.quietHours.end,
  (settings, time) => (settings.quietHours.end = time),
);
const isIdlePauseEnabled = useSettingModel(
  (settings) => settings.idlePause.enabled,
  (settings, enabled) => (settings.idlePause.enabled = enabled),
);
const idleMinutes = useSettingModel(
  (settings) => settings.idlePause.minutes,
  (settings, minutes) => (settings.idlePause.minutes = minutes),
);
const isLaunchOnLoginEnabled = useSettingModel(
  (settings) => settings.launchOnLogin,
  (settings, enabled) => (settings.launchOnLogin = enabled),
);
const isSoundEnabled = useSettingModel(
  (settings) => settings.soundEnabled,
  (settings, enabled) => (settings.soundEnabled = enabled),
);
</script>

<template>
  <div class="settings">
    <section>
      <h2 class="settings-section__label">
        Reminders
      </h2>
      <div class="settings-card">
        <CategoryRow
          v-for="category in CATEGORIES"
          :key="category.key"
          :category="category"
        />
      </div>
    </section>

    <section>
      <h2 class="settings-section__label">
        General
      </h2>
      <div class="settings-card">
        <div class="settings-row">
          <div class="settings-row__head">
            <span class="settings-row__title">Quiet hours</span>
            <ToggleSwitch
              v-model="isQuietHoursEnabled"
              label="Quiet hours"
            />
          </div>
          <div
            class="settings-row__detail"
            :class="{ 'is-dimmed': !isQuietHoursEnabled }"
          >
            <span class="settings-row__description">
              Pause reminders automatically during these hours.
            </span>
          </div>
          <div
            class="quiet-hours-range"
            :class="{ 'is-dimmed': !isQuietHoursEnabled }"
          >
            <span class="range-label">From</span>
            <SelectField
              v-model="quietHoursStart"
              :options="HALF_HOUR_TIME_OPTIONS"
              label="Quiet hours start"
              :disabled="!isQuietHoursEnabled"
              is-compact
            />
            <span class="range-label">to</span>
            <SelectField
              v-model="quietHoursEnd"
              :options="HALF_HOUR_TIME_OPTIONS"
              label="Quiet hours end"
              :disabled="!isQuietHoursEnabled"
              is-compact
            />
          </div>
        </div>

        <div class="settings-row">
          <div class="settings-row__head">
            <span class="settings-row__title">Auto-pause when idle</span>
            <ToggleSwitch
              v-model="isIdlePauseEnabled"
              label="Auto-pause when idle"
            />
          </div>
          <div
            class="settings-row__detail"
            :class="{ 'is-dimmed': !isIdlePauseEnabled }"
          >
            <span class="settings-row__description">Pauses reminders when you step away, after</span>
            <SelectField
              v-model="idleMinutes"
              :options="IDLE_OPTIONS"
              label="Idle time before pausing"
              :disabled="!isIdlePauseEnabled"
            />
          </div>
        </div>

        <div class="settings-row">
          <div class="settings-row__head">
            <div class="settings-row__text">
              <div class="settings-row__title">
                Launch on login
              </div>
              <div class="settings-row__description">
                Start automatically when you sign in.
              </div>
            </div>
            <ToggleSwitch
              v-model="isLaunchOnLoginEnabled"
              label="Launch on login"
            />
          </div>
        </div>

        <div class="settings-row">
          <div class="settings-row__head">
            <div class="settings-row__text">
              <div class="settings-row__title">
                Notification sound
              </div>
              <div class="settings-row__description">
                Play a sound when a reminder appears.
              </div>
            </div>
            <ToggleSwitch
              v-model="isSoundEnabled"
              label="Notification sound"
            />
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped lang="scss">
.settings {
  display: flex;
  // Never shrink: the list is taller than the window, so it must scroll, not squash.
  flex: 1 0 auto;
  flex-direction: column;
  gap: 22px;
  padding: 18px 15px;
}

h2 {
  margin: 0;
}

.quiet-hours-range {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
  transition: opacity 150ms ease;
}

.range-label {
  font-size: 11.5px;
  color: var(--text3);
}

.settings-row__text {
  flex: 1;

  .settings-row__description {
    margin-top: 3px;
  }
}
</style>
