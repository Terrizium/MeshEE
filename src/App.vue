<script setup lang="ts">
import { defineAsyncComponent, ref, onMounted, onUnmounted } from "vue";
import { useAuth } from "./composables/useAuth";
import { listen } from '@tauri-apps/api/event';
import BaseTauriAppBar from "./domains/main/components/BaseTauriAppBar.vue";
import BaseButton from "./domains/main/components/BaseButton.vue";
import LoginForm from "./domains/auth/components/LoginForm.vue";
import BaseModal from "./domains/main/components/BaseModal.vue";
import BaseChatContainer from "./domains/main/containers/BaseChatContainer.vue";
import IconButton from "./domains/main/components/IconButton.vue";
import type { Chat, Message } from "./types";

const BaseUserForm = defineAsyncComponent(() => import("./domains/main/components/BaseUserForm.vue"));
const InviteModal = defineAsyncComponent(() => import("./domains/auth/components/InviteModal.vue"));

const { user, login: authLogin, logout, initP2p } = useAuth();
const config = ref(null);

const loginLoading = ref(false);
const modalVisible = ref(false);
const modalType = ref<'invite' | 'settings' | null>(null);
const newLogin = ref(null);
const newPassword = ref(null);

const openInviteModal = () => {
  modalType.value = 'invite';
  modalVisible.value = true;
};

const openSettingsModal = () => {
  modalType.value = 'settings';
  modalVisible.value = true;
};

function handleLogin({login, password}: {login: string; password: string}): void {
  loginLoading.value = true;
  authLogin(login, password)
  .finally(() => loginLoading.value = false);
}

function handleLogout() {
  if (config.value) {
    (config.value as any).closeMenu();
  }
  logout()
  .then(() => user.value = null);
}

function closeModal() {
  modalVisible.value = false;
  modalType.value = null;
}

const onMenuClose = () => console.log('меню закрыто');

// Подписка на события
onMounted(async () => {
  // Если пользователь уже авторизован, инициализируем P2P
  if (user.value) {
    await initP2p();
  }

  // Подписка на новое сообщение
  const unlistenNewMessage = await listen<Message>('new-message', (event) => {
    console.log('Новое сообщение получено:', event.payload);
    // Эмитим кастомное событие для обновления чатов
    window.dispatchEvent(new CustomEvent('new-message', { detail: event.payload }));
  });

  // Подписка на новый чат
  const unlistenNewChat = await listen<Chat>('new-chat', (event) => {
    console.log('Новый чат получен:', event.payload);
    // Эмитим кастомное событие для обновления списка чатов
    window.dispatchEvent(new CustomEvent('new-chat', { detail: event.payload }));
  });

  onUnmounted(() => {
    unlistenNewMessage();
    unlistenNewChat();
  });
});
</script>

<template>
  <div class="container position-relative">
    <BaseTauriAppBar>
      <IconButton v-if="user" ref="config" placement="bottom-end" @close="onMenuClose">
        <!-- содержимое меню -->
        <BaseButton variant="outlined" class="menu-item" @click="openInviteModal">Приглашение</BaseButton>
        <BaseButton variant="outlined" class="menu-item" @click="openSettingsModal">Настройки</BaseButton>
        <BaseButton variant="outlined" class="menu-item" @click="handleLogout">Выйти</BaseButton>
      </IconButton>
    </BaseTauriAppBar>
    <main class="container">
        <LoginForm withBtn v-if="!user" :loading="loginLoading" @submit="handleLogin"/>
        <BaseChatContainer v-else />
    </main>
    <BaseModal v-model:visible="modalVisible" :title="modalType === 'invite' ? 'Приглашение' : 'Настройки'" :close-on-overlay="true">
      <div class="settings-content">
        <InviteModal v-if="modalType === 'invite'" v-model:visible="modalVisible" />
        <component 
        v-else-if="modalType === 'settings'"
        :is="BaseUserForm"
        v-model:login="newLogin"
        v-model:password="newPassword"
        />
      </div>
      <template #footer>
        <BaseButton variant="outlined" @click="closeModal">Закрыть</BaseButton>
        <BaseButton v-if="modalType === 'settings'" variant="secondary">Сохранить</BaseButton>
      </template>
    </BaseModal>
  </div>
</template>

<style>



</style>

