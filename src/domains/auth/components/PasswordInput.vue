<script setup lang="ts">
import BaseInput from "../../main/components/BaseInput.vue";
import EyeClosed from "../../main/icons/EyeClosed.vue";
import EyeOutline from "../../main/icons/EyeOutline.vue";
import { Ref, ref } from "vue";
const { 
        modelValue,
        placeholder = 'Enter password...'
    } = defineProps<{
        modelValue: number | string;
        placeholder?: string
    }>()
    const emit = defineEmits<{
        (e: 'update:model-value', v: number | string): void
    }>()

    const showPassword: Ref<boolean> = ref(false);
</script>
<template>
    <BaseInput id="greet-input" :model-value="modelValue" :type="showPassword ? undefined : 'password'" :placeholder="placeholder" @update:model-value="emit('update:model-value', $event)">
        <template #append>
          <EyeOutline v-if="showPassword" style="cursor: pointer" @click.stop="showPassword = !showPassword"/>
          <EyeClosed v-else style="cursor: pointer" @click.stop="showPassword = !showPassword"/>
        </template>
      </BaseInput>
</template>