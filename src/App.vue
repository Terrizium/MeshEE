<script setup lang="ts">
import { onUnmounted, Ref, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Message } from "./types";
import BaseButton from "./domains/main/components/BaseButton.vue";
import BaseInput from "./domains/main/components/BaseInput.vue";
import { useTauriWindow } from "./composables/useTauriWindow";

const {
  closeWindow,
  toggleMaximizeWindow,
  minimizeWindow
} = useTauriWindow();

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
    <div data-tauri-drag-region class="titlebar">
      <div class="titlebar-text">MeshEE</div>
      <div class="titlebar-buttons">
        <button class="control-btn" @click="minimizeWindow">_</button>
        <button class="control-btn" @click="toggleMaximizeWindow">□</button>
        <button class="control-btn" @click="closeWindow">×</button>
      </div>
    </div>
    <main class="container pa-2">
    <form class="row" @submit.prevent="greet">
      <BaseInput id="greet-input" v-model="name" placeholder="Enter a name..."/>
      <BaseButton type="submit">Greet</BaseButton>
    </form>
    <p>{{ greetMsg }}</p>
    <div>
      <BaseButton @click="startReceiving" :disabled="isReceiving">
        {{ isReceiving ? "Получение..." : "Начать получение сообщений" }}
      </BaseButton>
      <div v-if="messages.length">
        <h3>Сообщения:</h3>
        <ul>
          <li v-for="msg in messages" :key="msg.id">
            {{ msg.msg }}
          </li>
        </ul>
      </div>
      <p v-else>Нет сообщений</p>
    </div>
  </main>
  </div>

</template>

<style>



</style>

