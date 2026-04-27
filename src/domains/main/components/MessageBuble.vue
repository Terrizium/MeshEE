<template>
  <div class="message" :class="`message--${isOwn ? 'own' : 'other'}`">
    <div class="message__avatar" v-if="!isOwn">
      {{ senderInitial }}
    </div>
    <div class="message__content">
      <div class="message__header" v-if="!isOwn">
        <span class="message__sender">{{ senderLogin }}</span>
      </div>
      <div class="message__bubble">
        <p class="message__text">{{ message.body }}</p>
        <span class="message__time">{{ formattedTime }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Message } from '../../../types';

const props = defineProps<{
  message: Message;
  isOwn: boolean;      // message.user_id === currentUserId
  senderLogin?: string; // для чужих сообщений – логин отправителя
}>();

const formattedTime = computed(() => {
  const date = new Date(props.message.date);
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
});

const senderInitial = computed(() => {
  return props.senderLogin?.charAt(0).toUpperCase() || '?';
});
</script>

<style scoped>
.message {
  display: flex;
  gap: 12px;
  padding: 8px 16px;
  max-width: 100%;
  animation: fadeIn 0.2s ease;
}

.message--own {
  flex-direction: row-reverse;
}

.message__avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: linear-gradient(135deg, #7c3aed, #a855f7);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  color: white;
  font-size: 14px;
  flex-shrink: 0;
  text-transform: uppercase;
}

.message__content {
  max-width: 70%;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.message--own .message__content {
  align-items: flex-end;
}

.message__header {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-secondary, #94a3b8);
  margin-left: 8px;
}

.message__bubble {
  background-color: var(--color-surface, #2d2d3a);
  border-radius: 18px;
  padding: 8px 14px;
  position: relative;
  word-wrap: break-word;
}

.message--own .message__bubble {
  background-color: rgba(124, 58, 237, 0.2);
  border: 1px solid rgba(124, 58, 237, 0.3);
}

.message__text {
  margin: 0;
  font-size: 14px;
  line-height: 1.4;
  color: var(--color-text, #e2e8f0);
}

.message__time {
  font-size: 10px;
  color: var(--color-text-secondary, #94a3b8);
  display: block;
  text-align: right;
  margin-top: 4px;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>