<template>
        <div class="d-flex flex-column jc ac gap-2">
            <BaseInput :model-value="login" placeholder="login" @update:model-value="emit('update:login', $event)"/>
            <PasswordInput :model-value="password" placeholder="password" @update:model-value="emit('update:password', $event)"/>
        </div>
</template>   

<script setup lang="ts">
import BaseInput from './BaseInput.vue';
import PasswordInput from '../../auth/components/PasswordInput.vue';
const { login, password } = defineProps<{
    login: string | null;
    password: string | null;
}>();
const emit = defineEmits<{
    (e: 'update:login' | 'update:password', v: string | null): void
}>();
</script>