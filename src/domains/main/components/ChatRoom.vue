<template>
  <div ref="scroller" class="chat-messages">
    <BaseSpinner ref="loader" v-if="hasMore" />
    <template v-if="reversedMessages.length">
      <template v-for="msg of reversedMessages">
        <MessageBubble
          :message="msg"
          :is-own="msg.user_id === currentUserId"
          :sender-login="getSenderLogin(msg.user_id)"
        />
    </template>
    
    </template>
    <div v-else class="chat-messages__empty">
      Нет сообщений. Напишите первое!
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, useTemplateRef, onMounted } from 'vue';
import MessageBubble from './MessageBuble.vue';
import { useApi } from '../../../composables/useApi';
import { Message } from '../../../types';
import { useIntersectionObserver } from '../../../composables/useIntersectionObserver';
import BaseSpinner from './BaseSpinner.vue';

const props = defineProps<{
  chatId: number;
  currentUserId: number;
  otherUserLogin?: string; // логин собеседника (если нужно)
}>();

const {getChat} = useApi()
const PAGE_SIZE = 20

// Элементы
const scroller = ref(null);
const loader = ref(null)
useIntersectionObserver(loader, loadPrevMessages, ref(true))

// Состояние
const messages = ref<Message[]>([]);
const loadingPrev = ref(false);
const hasMore = ref(true);
const page = ref(1);

// Инверсия для отображения снизу вверх (новые сообщения внизу)
const reversedMessages = computed(() => {
  return [...messages.value].reverse();
});

// Загрузка начальных сообщений (последние)
const loadInitialMessages = async () => {
  page.value = 1;
  const result = await getChat(props.chatId, {
    page: page.value,
    per_page: PAGE_SIZE,
  });
  
  if (result.messages) {
    messages.value = result.messages; 
    
    hasMore.value = result.messages.length !== result.meta.total;
    console.log(hasMore.value);
    
  }
};

// Подгрузка предыдущих сообщений (вверх)
async function loadPrevMessages() {
  console.log('load prev');
  
  if (loadingPrev.value || !hasMore.value) return;
  loadingPrev.value = true;
  const nextPage = page.value + 1;
  const result = await getChat(props.chatId, {
    page: nextPage,
    per_page: PAGE_SIZE,
  });
  if (messages && result.messages.length) {
    // Добавляем в начало массива (более старые сообщения)
    messages.value = [...result.messages, ...messages.value];
    page.value = nextPage;
    hasMore.value = result.messages.length !== result.meta.total;
    await nextTick();
  }
  loadingPrev.value = false;
};

// При изменении chatId перезагружаем
watch(() => props.chatId, () => {
  messages.value = [];
  page.value = 1;
  hasMore.value = true;
  
  loadInitialMessages();
}, { immediate: true });

// Функция для получения логина отправителя (можно передавать из родителя)
const getSenderLogin = (userId: number) => {
  if (userId === props.currentUserId) return '';
  return props.otherUserLogin || 'Собеседник';
};




defineExpose({ 
  scroller,
  loader
  });
</script>

<style scoped>
.chat-messages {
  flex: 1;
  position: relative;
  overflow-y: auto;
  background-color: var(--color-bg, #121217);
}

.scroller {
  height: 100%;
  width: 100%;
}

.chat-messages__empty {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  color: var(--color-text-secondary, #94a3b8);
  font-size: 14px;
}

</style>