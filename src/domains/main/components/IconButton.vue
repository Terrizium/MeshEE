<template>
  <div ref="buttonContainer" class="icon-menu-wrapper">
    <button
      ref="buttonRef"
      class="icon-btn"
      :class="{ 'icon-btn--active': isOpen }"
      @click.stop="toggleMenu"
    >
      <slot name="icon">
        <!-- Иконка шестерёнки по умолчанию -->
        <svg
          class="icon-btn__svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H5.78a1.65 1.65 0 0 0-1.51 1 1.65 1.65 0 0 0 .33 1.82L12 22Z" />
          <path d="M4.6 9a1.65 1.65 0 0 0-.33 1.82c.26.5.86.82 1.51.82H18.22c.65 0 1.25-.32 1.51-.82A1.65 1.65 0 0 0 19.4 9L12 2Z" />
        </svg>
      </slot>
    </button>

    <Transition name="menu">
      <div v-if="isOpen" ref="menuRef" class="icon-menu">
        <div class="icon-menu__content">
          <slot @click="toggleMenu"></slot>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';

const props = defineProps<{
  placement?: 'bottom-start' | 'bottom-end' | 'top-start' | 'top-end';
}>();

const emit = defineEmits<{
  (e: 'open'): void;
  (e: 'close'): void;
}>();

const isOpen = ref(false);
const buttonRef = ref<HTMLElement | null>(null);
const menuRef = ref<HTMLElement | null>(null);
const buttonContainer = ref<HTMLElement | null>(null);

const toggleMenu = () => {
  if (isOpen.value) {
    closeMenu();
  } else {
    openMenu();
  }
};

const openMenu = async () => {
  isOpen.value = true;
  emit('open');
  await nextTick();
  // позиционирование меню относительно кнопки
  positionMenu();
  // добавляем глобальный обработчик закрытия
  document.addEventListener('click', handleClickOutside);
};

const closeMenu = () => {
  if (!isOpen.value) return;
  isOpen.value = false;
  emit('close');
  document.removeEventListener('click', handleClickOutside);
};

const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as Node;
  if (
    buttonContainer.value &&
    !buttonContainer.value.contains(target) &&
    menuRef.value &&
    !menuRef.value.contains(target)
  ) {
    closeMenu();
  }
};

const positionMenu = () => {
  if (!buttonRef.value || !menuRef.value) return;
  const buttonRect = buttonRef.value.getBoundingClientRect();
  const menuEl = menuRef.value;

  // Сброс стилей для пересчёта
  menuEl.style.top = '';
  menuEl.style.bottom = '';
  menuEl.style.left = '';
  menuEl.style.right = '';

  const placement = props.placement || 'bottom-end';
  const margin = 8; // отступ от кнопки

  if (placement === 'bottom-start') {
    menuEl.style.top = `${buttonRect.bottom + margin}px`;
    menuEl.style.left = `${buttonRect.left}px`;
  } else if (placement === 'bottom-end') {
    menuEl.style.top = `${buttonRect.bottom + margin}px`;
    menuEl.style.right = `${window.innerWidth - buttonRect.right}px`;
  } else if (placement === 'top-start') {
    menuEl.style.bottom = `${window.innerHeight - buttonRect.top + margin}px`;
    menuEl.style.left = `${buttonRect.left}px`;
  } else if (placement === 'top-end') {
    menuEl.style.bottom = `${window.innerHeight - buttonRect.top + margin}px`;
    menuEl.style.right = `${window.innerWidth - buttonRect.right}px`;
  }
};

// При изменении размера окна перепозиционируем меню, если открыто
const handleResize = () => {
  if (isOpen.value) {
    positionMenu();
  }
};

onMounted(() => {
  window.addEventListener('resize', handleResize);
});

onUnmounted(() => {
  window.removeEventListener('resize', handleResize);
  document.removeEventListener('click', handleClickOutside);
});

// Закрыть меню из слота можно вызвав closeMenu (например, по клику на пункт)
defineExpose({
  closeMenu,
  openMenu,
  toggleMenu,
});
</script>

<style scoped>
.icon-menu-wrapper {
  position: relative;
  display: inline-block;
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--color-text-secondary, #94a3b8);
  border-radius: var(--radius-md, 8px);
  cursor: pointer;
  transition: all 0.2s ease;
}

.icon-btn:hover {
  background-color: rgba(255, 255, 255, 0.1);
  color: var(--color-text, #e2e8f0);
}

.icon-btn--active {
  color: var(--color-action, #2dd4bf);
}

.icon-btn--active .icon-btn__svg {
  transform: rotate(90deg);
}

.icon-btn__svg {
  width: 20px;
  height: 20px;
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.icon-menu {
  position: fixed;
  z-index: 1000;
  min-width: 180px;
  background-color: var(--color-surface, #1e1e2a);
  border-radius: var(--radius-lg, 12px);
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.08);
  overflow: hidden;
  backdrop-filter: blur(8px);
}

.icon-menu__content {
  display: flex;
  flex-direction: column;
  padding: 8px 0;
}

/* Анимация меню */
.menu-enter-active,
.menu-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.menu-enter-from {
  opacity: 0;
  transform: scale(0.95) translateY(-8px);
}

.menu-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(-8px);
}
</style>