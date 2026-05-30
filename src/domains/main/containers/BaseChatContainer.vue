<template>
  <div class="app-layout">
    <div class="main-container">
      <aside 
        class="chats-list"
        :class="{ 'mobile-hidden': isMobile && mobileView !== 'chats' }"
      >
      <BaseVirtualScroll v-if="chatsList.length" :items="chatsList">
        <template #default="{item}">
          <BaseChatList :active="selectedChat?.id === item.id" :chat="item" @click="selectChat(item)" />
        </template>        
      </BaseVirtualScroll>
        
      </aside>

      <main 
        class="room-content"
        :class="{ 'mobile-hidden': isMobile && mobileView !== 'room' }"
      >
      <BaseAppBar v-if="selectedChat">
        <template #start>
           <BaseButton small v-if="isMobile && mobileView === 'room'" @click="goBackToChats">
            <span style="height: 100%; display: flex; align-items: center; padding-bottom: 4px;">←</span>
           </BaseButton>
        </template>
        <template #title>
          <span>{{ selectedChat.login }}</span>
        </template>
      </BaseAppBar>
       <ChatRoom
       v-if="selectedChat && user"
      ref="chatMessagesRef"
      :chat-id="selectedChat.id"
      :current-user-id="user.id"
      :other-user-login="selectedChat.login"
    />
    
      </main>
    </div>

  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, Ref, ref } from 'vue';
import { useBreakpoints } from '../../../composables/useBreakpoints';
import { Chat, Message } from '../../../types';
import { tauri } from '../../../api/tauri';
import BaseChatList from '../components/BaseChatList.vue'
import BaseVirtualScroll from '../components/BaseVirtualScroll.vue';
import BaseAppBar from '../components/BaseAppBar.vue';
import BaseButton from '../components/BaseButton.vue';
import ChatRoom from '../components/ChatRoom.vue';
import { useAuth } from '../../../composables/useAuth';
import { useError } from '../../../composables/useError';
import { listen } from '@tauri-apps/api/event';


const {
    isMobile
} = useBreakpoints();

const { user } = useAuth();

const mobileView: Ref<'room' | 'chats'> = ref('chats');
const selectedChat: Ref<Chat | null> = ref(null);
const chatsList: Ref<Chat[] | []> = ref([]);
const chatMessagesRef = ref<InstanceType<typeof ChatRoom> | null>(null);

function goBackToChats() {
    mobileView.value = 'chats';
}

function selectChat(chat: Chat) {
    selectedChat.value = chat;
    mobileView.value = 'room';
}

async function loadChats() {
  try {
    const { data, execute } = tauri.getChats();
    await execute();
    if (data.value) {
      chatsList.value = data.value;
    }
  } catch (e) {
    useError(e);
  }
}

onMounted(async () => {
  await loadChats();

  // Подписка на новый чат
  const unlistenNewChat = await listen<Chat>('new-chat', (event) => {
    const newChat = event.payload;
    // Добавляем чат в начало списка если его нет
    const existingIndex = chatsList.value.findIndex(c => c.id === newChat.id);
    if (existingIndex === -1) {
      chatsList.value.unshift(newChat);
    } else {
      // Если чат уже есть, перемещаем его в начало
      const chat = chatsList.value.splice(existingIndex, 1)[0];
      chatsList.value.unshift(chat);
    }
  });

  // Подписка на новое сообщение
  const unlistenNewMessage = await listen<Message>('new-message', (event) => {
    const msg = event.payload;
    
    // Если текущий чат открыт - добавляем сообщение
    if (selectedChat.value) {
      // user_id === 1 означает сообщение от собеседника
      // user_id === 0 означает собственное сообщение
      if (msg.user_id === 1 || msg.user_id === 0) {
        chatMessagesRef.value?.addNewMessage(msg);
      }
    }
    
    // Обновляем флаг unread и перемещаем чат в начало
    const chatIndex = chatsList.value.findIndex(c => c.id === msg.user_id.toString());
    if (chatIndex !== -1) {
      const chat = chatsList.value.splice(chatIndex, 1)[0];
      chat.has_unread = true;
      chatsList.value.unshift(chat);
    }
  });

  onUnmounted(() => {
    unlistenNewChat();
    unlistenNewMessage();
  });
});

</script>

<style scoped>
/* Десктопная раскладка */
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.main-container {
  flex: 1;
  display: flex;
  overflow: hidden;
  height: 100%;
}

.chats-list {
  width: 300px;
  border-right: 1px solid #ccc;
  overflow-y: auto;
}

.room-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  overflow: hidden;
}

/* Мобильная раскладка */
@media (max-width: 768px) {
  .main-container {
    position: relative;
  }

  .chats-list,
  .room-content {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    transition: transform 0.3s ease;
  }

  /* Скрываем неактивный блок */
  .mobile-hidden {
    transform: translateX(-100%);
    pointer-events: none;
  }

  /* Альтернатива – display: none, но тогда нет анимации */
  /*
  .mobile-hidden {
    display: none;
  }
  */
}

/* Планшет – можно показывать оба блока, но с изменёнными пропорциями */
@media (min-width: 768px) and (max-width: 1023px) {
  .chats-list {
    width: 250px;
  }
}

/* Футер только для мобилок и планшетов */
.app-footer {
  padding: 10px;
  background: #f5f5f5;
  text-align: center;
}
</style>