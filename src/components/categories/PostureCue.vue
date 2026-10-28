<template>
  <div
    class="cue-posture"
    aria-hidden="true"
  >
    <span class="cue-posture__line cue-posture__line--long" />
    <span class="cue-posture__line cue-posture__line--short" />
    <span class="cue-posture__line cue-posture__line--longest" />
  </div>
</template>

<style scoped lang="scss">
.cue-posture {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  flex-shrink: 0;
  width: 26px;
}

.cue-posture__line {
  height: 3px;
  border-radius: 2px;
  background: var(--cat-ink);

  &--long {
    width: 16px;
  }

  &--short {
    width: 11px;
    opacity: 0.75;
  }

  &--longest {
    width: 20px;
    opacity: 0.5;
  }
}
</style>
