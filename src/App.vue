<script setup lang="ts">
import { ref } from "vue";
import { useAuth } from "./composables/useAuth";
import LoginForm from "./domains/auth/components/LoginForm.vue";
import BaseTauriAppBar from "./domains/main/components/BaseTauriAppBar.vue";
import BaseButton from "./domains/main/components/BaseButton.vue";
import BaseModal from "./domains/main/components/BaseModal.vue";
import BaseChatContainer from "./domains/main/containers/BaseChatContainer.vue";
import IconButton from "./domains/main/components/IconButton.vue";

const { user, login: authLogin } = useAuth();
const loginLoading = ref(false)
const modalVisible = ref(false);

const openModal = () => modalVisible.value = true;
function handleLogin({login, password}: {login: string; password: string}): void {
  loginLoading.value = true;
  authLogin(login, password)
  .finally(() => loginLoading.value = false)
}
const onMenuClose = () => console.log('меню закрыто');

</script>

<template>
  <div class="container position-relative">
    <BaseTauriAppBar>
      <IconButton placement="bottom-end" @close="onMenuClose">
        <!-- кастомная иконка (можно и без слота, тогда будет шестерёнка) -->
        <!-- <template #icon>
          <div class="avatar">👤</div>
        </template> -->
        <!-- содержимое меню -->
        <BaseButton variant="outlined" class="menu-item" @click="openModal">Профиль</BaseButton>
        <BaseButton variant="outlined" class="menu-item">Настройки</BaseButton>
        <BaseButton variant="outlined" class="menu-item">Выйти</BaseButton>
      </IconButton>
    </BaseTauriAppBar>
    <main class="container">
        <LoginForm v-if="!user" :loading="loginLoading" @submit="handleLogin"/>
        <BaseChatContainer v-else />
    </main>
    <BaseModal v-model:visible="modalVisible" title="Настройки" :close-on-overlay="true">
      <!-- основное содержимое -->
      <div class="settings-content">
        <p>Здесь будут настройки приложения</p>
      </div>
      <template #footer>
        <BaseButton @click="modalVisible = false">Закрыть</BaseButton>
        <BaseButton variant="primary">Сохранить</BaseButton>
      </template>
    </BaseModal>
  </div>

</template>

<style>



</style>

