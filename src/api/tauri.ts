import { invoke } from '@tauri-apps/api/core';
import { Api, Chat, Message, PaginateRequest, PaginateResponse, User } from '../types';
import { useTauriCommand } from '../composables/useTauri';

export type ProfileData = {
    username: string;
    peer_id: string;
    device_id: string;
    chats: Chat[];
};

export const tauri: Api = {
    login: (name: string, password: string) => useTauriCommand<User, { name: string; password: string }>('login', { name, password }),
    getUser: async () => invoke<User>('get_user'),
    getChats: () => useTauriCommand<Chat[], void>('get_chats'),
    getChat: (id: string, meta: PaginateRequest) => useTauriCommand<{ messages: Message[]; meta: PaginateResponse }, { chat_id: string; page: number; per_page: number }>('get_chat', { chat_id: id, page: meta.page, per_page: meta.per_page }),
    sendMessage: (id: string, text: string) => useTauriCommand<Message, { chat_id: string; text: string }>('send_message', { chat_id: id, text }),
    connectToPeer: (peerId: string) => useTauriCommand<Chat, { peer_id: string }>('connect_to_peer', { peer_id: peerId }),
    getMyPeerId: async () => invoke<string>('get_local_peer_id'),
    initP2p: async (deviceId: string) => invoke<string>('init_p2p', { device_id: deviceId }),
    loadProfile: async () => invoke<ProfileData>('load'),
};
