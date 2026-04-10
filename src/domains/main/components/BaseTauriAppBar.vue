<template>
    <div data-tauri-drag-region class="titlebar">
      <div class="titlebar-text" style="flex:1; padding-left: 16px;">MeshEE</div>
      <div class="titlebar-buttons">
       <button class="titlebar-button minimize" @click="minimizeWindow">─</button>
       <button class="titlebar-button maximize" @click="toggleMaximizeWindow">□</button>
       <button class="titlebar-button close" @click="closeWindow">✕</button>
      </div>
    </div>
</template>

<script setup>
import { useTauriWindow } from '../../../composables/useTauriWindow';

const {
  closeWindow,
  toggleMaximizeWindow,
  minimizeWindow
} = useTauriWindow();
</script>