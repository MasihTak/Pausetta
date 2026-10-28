<template>
  <div
    class="cue-glass"
    aria-hidden="true"
  >
    <span />
  </div>
</template>

<style scoped lang="scss">
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

@keyframes cue-fill {
  0% {
    height: 24%;
  }
  100% {
    height: 82%;
  }
}
</style>
