<template>
  <div 
    class="app-bar" 
  >
<!-- :class="{ 'app-bar--draggable': draggable }" -->
    <div class="app-bar__start">
      <slot name="start"></slot>
    </div>

    <div 
      class="app-bar__center"
    >
      <slot name="title">
        <span class="app-bar__title">{{ title }}</span>
      </slot>
    </div>

    <div class="app-bar__end">
      <slot name="end"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  title?: string;
}

withDefaults(defineProps<Props>(), {
  title: '',
});
</script>

<style scoped>
.app-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 48px;
  padding: 0 var(--spacing-lg, 16px);
  background-color: var(--color-surface, #1e1e2a);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  gap: var(--spacing-md, 12px);
  flex-shrink: 0;
}

/* Область для перетаскивания окна (только если draggable=true) */
.app-bar--draggable {
  -webkit-app-region: drag;
  user-select: none;
}

/* Дочерние элементы не должны перетаскивать окно (кнопки, интерактив) */
.app-bar__start,
.app-bar__end,
.app-bar__center :deep(button),
.app-bar__center :deep(a),
.app-bar__center :deep(.no-drag) {
  -webkit-app-region: no-drag;
}

/* Левый слот — выравнивание влево */
.app-bar__start {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm, 8px);
  flex-shrink: 0;
}

/* Центральная область — заголовок (может быть гибкой) */
.app-bar__center {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  text-align: center;
  min-width: 0;
}

/* Если центральный слот не используется, заголовок не переносится */
.app-bar__title {
  font-size: 18px;
  font-weight: 500;
  color: var(--color-text, #e2e8f0);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  padding: 0 var(--spacing-sm, 8px);
}

/* Правый слот */
.app-bar__end {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm, 8px);
  flex-shrink: 0;
}

/* Кнопки внутри слотов — общие стили (опционально) */
.app-bar :deep(.app-bar-btn) {
  background: transparent;
  border: none;
  color: var(--color-text-secondary, #94a3b8);
  cursor: pointer;
  padding: 6px;
  border-radius: var(--radius-sm, 4px);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.app-bar :deep(.app-bar-btn:hover) {
  background-color: rgba(255, 255, 255, 0.1);
  color: var(--color-text, #e2e8f0);
}

.app-bar :deep(.app-bar-btn.active) {
  color: var(--color-primary, #7c3aed);
}
</style>