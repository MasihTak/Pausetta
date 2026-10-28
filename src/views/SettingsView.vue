<script setup>
import CategoryRow from "../components/CategoryRow.vue";
import SettingRow from "../components/SettingRow.vue";
import SelectField from "../components/SelectField.vue";
import ToggleSwitch from "../components/ToggleSwitch.vue";
import { CATEGORIES, HALF_HOUR_TIME_OPTIONS, IDLE_OPTIONS } from "../constants/categories.js";
import { useSettingModel } from "../stores/settings.js";

const isQuietHoursEnabled = useSettingModel("quietHours.enabled");
const quietHoursStart = useSettingModel("quietHours.start");
const quietHoursEnd = useSettingModel("quietHours.end");
const isIdlePauseEnabled = useSettingModel("idlePause.enabled");
const idleMinutes = useSettingModel("idlePause.minutes");
const isLaunchOnLoginEnabled = useSettingModel("launchOnLogin");
const isSoundEnabled = useSettingModel("soundEnabled");
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
        <SettingRow
          title="Quiet hours"
          description="Pause reminders automatically during these hours."
          :is-dimmed="!isQuietHoursEnabled"
        >
          <template #control>
            <ToggleSwitch
              v-model="isQuietHoursEnabled"
              label="Quiet hours"
            />
          </template>
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
        </SettingRow>

        <SettingRow
          title="Auto-pause when idle"
          description="Pauses reminders when you step away, after"
          :is-dimmed="!isIdlePauseEnabled"
        >
          <template #control>
            <ToggleSwitch
              v-model="isIdlePauseEnabled"
              label="Auto-pause when idle"
            />
          </template>
          <template #detail>
            <SelectField
              v-model="idleMinutes"
              :options="IDLE_OPTIONS"
              label="Idle time before pausing"
              :disabled="!isIdlePauseEnabled"
            />
          </template>
        </SettingRow>

        <SettingRow
          title="Launch on login"
          description="Start automatically when you sign in."
        >
          <template #control>
            <ToggleSwitch
              v-model="isLaunchOnLoginEnabled"
              label="Launch on login"
            />
          </template>
        </SettingRow>

        <SettingRow
          title="Notification sound"
          description="Play a sound when a reminder appears."
        >
          <template #control>
            <ToggleSwitch
              v-model="isSoundEnabled"
              label="Notification sound"
            />
          </template>
        </SettingRow>
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
</style>
