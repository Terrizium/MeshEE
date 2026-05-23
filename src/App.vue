<script setup lang="ts">
import { defineAsyncComponent, ref } from "vue";
import { useAuth } from "./composables/useAuth";
import BaseTauriAppBar from "./domains/main/components/BaseTauriAppBar.vue";
import BaseButton from "./domains/main/components/BaseButton.vue";
import LoginForm from "./domains/auth/components/LoginForm.vue";
import BaseModal from "./domains/main/components/BaseModal.vue";
import BaseChatContainer from "./domains/main/containers/BaseChatContainer.vue";
import IconButton from "./domains/main/components/IconButton.vue";

const BaseUserForm = defineAsyncComponent(() => import("./domains/main/components/BaseUserForm.vue"));

const { user, login: authLogin, logout } = useAuth();
const config = ref(null);

const loginLoading = ref(false);
const modalVisible = ref(false);
const newLogin = ref(null);
const newPassword = ref(null);
const inviteLink = ref(null);

const modalContentComponent = ref(null);

const openModal = () => modalVisible.value = true;
function handleLogin({login, password}: {login: string; password: string}): void {
  loginLoading.value = true;
  authLogin(login, password)
  .finally(() => loginLoading.value = false)
}
function handleLogout() {
  if (config.value) {
    config.value.closeMenu()
  }
  logout()
  .then(() => user.value = null)
}

function handleSettings() {
  modalVisible.value = true;
   modalContentComponent.value = BaseUserForm
}

function closeModal() {
  modalVisible.value = false;
  modalContentComponent.value = null;
}
const onMenuClose = () => console.log('меню закрыто');

</script>

<template>
  <div class="container position-relative">
    <BaseTauriAppBar>
      <IconButton v-if="user" ref="config" placement="bottom-end" @close="onMenuClose">
        <!-- кастомная иконка (можно и без слота, тогда будет шестерёнка) -->
        <!-- <template #icon>
          <div class="avatar">👤</div>
        </template> -->
        <!-- содержимое меню -->
        <BaseButton variant="outlined" class="menu-item" @click="openModal">Приглашение</BaseButton>
        <BaseButton variant="outlined" class="menu-item" @click="handleSettings">Настройки</BaseButton>
        <BaseButton variant="outlined" class="menu-item" @click="handleLogout">Выйти</BaseButton>
      </IconButton>
    </BaseTauriAppBar>
    <main class="container">
        <LoginForm withBtn v-if="!user" :loading="loginLoading" @submit="handleLogin"/>
        <BaseChatContainer v-else />
    </main>
    <BaseModal v-model:visible="modalVisible" title="Настройки" :close-on-overlay="true">
      <!-- основное содержимое -->
      <div class="settings-content">
        <component 
        v-if="modalContentComponent" 
        :is="modalContentComponent"
        v-model:login="newLogin"
        v-model:password="newPassword"
        :link="inviteLink"
        />
      </div>
      <template #footer>
        <BaseButton variant="outlined" @click="closeModal">Закрыть</BaseButton>
        <BaseButton variant="secondary">Сохранить</BaseButton>
      </template>
    </BaseModal>
  </div>

</template>

<style>



</style>

