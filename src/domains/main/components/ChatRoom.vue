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
          :is-own="msg.user_id === currentUserId"
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
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from 'vue';
import MessageBubble from './MessageBuble.vue';
import { useApi } from '../../../composables/useApi';
import type { Message } from '../../../types';
import { useIntersectionObserver } from '../../../composables/useIntersectionObserver';
import BaseSpinner from './BaseSpinner.vue';

const props = defineProps<{
  chatId: number;
  currentUserId: number;
  otherUserLogin?: string;
}>();

const { getChat } = useApi();
const PAGE_SIZE = 20;

// DOM элементы
const scroller = ref<HTMLElement | null>(null);
const loader = ref<HTMLElement | null>(null);

// Состояние
const messages = ref<Message[]>([]);
const loadingPrev = ref(false);
const hasMore = ref(true);
const page = ref(1);
const isAtBottom = ref(true);          // находится ли пользователь внизу
const showScrollDownButton = ref(false);

// Отслеживаем появление спиннера в зоне видимости (для подгрузки старых)
useIntersectionObserver(loader, loadPrevMessages, ref(true));

// Загрузка начальных сообщений (последние страницы – сервер отдаёт от старых к новым)
const loadInitialMessages = async () => {
  page.value = 1;
  const result = await getChat(props.chatId, {
    page: page.value,
    per_page: PAGE_SIZE,
  });
  if (result?.messages) {
    messages.value = result.messages; // старые → новые
    hasMore.value = messages.value.length < result.meta.total;
    // После загрузки скроллим вниз
    await nextTick();
    scrollToBottom();
  }
};

// Подгрузка предыдущих (старых) сообщений
async function loadPrevMessages() {
  if (loadingPrev.value || !hasMore.value) return;

  loadingPrev.value = true;
  const nextPage = page.value + 1;
  const result = await getChat(props.chatId, {
    page: nextPage,
    per_page: PAGE_SIZE,
  });

  if (result?.messages?.length) {
    // Сохраняем высоту скролла до добавления
    const oldScrollHeight = scroller.value?.scrollHeight || 0;
    const oldScrollTop = scroller.value?.scrollTop || 0;

    // Добавляем старые сообщения в начало
    messages.value = [...result.messages, ...messages.value];
    page.value = nextPage;
    
    hasMore.value = messages.value.length < result.meta.total;
    await nextTick();
    // Корректируем скролл, чтобы видимая область не изменилась
    if (scroller.value) {
      const newScrollHeight = scroller.value.scrollHeight;
      console.log(scroller.value.scrollTop, newScrollHeight, isAtBottom.value, oldScrollHeight);
      
      scroller.value.scrollTop = isAtBottom.value ? newScrollHeight : newScrollHeight - oldScrollHeight + oldScrollTop;
    }
  }
  
  loadingPrev.value = false;
}

// Обработчик скролла – проверяем, находится ли пользователь внизу
const onScroll = () => {
  console.log('scroll', isAtBottom.value);
  
  if (!scroller.value) return;
  const { scrollTop, scrollHeight, clientHeight } = scroller.value;
  const bottom = scrollHeight - scrollTop - clientHeight;
  isAtBottom.value = bottom <= 20; // допуск 20px

  // Показываем кнопку "вниз", если не внизу и есть сообщения
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

// Добавление нового сообщения (вызывается извне при получении через сокет)
const addNewMessage = (newMessage: Message) => {
  messages.value.push(newMessage);
  // Если пользователь был внизу – автоматически прокручиваем к новому сообщению
  if (isAtBottom.value) {
    nextTick(() => scrollToBottom());
  } else {
    // Иначе просто показываем кнопку "вниз" (уже обработается в onScroll)
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
const getSenderLogin = (userId: number) => {
  if (userId === props.currentUserId) return '';
  return props.otherUserLogin || 'Собеседник';
};

// Мок для имитации получения сообщений (будет заменён на реальный сокет)
// Пример регистрации в родительском компоненте:
// onMounted(() => {
//   const unlisten = startListening((msg) => chatMessagesRef.value?.addNewMessage(msg));
//   onUnmounted(unlisten);
// });

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