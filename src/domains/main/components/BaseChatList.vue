<template>
  <div
    class="chat-item"
    :class="{ 'chat-item--active': active, 'chat-item--unread': chat.has_unread }"
    @click="handleClick"
    v-memo="[chat.id, chat.has_unread, active, chat.login]"
  >
    <div class="chat-item__avatar">
      {{ chat.login.charAt(0).toUpperCase() }}
    </div>
    <div class="chat-item__info">
      <div class="chat-item__login">{{ chat.login }}</div>
      <div class="chat-item__status" v-if="chat.has_unread">Новое сообщение</div>
    </div>
    <div class="chat-item__unread-badge" v-if="chat.has_unread">
      <span class="badge"></span>
    </div>
  </div>
</template>

<script setup lang="ts">
export interface Chat {
  id: number;
  login: string;
  has_unread: boolean;
}

const props = defineProps<{
  chat: Chat;
  active?: boolean;
}>();

const emit = defineEmits<{
  (e: 'select', chat: Chat): void;
}>();

const handleClick = () => {
  emit('select', props.chat);
};
</script>

<style scoped>
.chat-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-md, 12px);
  padding: var(--spacing-md, 12px) var(--spacing-lg, 16px);
  height: 64px; /* фиксированная высота для virtual scroll */
  background-color: var(--color-surface, #1e1e2a);
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  cursor: pointer;
  transition: background-color 0.15s ease;
  user-select: none;
}

.chat-item:hover {
  background-color: rgba(124, 58, 237, 0.1);
}

.chat-item--active {
  background-color: rgba(124, 58, 237, 0.2);
  border-left: 3px solid var(--color-primary, #7c3aed);
  padding-left: calc(var(--spacing-lg, 16px) - 3px);
}

.chat-item--unread .chat-item__login {
  font-weight: 600;
  color: var(--color-primary, #7c3aed);
}

.chat-item__avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: linear-gradient(135deg, #7c3aed, #a855f7);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  font-size: 18px;
  color: white;
  text-transform: uppercase;
  flex-shrink: 0;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.chat-item__info {
  flex: 1;
  min-width: 0;
}

.chat-item__login {
  font-size: 15px;
  font-weight: 500;
  color: var(--color-text, #e2e8f0);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.chat-item__status {
  font-size: 12px;
  color: var(--color-text-secondary, #94a3b8);
  margin-top: 2px;
}

.chat-item__unread-badge {
  flex-shrink: 0;
  margin-left: 8px;
}

.badge {
  display: block;
  width: 10px;
  height: 10px;
  background-color: var(--color-primary, #7c3aed);
  border-radius: 50%;
  box-shadow: 0 0 0 2px var(--color-surface, #1e1e2a);
}
</style>