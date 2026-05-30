import { UseCommandReturn } from "../composables/useTauri";

export type Message = {
    id: string;
    body: string;
    is_read: boolean;
    date: string;
    user_id: number;
}

export type PaginateRequest = {
    page: number;
    per_page: number;
}

export type PaginateResponse = PaginateRequest & {
    total: number;
}

export type User = {
    id: string;
    login: string;
}

export type Chat = {
    id: string;
    login: string;
    has_unread: boolean;
}

export type ProfileData = {
    username: string;
    peer_id: string;
    device_id: string;
    chats: Chat[];
}

export type Api = {
    login: (name: string, password: string) => UseCommandReturn<User, { name: string; password: string }>;
    getUser: () => Promise<User>;
    getChats: () => UseCommandReturn<Chat[], void>;
    getChat: (id: string, meta: PaginateRequest) => UseCommandReturn<{ messages: Message[]; meta: PaginateResponse }, { chat_id: string; page: number; per_page: number }>;
    sendMessage: (id: string, text: string) => UseCommandReturn<Message, { chat_id: string; text: string }>;
    connectToPeer: (peerId: string) => UseCommandReturn<Chat, { peer_id: string }>;
    getMyPeerId: () => Promise<string>;
    initP2p: (deviceId: string) => Promise<string>;
    loadProfile: () => Promise<ProfileData>;
}