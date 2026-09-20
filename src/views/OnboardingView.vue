<script setup>
import { computed, ref } from "vue";
import OnboardingScene from "../components/OnboardingScene.vue";
import { ONBOARDING_SLIDES } from "../constants/onboarding.js";

const emit = defineEmits(["finish"]);

const stepIndex = ref(0);
const isLastStep = computed(() => stepIndex.value === ONBOARDING_SLIDES.length - 1);

// The welcome slide has no category of its own, so it borrows the app accent.
const slideColor = (slide) => (slide.key === "welcome" ? "var(--accent)" : `var(--cat-${slide.key})`);

function goNext() {
  if (isLastStep.value) {
    emit("finish");
    return;
  }
  stepIndex.value += 1;
}

function goBack() {
  stepIndex.value = Math.max(0, stepIndex.value - 1);
}
</script>

<template>
  <div
    class="onboarding"
    :style="{ '--slide': slideColor(ONBOARDING_SLIDES[stepIndex]) }"
  >
    <header class="onboarding__header">
      <span class="onboarding__step">{{ stepIndex + 1 }} of {{ ONBOARDING_SLIDES.length }}</span>
      <button
        v-if="!isLastStep"
        type="button"
        class="onboarding__skip"
        @click="emit('finish')"
      >
        Skip
      </button>
    </header>

    <div class="onboarding__viewport">
      <div
        class="onboarding__track"
        :style="{ transform: `translateX(-${stepIndex * 100}%)` }"
      >
        <section
          v-for="(slide, index) in ONBOARDING_SLIDES"
          :key="slide.key"
          class="slide"
          :class="{ 'is-active': index === stepIndex }"
          :style="{ '--slide': slideColor(slide) }"
          :aria-hidden="index !== stepIndex"
        >
          <div class="slide__stage">
            <div class="slide__glow" />
            <OnboardingScene
              :category="slide.key"
              class="slide__scene"
            />
          </div>
          <h1 class="slide__title">
            {{ slide.title }}
          </h1>
          <p class="slide__body">
            {{ slide.body }}
          </p>
          <!-- The caveat reads as a margin note, not more body copy. -->
          <aside
            v-if="slide.note"
            class="slide__note"
          >
            {{ slide.note }}
          </aside>
        </section>
      </div>
    </div>

    <div class="dots">
      <button
        v-for="(slide, index) in ONBOARDING_SLIDES"
        :key="slide.key"
        type="button"
        class="dots__dot"
        :class="{ 'is-active': index === stepIndex }"
        :aria-label="`Go to ${slide.title}`"
        :aria-current="index === stepIndex"
        @click="stepIndex = index"
      />
    </div>

    <footer class="onboarding__actions">
      <button
        v-if="stepIndex > 0"
        type="button"
        class="onboarding__back"
        @click="goBack"
      >
        Back
      </button>
      <span v-else />
      <button
        type="button"
        class="onboarding__next"
        @click="goNext"
      >
        {{ isLastStep ? "Get started" : "Next" }}
      </button>
    </footer>
  </div>
</template>

<style scoped lang="scss">
.onboarding {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.onboarding__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
  height: 46px;
  padding: 0 16px;
  border-bottom: 1px solid var(--border);
}

.onboarding__step {
  font-size: 11.5px;
  color: var(--text3);
}

.onboarding__skip {
  padding: 6px 10px;
  border: none;
  border-radius: 7px;
  background: transparent;
  color: var(--text2);
  font-size: 12.5px;
  cursor: pointer;

  &:hover {
    background: var(--border);
  }
}

.onboarding__viewport {
  display: flex;
  flex: 1;
  overflow: hidden;
}

// Exactly one viewport wide (so a slide's flex-basis of 100% is the window, not the
// track), while the row's default stretch gives it the window's full spare height.
.onboarding__track {
  display: flex;
  flex: 0 0 100%;
  // Without this, min-width:auto floors the track at the combined width of all five
  // slides, and each slide's 100% basis would resolve against that instead of the window.
  min-width: 0;
  transition: transform 340ms cubic-bezier(0.3, 0.85, 0.3, 1);
}

.slide {
  display: flex;
  flex: 0 0 100%;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-width: 0;
  padding: 24px 30px;
  text-align: center;

  // All five slides stay mounted for the swipe transition; only the active one needs
  // its scene and cue animations actually running.
  &:not(.is-active) :deep(*) {
    animation-play-state: paused;
  }
}

.slide__stage {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 100%;
  max-width: 216px;
}

.slide__glow {
  position: absolute;
  inset: -22px -10px;
  border-radius: 50%;
  pointer-events: none;
  background: radial-gradient(circle, color-mix(in srgb, var(--slide) 20%, transparent), transparent 70%);
}

.slide__scene {
  position: relative;
  color: var(--slide);
}

.slide__title {
  margin: 18px 0 0;
  font-size: 26px;
  font-weight: 700;
  letter-spacing: -0.03em;
  line-height: 1.15;
  color: var(--text);
}

.slide__body {
  max-width: 320px;
  margin: 10px 0 0;
  font-size: 14px;
  line-height: 1.65;
  color: var(--text2);
  text-wrap: pretty;
}

.slide__note {
  max-width: 292px;
  margin: 16px 0 0;
  padding-left: 11px;
  border-left: 2px solid color-mix(in srgb, var(--slide) 55%, transparent);
  font-size: 11.5px;
  line-height: 1.55;
  color: var(--text3);
  text-align: left;
  text-wrap: pretty;
}

// Each part of the active slide arrives just after the one above it, so the screen
// assembles itself rather than appearing all at once.
.slide.is-active {
  .slide__stage,
  .slide__title,
  .slide__body,
  .slide__note {
    animation: slide-in 420ms cubic-bezier(0.25, 0.9, 0.3, 1) backwards;
  }

  .slide__title {
    animation-delay: 70ms;
  }

  .slide__body {
    animation-delay: 130ms;
  }

  .slide__note {
    animation-delay: 190ms;
  }
}

.dots {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 20px 0 16px;
}

.dots__dot {
  width: 6px;
  height: 6px;
  padding: 0;
  border: none;
  border-radius: 999px;
  background: var(--border-strong);
  cursor: pointer;
  transition:
    width 220ms ease,
    background-color 220ms ease;

  &.is-active {
    width: 18px;
    background: var(--accent);
  }
}

.onboarding__actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 26px 28px;
}

.onboarding__back {
  padding: 9px 14px;
  border: 1px solid var(--border-strong);
  border-radius: 9px;
  background: transparent;
  color: var(--text2);
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 160ms ease;

  &:hover {
    background: var(--field-bg);
  }

  &:active {
    transform: translateY(1px);
  }
}

.onboarding__next {
  padding: 10px 22px;
  border: none;
  border-radius: 10px;
  background: color-mix(in srgb, var(--slide) 94%, #2a1410);
  box-shadow: 0 4px 14px -4px color-mix(in srgb, var(--slide) 55%, transparent);
  color: #ffffff;
  font-size: 13.5px;
  font-weight: 600;
  cursor: pointer;
  transition:
    background-color 240ms ease,
    box-shadow 160ms ease,
    transform 160ms ease;

  &:hover {
    box-shadow: 0 6px 18px -4px color-mix(in srgb, var(--slide) 65%, transparent);
    transform: translateY(-1px);
  }

  &:active {
    transform: translateY(1px);
  }
}

@keyframes slide-in {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
}

@media (prefers-color-scheme: dark) {
  .onboarding__next {
    background: var(--slide);
    color: #15110e;
  }
}
</style>
