<template>
  <div class="chat-container">
    <div ref="scroller" class="chat-messages" @scroll="onScroll">
      <!-- Индикатор загрузки старых сообщений (вверху) -->
      <div v-if="hasMore" class="chat-messages__loader-wrapper">
        <BaseSpinner ref="loader" />
      </div>

      <!-- Список сообщений (старые сверху, новые снизу) -->
      <template v-for="msg in messages" :key="msg.id">
        <MessageBubble
          :message="msg"
          :is-own="msg.user_id === 0"
          :sender-login="getSenderLogin(msg.user_id)"
        />
      </template>

      <div v-if="!messages.length" class="chat-messages__empty">
        Нет сообщений. Напишите первое!
      </div>
    </div>

    <!-- Кнопка "вниз" (показывается, если не внизу) -->
    <Transition name="fade">
      <button v-if="showScrollDownButton" class="scroll-down-btn" @click="scrollToBottom">
        ↓
      </button>
    </Transition>

    <!-- Поле ввода сообщения -->
    <div class="chat-input">
      <BaseInput
        v-model="newMessage"
        placeholder="Введите сообщение..."
        @keyup.enter="sendMessage"
      />
      <BaseButton
        @click="sendMessage"
        :loading="sending"
        :disabled="!newMessage.trim()"
        class="send-btn"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
          <path
            d="M22 2L11 13"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            d="M22 2L15 22L11 13L2 9L22 2Z"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </BaseButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from 'vue';
import MessageBubble from './MessageBuble.vue';
import { tauri } from '../../../api/tauri';
import type { Message, Chat } from '../../../types';
import { useIntersectionObserver } from '../../../composables/useIntersectionObserver';
import BaseSpinner from './BaseSpinner.vue';
import BaseInput from './BaseInput.vue';
import BaseButton from './BaseButton.vue';
import { listen } from '@tauri-apps/api/event';

const props = defineProps<{
  chatId: string;
  currentUserId: string;
  otherUserLogin?: string;
}>();

const PAGE_SIZE = 20;

// DOM элементы
const scroller = ref<HTMLElement | null>(null);
const loader = ref<HTMLElement | null>(null);

// Состояние
const messages = ref<Message[]>([]);
const loadingPrev = ref(false);
const hasMore = ref(true);
const page = ref(1);
const isAtBottom = ref(true);
const showScrollDownButton = ref(false);
const newMessage = ref('');
const sending = ref(false);

// Отслеживаем появление спиннера в зоне видимости (для подгрузки старых)
useIntersectionObserver(loader, loadPrevMessages, ref(true));

// Загрузка начальных сообщений (последние страницы – сервер отдаёт от старых к новым)
const loadInitialMessages = async () => {
  page.value = 1;
  const { data, execute } = tauri.getChat(props.chatId, {
    page: page.value,
    per_page: PAGE_SIZE,
  });
  await execute();
  if (data.value?.messages) {
    messages.value = data.value.messages;
    hasMore.value = messages.value.length < data.value.meta.total;
    await nextTick();
    scrollToBottom();
  }
};

// Подгрузка предыдущих (старых) сообщений
async function loadPrevMessages() {
  if (loadingPrev.value || !hasMore.value) return;

  loadingPrev.value = true;
  const nextPage = page.value + 1;
  const { data, execute } = tauri.getChat(props.chatId, {
    page: nextPage,
    per_page: PAGE_SIZE,
  });
  await execute();

  if (data.value?.messages?.length) {
    const oldScrollHeight = scroller.value?.scrollHeight || 0;
    const oldScrollTop = scroller.value?.scrollTop || 0;

    messages.value = [...data.value.messages, ...messages.value];
    page.value = nextPage;
    
    hasMore.value = messages.value.length < data.value.meta.total;
    await nextTick();
    if (scroller.value) {
      const newScrollHeight = scroller.value.scrollHeight;
      scroller.value.scrollTop = isAtBottom.value ? newScrollHeight : newScrollHeight - oldScrollHeight + oldScrollTop;
    }
  }
  
  loadingPrev.value = false;
}

// Отправка сообщения
async function sendMessage() {
  const text = newMessage.value.trim();
  if (!text) return;

  sending.value = true;
  try {
    const { data, execute } = tauri.sendMessage(props.chatId, text);
    await execute({ chat_id: props.chatId, text });
    if (data.value) {
      // Добавляем собственное сообщение сразу
      addNewMessage(data.value);
      newMessage.value = '';
    }
  } catch (e) {
    console.error('Failed to send message:', e);
  } finally {
    sending.value = false;
  }
}

// Обработчик скролла
const onScroll = () => {
  if (!scroller.value) return;
  const { scrollTop, scrollHeight, clientHeight } = scroller.value;
  const bottom = scrollHeight - scrollTop - clientHeight;
  isAtBottom.value = bottom <= 20;

  showScrollDownButton.value = messages.value.length > 0 && !isAtBottom.value;
};

// Прокрутка в самый низ
const scrollToBottom = () => {
  if (!scroller.value) return;
  scroller.value.style.scrollBehavior = 'smooth';
  scroller.value.scrollTop = scroller.value.scrollHeight;
  scroller.value.style.scrollBehavior = 'auto';
  isAtBottom.value = true;
  showScrollDownButton.value = false;
};

// Добавление нового сообщения
const addNewMessage = (newMsg: Message) => {
  messages.value.push(newMsg);
  if (isAtBottom.value) {
    nextTick(() => scrollToBottom());
  } else {
    showScrollDownButton.value = true;
  }
};

// Сброс чата при смене ID
watch(
  () => props.chatId,
  () => {
    messages.value = [];
    page.value = 1;
    hasMore.value = true;
    loadInitialMessages();
  },
  { immediate: true }
);

// Функция получения логина отправителя
// user_id === 0 - собственное сообщение, user_id === 1 - сообщение собеседника
const getSenderLogin = (userId: number) => {
  if (userId === 0) return '';
  return props.otherUserLogin || 'Собеседник';
};

// Экспонируем метод для добавления сообщений извне
defineExpose({
  addNewMessage,
  scrollToBottom,
});
</script>

<style scoped>
.chat-container {
  flex: 1;
  position: relative;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding-bottom: 12px;
}

.chat-messages {
  flex: 1;
  overflow: auto !important;
  background-color: var(--color-bg, #121217);
  padding: 16px 0;
}

.chat-messages__loader-wrapper {
  display: flex;
  justify-content: center;
  padding: 12px 0;
}

.chat-messages__empty {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 200px;
  color: var(--color-text-secondary, #94a3b8);
  font-size: 14px;
  text-align: center;
}

.scroll-down-btn {
  position: absolute;
  bottom: 60px;
  right: 20px;
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background-color: var(--color-primary, #7c3aed);
  border: none;
  color: white;
  font-size: 24px;
  cursor: pointer;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
  transition: transform 0.2s, background 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
}

.scroll-down-btn:hover {
  background-color: var(--color-primary-hover, #8b5cf6);
  transform: scale(1.05);
}

.chat-input {
  display: flex;
  gap: 0.5rem;
  padding: 12px 16px;
  background-color: var(--color-bg, #121217);
  border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.chat-input :deep(.gradient-input-wrapper) {
  flex: 1;
}

.send-btn {
  flex-shrink: 0;
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.send-btn svg {
  width: 20px;
  height: 20px;
}

/* анимация появления/исчезновения кнопки */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>