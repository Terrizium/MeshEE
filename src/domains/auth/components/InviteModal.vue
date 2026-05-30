<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { tauri } from '../../../api/tauri';
import BaseInput from '../../main/components/BaseInput.vue';
import BaseButton from '../../main/components/BaseButton.vue';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
}>();

const myPeerId = ref<string>('');
const myInviteLink = ref<string>('');
const friendPeerId = ref<string>('');
const loading = ref(false);
const copySuccess = ref(false);

async function loadMyPeerId() {
  try {
    myPeerId.value = await tauri.getMyPeerId();
    myInviteLink.value = myPeerId.value;
    console.log(myPeerId)
    friendPeerId.value = '';
  } catch (e) {
    console.error('Failed to load peer ID:', e);
  }
}

async function copyToClipboard() {
  try {
    await navigator.clipboard.writeText(myInviteLink.value);
    copySuccess.value = true;
    setTimeout(() => {
      copySuccess.value = false;
    }, 2000);
  } catch (e) {
    console.error('Failed to copy:', e);
  }
}

async function connectToPeer() {
  if (!friendPeerId.value.trim()) return;
  
  loading.value = true;
  try {
    const { execute } = tauri.connectToPeer(friendPeerId.value.trim());
    await execute({ peer_id: friendPeerId.value.trim() });
    friendPeerId.value = '';
    emit('update:visible', false);
  } catch (e) {
    console.error('Failed to connect to peer:', e);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  console.log('Mounted and load peer');
    loadMyPeerId();
});

watch(() => props.visible, (newVal) => {
  if (newVal) {
    loadMyPeerId();
    copySuccess.value = false;
  }
});
</script>

<template>
  <div class="invite-modal">
    <div class="invite-modal__header">
      <h3>Приглашение в чат</h3>
    </div>
    
    <div class="invite-modal__content">
      <!-- Поле с моей ссылкой для копирования -->
      <div class="invite-modal__section">
        <label class="invite-modal__label">Ваша ссылка для приглашения:</label>
        <div class="invite-input-wrapper">
          <BaseInput
            v-model="myInviteLink"
            readonly
            placeholder="Загрузка..."
          />
          <BaseButton
            @click="copyToClipboard"
            :disabled="!myInviteLink"
            class="copy-btn"
          >
            {{ copySuccess ? 'Скопировано!' : 'Копировать' }}
          </BaseButton>
        </div>
      </div>

      <!-- Поле для ввода ссылки друга -->
      <div class="invite-modal__section">
        <label class="invite-modal__label">Введите ссылку друга:</label>
        <div class="invite-input-wrapper">
          <BaseInput
            v-model="friendPeerId"
            placeholder="Вставьте ссылку для подключения"
          />
          <BaseButton
            @click="connectToPeer"
            :loading="loading"
            :disabled="!friendPeerId.trim()"
            class="connect-btn"
          >
            Подключиться
          </BaseButton>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.invite-modal {
  padding: 1rem;
}

.invite-modal__header {
  margin-bottom: 1.5rem;
}

.invite-modal__header h3 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: #e2e8f0;
}

.invite-modal__content {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.invite-modal__section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.invite-modal__label {
  font-size: 0.875rem;
  font-weight: 500;
  color: #94a3b8;
}

.invite-input-wrapper {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.invite-input-wrapper :deep(.gradient-input-wrapper) {
  flex: 1;
}

.copy-btn,
.connect-btn {
  flex-shrink: 0;
  white-space: nowrap;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
