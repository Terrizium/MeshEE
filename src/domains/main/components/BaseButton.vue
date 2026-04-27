<template>
  <button
    class="base-button"
    :class="[
      `base-button--size-${size}`,
      `base-button--variant-${variant}`,
      { 'base-button--loading': loading }
    ]"
    :disabled="disabled || loading"
    :type="type"
    @click="handleClick"
  >
    <BaseSpinner v-if="loading" inline :size="spinnerSize" />
    <slot v-else />
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import BaseSpinner from './BaseSpinner.vue';

type ButtonSize = 'icon' | 'auto' | 'block';
type ButtonVariant = 'primary' | 'secondary' | 'outlined';

interface Props {
  disabled?: boolean;
  loading?: boolean;
  size?: ButtonSize;
  variant?: ButtonVariant;
  type?: 'button' | 'submit' | 'reset';
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  loading: false,
  size: 'auto',
  variant: 'primary',
  type: 'button',
});

const emit = defineEmits<{
  (e: 'click', event: MouseEvent): void;
}>();

const handleClick = (event: MouseEvent) => {
  if (!props.disabled && !props.loading) {
    emit('click', event);
  }
};

// Размер спиннера в зависимости от размера кнопки
const spinnerSize = computed(() => {
  if (props.size === 'icon') return 16;
  return 18;
});
</script>

<style scoped>
.base-button {
  font-family: inherit;
  font-weight: 600;
  font-size: 0.9rem;
  padding: 0.5rem 1.5rem;
  border-radius: 2rem;
  cursor: pointer;
  text-align: center;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  outline: none;
  transition: transform 0.1s ease, box-shadow 0.2s;
  border: none;
  position: relative;
  white-space: nowrap;
}

/* ---------------------- Размеры ---------------------- */
.base-button--size-auto {
  width: auto;
}

.base-button--size-block {
  width: 100%;
  display: flex;
}

.base-button--size-icon {
  width: 2.2rem;
  height: 2.2rem;
  padding: 0;
  border-radius: 50%;
  font-size: 1.2rem;
  flex-shrink: 0;
}

/* ---------------------- Вариант primary ---------------------- */
.base-button--variant-primary {
  background: linear-gradient(135deg, #a531d6, #34113b, #df39d1);
  background-size: 200% 200%;
  color: white;
  box-shadow: 0 0 0 0 rgba(178, 36, 239, 0);
  animation: gradientShift 5s ease infinite;
}

.base-button--variant-primary:hover:not(:disabled) {
  box-shadow: 0 0 0 6px rgba(178, 36, 239, 0.4), 0 0 20px rgba(117, 121, 255, 0.3);
}

/* ---------------------- Вариант secondary ---------------------- */
.base-button--variant-secondary {
  background: linear-gradient(135deg, #2dd4bf, #0f766e, #14b8a6);
  background-size: 200% 200%;
  color: white;
  animation: gradientShift 5s ease infinite;
}

.base-button--variant-secondary:hover:not(:disabled) {
  box-shadow: 0 0 0 6px rgba(45, 212, 191, 0.4), 0 0 20px rgba(45, 212, 191, 0.3);
}

/* ---------------------- Вариант outlined (анимированная градиентная рамка) ---------------------- */
.base-button--variant-outlined {
  background: transparent;
  color: var(--color-text, #e2e8f0);
  border: 1px solid transparent;
  z-index: 0;
}

.base-button--variant-outlined::before {
  content: '';
  position: absolute;
  inset: -1px;
  border-radius: inherit;
  padding: 1px;
  background: linear-gradient(
    90deg,
    var(--color-action, #2dd4bf),
    var(--color-primary, #a531d6),
    var(--color-action, #2dd4bf)
  );
  background-size: 200% 100%;
  -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
  mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  pointer-events: none;
  z-index: -1;
  animation: borderShift 3s ease infinite;
}

.base-button--variant-outlined:hover:not(:disabled) {
  box-shadow: 0 0 0 4px rgba(45, 212, 191, 0.2);
  color: var(--color-action, #2dd4bf);
}

/* ---------------------- Общие состояния ---------------------- */
.base-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  filter: grayscale(0.1);
  animation: none;
  box-shadow: none;
}

.base-button--loading {
  cursor: wait;
}

.base-button:active:not(:disabled) {
  transform: scale(0.98);
  transition: transform 0.05s;
}

.base-button:focus-visible {
  outline: 2px solid var(--color-text, white);
  outline-offset: 2px;
}

/* ---------------------- Анимации ---------------------- */
@keyframes gradientShift {
  0% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
  100% { background-position: 0% 50%; }
}

@keyframes borderShift {
  0% { background-position: 0% 50%; }
  100% { background-position: 200% 50%; }
}
</style>