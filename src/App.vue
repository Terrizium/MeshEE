<script setup lang="ts">
import { onUnmounted, Ref, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Message } from "./types";
import { useAuth } from "./composables/useAuth";
import LoginForm from "./domains/auth/components/LoginForm.vue";
import BaseTauriAppBar from "./domains/main/components/BaseTauriAppBar.vue";

const { user } = useAuth();

const greetMsg = ref("");
const name = ref("");









const messages: Ref<Message[]> = ref([]);

const isReceiving = ref(false);
let unlisten: UnlistenFn | null = null;

async function startReceiving() {
  if (isReceiving.value) return;

  try {
    // 1. Запускаем периодическую генерацию сообщений в Rust
    await invoke("start_periodic_messages");

    // 2. Подписываемся на события new-message
    unlisten = await listen<Message>("new-message", (event) => {
      console.log("Новое сообщение от Rust:", event.payload);
      messages.value.push(event.payload);
    });

    isReceiving.value = true;
  } catch (error) {
    console.error("Ошибка запуска получения сообщений:", error);
  }
}

// При размонтировании компонента отписываемся от событий
onUnmounted(() => {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
});

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<template>
  <div class="container">
    <BaseTauriAppBar />
    <main class="container pa-2">
      <LoginForm v-if="!user" />
    </main>
  </div>
</template>

<style>



</style>

