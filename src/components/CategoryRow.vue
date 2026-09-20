<script setup>
import CategoryIcon from "./CategoryIcon.vue";
import SelectField from "./SelectField.vue";
import ToggleSwitch from "./ToggleSwitch.vue";
import { INTERVAL_OPTIONS } from "../constants/categories.js";
import { useSettingModel } from "../stores/settings.js";

const props = defineProps({
  category: { type: Object, required: true },
});

const categoryKey = props.category.key;

const isEnabled = useSettingModel(
  (settings) => settings.categories[categoryKey].enabled,
  (settings, enabled) => {
    settings.categories[categoryKey].enabled = enabled;
  },
);

const intervalMinutes = useSettingModel(
  (settings) => settings.categories[categoryKey].intervalMinutes,
  (settings, minutes) => {
    settings.categories[categoryKey].intervalMinutes = minutes;
  },
);
</script>

<template>
  <div class="settings-row">
    <div class="settings-row__head">
      <span
        class="badge"
        :style="{ '--cat': `var(--cat-${categoryKey})` }"
      >
        <CategoryIcon
          :category="categoryKey"
          :size="16"
        />
      </span>
      <span class="settings-row__title">{{ category.label }}</span>
      <ToggleSwitch
        v-model="isEnabled"
        :label="category.label"
      />
    </div>
    <div
      class="settings-row__detail category-detail"
      :class="{ 'is-dimmed': !isEnabled }"
    >
      <span class="settings-row__description">{{ category.description }}</span>
      <SelectField
        v-model="intervalMinutes"
        :options="INTERVAL_OPTIONS"
        :label="`${category.label} interval`"
        :disabled="!isEnabled"
      />
    </div>
  </div>
</template>

<style scoped lang="scss">
.badge {
  position: relative;
  // Optically centres the badge against the title and the description below it.
  top: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 34px;
  height: 34px;
  border-radius: 12px;
  background: linear-gradient(
    150deg,
    color-mix(in srgb, var(--cat) var(--cat-mix-strong), var(--card)),
    color-mix(in srgb, var(--cat) var(--cat-mix-soft), var(--card))
  );
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--cat) var(--cat-mix-ring), transparent);
  color: var(--cat);
}

// Aligns the description with the title, past the 34px badge + 10px gap.
.category-detail {
  margin-left: 44px;
}
</style>
