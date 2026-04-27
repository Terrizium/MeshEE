<template>
  <Teleport to="body">
    <Transition name="modal-overlay">
      <div v-if="visible" class="modal-overlay" @click.self="handleOverlayClick">
        <Transition name="modal-content">
          <div class="modal" :class="{ 'modal--mobile': isMobile }">
            <!-- Заголовок с кнопкой закрытия (на мобильных — стрелка назад) -->
            <div class="modal__header">
              <button v-if="isMobile" class="modal__back-btn" @click="close">
                ←
              </button>
              <h2 v-if="title" class="modal__title">{{ title }}</h2>
              <button v-if="showClose && !isMobile" class="modal__close-btn" @click="close">
                ✕
              </button>
            </div>
            <div class="modal__body">
              <slot></slot>
            </div>
            <div v-if="$slots.footer" class="modal__footer">
              <slot name="footer"></slot>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { useBreakpoints } from '../../../composables/useBreakpoints';


const props = defineProps<{
  visible: boolean;
  title?: string;
  showClose?: boolean;      // показывать крестик на десктопе
  closeOnOverlay?: boolean; // закрывать по клику на оверлей
}>();

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'close'): void;
}>();

const { isMobile } = useBreakpoints();

const close = () => {
  emit('update:visible', false);
  emit('close');
};

const handleOverlayClick = () => {
  if (props.closeOnOverlay !== false) {
    close();
  }
};
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.modal {
  background-color: var(--color-surface, #1e1e2a);
  border-radius: var(--radius-lg, 16px);
  box-shadow: 0 25px 40px rgba(0, 0, 0, 0.2);
  width: 90%;
  max-width: 500px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  transition: transform 0.2s ease;
}

.modal--mobile {
  width: 100%;
  max-width: none;
  height: 100%;
  max-height: none;
  border-radius: 0;
  background-color: var(--color-bg, #121217);
}

.modal__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background-color: inherit;
}

.modal__title {
  margin: 0;
  font-size: 18px;
  font-weight: 500;
  color: var(--color-text, #e2e8f0);
  flex: 1;
  text-align: center;
}

.modal__back-btn,
.modal__close-btn {
  background: transparent;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--color-text-secondary, #94a3b8);
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm, 6px);
  transition: all 0.15s;
}

.modal__back-btn:hover,
.modal__close-btn:hover {
  background-color: rgba(255, 255, 255, 0.1);
  color: var(--color-text, #e2e8f0);
}

.modal__body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.modal__footer {
  padding: 12px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

/* Анимации */
.modal-overlay-enter-active,
.modal-overlay-leave-active {
  transition: opacity 0.2s ease;
}
.modal-overlay-enter-from,
.modal-overlay-leave-to {
  opacity: 0;
}

.modal-content-enter-active,
.modal-content-leave-active {
  transition: transform 0.2s ease, opacity 0.2s ease;
}
.modal-content-enter-from,
.modal-content-leave-to {
  transform: scale(0.95);
  opacity: 0;
}
</style>