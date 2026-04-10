<template>
  <button
    class="gradient-button"
    :disabled="disabled"
    :type="type"
    @click="handleClick"
  >
    <BaseSpinner v-if="props.loading" inline size="12"/>
    <slot v-else />
  </button>
</template>

<script setup lang="ts">
import BaseSpinner from './BaseSpinner.vue';

interface Props {
  disabled?: boolean;
  loading?: boolean;
  type?: 'button' | 'submit' | 'reset';
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  loading: false,
  type: 'button',
});

const emit = defineEmits<{
  (e: 'click', event: MouseEvent): void;
}>();

const handleClick = (event: MouseEvent) => {
  if (!props.disabled) {
    emit('click', event);
  }
};
</script>

<style scoped>
.gradient-button {
  font-family: inherit;
  font-weight: 600;
  font-size: 1rem;
  padding: 0.5rem 1.5rem;
  border: 1px solid rgba(255, 255, 255, 0.25); 
  border-radius: 2rem;
  cursor: pointer;
  color: white;
  background: linear-gradient(135deg, #a531d6, #34113b, #df39d1);
  background-size: 200% 200%;
  text-align: center;
  letter-spacing: 0.3px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  outline: none;
  transition: box-shadow 0.25s ease, transform 0.1s ease, border-color 0.2s;
  box-shadow: 0 0 0 0 rgba(178, 36, 239, 0);
  animation: gradientShift 5s ease infinite;
}

.gradient-button:hover:not(:disabled) {
  box-shadow: 0 0 0 6px rgba(178, 36, 239, 0.4), 0 0 20px rgba(117, 121, 255, 0.3);
  border-color: rgba(255, 255, 255, 0.5);
}

.gradient-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  box-shadow: none;
  animation: none;
  filter: grayscale(0.1);
}

.gradient-button:focus-visible {
  outline: 2px solid #fff;
  outline-offset: 2px;
}

@keyframes gradientShift {
  0% {
    background-position: 0% 50%;
  }
  50% {
    background-position: 100% 50%;
  }
  100% {
    background-position: 0% 50%;
  }
}

.gradient-button:active:not(:disabled) {
  transform: scale(0.98);
  transition: transform 0.05s;
}
</style>