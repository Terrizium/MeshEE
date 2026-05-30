<template>
    <BaseVirtualScroll :items="chats">
        <template #default="{item}">
            <div>
            <BaseButton @click="emit('select-chat', item)">
                <div class="d-flex gap-2">
                    <span class="flex-grow-1">{{ item.login }}</span>
                    <BaseBadge v-if="item.has_unread" />
                </div>
            </BaseButton>
        </div>
        </template>
    </BaseVirtualScroll>
</template>
<script setup lang="ts">
import { onMounted, Ref, ref } from 'vue';
import { tauri } from '../../../api/tauri';
import { Chat } from '../../../types';
import { useError } from '../../../composables/useError';
import BaseVirtualScroll from '../../main/components/BaseVirtualScroll.vue';
import BaseButton from '../../main/components/BaseButton.vue';
import BaseBadge from '../../main/components/BaseBadge.vue';


    const emit = defineEmits<({
        (e: 'select-chat', v: Chat): void
    })>()
    const chats: Ref<Chat[]> = ref([])


    onMounted(async () => {
        try {
            const { execute } = tauri.getChats();
            const res = await execute();
            if (res) chats.value = res;
        } catch (e) {
            useError(e);
        }
    })
</script>