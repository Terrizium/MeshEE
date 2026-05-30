import { UseCommandReturn } from "../composables/useTauri";

export type Message = {
    id: number;
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
    id: number;
    login: string;
}

export type Chat = {
        id: number;
        login: string;
        has_unread: boolean;
    };

export type Api = {
    getUser: () => Promise<User>;
    login: () => UseCommandReturn<'login', null>;
    getChats: () => UseCommandReturn<'get_chats',Chat[]>;
    getChat: (id: number, meta: PaginateRequest) => UseCommandReturn<'get_chat',{messages: Message[]; meta: PaginateResponse}>
    sendMessage: (id: number, text: string) => UseCommandReturn<'send_message',Message>;
    getInvite: () => UseCommandReturn<'get_local_peer_id',string>;
}