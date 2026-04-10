<template>
  <div v-if="visible" class="spinner-wrapper" :class="{ inline }">
    <div class="spinner" :style="{ width: size + 'px', height: size + 'px' }"></div>
  </div>
</template>

<script setup>
defineProps({
  visible: { type: Boolean, default: true },
  size: { type: Number, default: 40 },
  inline: { type: Boolean, default: false } 
});
</script>

<style scoped>
.spinner-wrapper {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
  min-height: 80px;
}
.spinner-wrapper.inline {
  display: inline-flex;
  width: auto;
  min-height: auto;
}
.spinner {
  border: 3px solid rgba(124, 58, 237, 0.2);
  border-top: 3px solid var(--color-action);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>