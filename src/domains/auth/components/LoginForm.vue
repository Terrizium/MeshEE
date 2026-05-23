<script setup lang="ts">
import { ref } from 'vue';
import BaseButton from '../../main/components/BaseButton.vue';
import BaseUserForm from '../../main/components/BaseUserForm.vue';

const emit = defineEmits<{
    (e: 'submit', {login, password}: {login: string; password: string}): void
}>()

const {
    loading = false,
    withBtn = false,
    btnTitle = 'login'
} = defineProps<{
    loading: boolean;
    withBtn?: boolean;
    btnTitle?: string;
}>()


const usrLogin = ref('');
const pswd = ref('');

</script>
<template>
    <div class="d-flex flex-column jc ac gap-2 h-100">
        <div style="border-bottom: thin 2px">
            <span>Войдите в приложение</span>
        </div>
        <div class="d-flex flex-column jc ac gap-2">
            <BaseUserForm v-model:login="usrLogin" v-model:password="pswd" />
            <BaseInput v-model="usrLogin" />
            <PasswordInput v-model="pswd" />
            <BaseButton v-if="withBtn" :loading="loading" :disabled="!usrLogin || !pswd" @click.stop="emit('submit', {login: usrLogin, password: pswd})">{{ btnTitle }}</BaseButton>
        </div>
    </div>
</template>