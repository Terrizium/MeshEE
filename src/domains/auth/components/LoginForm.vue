<script setup lang="ts">
import { ref } from 'vue';
import BaseInput from '../../main/components/BaseInput.vue';
import PasswordInput from './PasswordInput.vue';
import BaseButton from '../../main/components/BaseButton.vue';
import { useAuth } from '../../../composables/useAuth';

const { login } = useAuth();

const usrLogin = ref('');
const pswd = ref('');
const loading = ref(false);

function submit() {
    loading.value = true;
    login(usrLogin.value, pswd.value)
    .finally(() => loading.value = false)
}

</script>
<template>
    <div class="d-flex flex-column jc ac gap-2 h-100">
        <div style="border-bottom: thin 2px">
            <span>Войдите в приложение</span>
        </div>
        <div class="d-flex flex-column jc ac gap-2">
            <BaseInput v-model="usrLogin" placeholder="login" />
            <PasswordInput v-model="pswd" placeholder="password" />
            <BaseButton :loading="loading" :disabled="!usrLogin || !pswd" @click.stop="submit">login</BaseButton>
        </div>
    </div>
</template>