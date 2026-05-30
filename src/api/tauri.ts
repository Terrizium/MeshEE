import { ref } from "vue";
import { useTauriCommand } from "../composables/useTauri";
import { Api, Chat, Message, PaginateRequest, PaginateResponse } from "../types";
import { User } from "../types";

export const tauri: Api = {
    login: async () => getApi().login(),
    getUser: async () => getApi().getUser(),
    getChats: async () => getApi().getChats(),
    getChat: async (id: number, meta: PaginateRequest) => getApi().getChat(id, meta),
    sendMessage: async (id: number, text: string) => getApi().sendMessage(id, text)
    //func connect_to_peer(peerId: string): Promise<Chat> 
    //func get_my_peer_id: string
    //emit new-message
    //emit new-chat
    ,
    getInvite: async () => getApi().getInvite()
}

function getApi(): Api {
  return {...mockApi, ...chatApi};
}

const chatApi: Api = {
    login: async () => useTauriCommand('login', null),
    getChats: async() => useTauriCommand('get_chats', null),
    getChat: async() => useTauriCommand('get_chat')
}

const mockApi: Api = {
    login: () => ({
        data: ref<User>({
            id: 1,
            login: 'Stiven'
        }),
        error: ref(null),
        pending: ref(false),
        execute: async ({login, password}) => this?.data || null
}),
    getUser: async (): Promise<User> => ({
            id: 1,
            login: 'Stiven'
    }),
    getChats: async () => ({
        data: ref([
        {
            id: 1,
            login: 'StaticRange',
            has_unread: true,
        },
        {
            id: 2,
            login: 'StaticRange',
            has_unread: true,
        },
        {
            id: 3,
            login: 'StaticRange',
            has_unread: false,
        },
    ]),
      error: ref(null),
      pending: ref(false),
      execute: () => this?.data || null        
    }),
    getInvite: async(): Promise<{link: string}> => new Promise.resolve({link: 'Here is your invite link'}),
    getChat: async (): Promise<{meta: PaginateResponse; messages: Message[]}> => ({
        meta: {
        page: counter++,
        per_page: 20,
        total: 25
    },messages: [
        {
            id: 1,
            body: 'Hi there',
            is_read: false,
            date: '07:02:2023 20:30',
            user_id: 2
        },
        {
            id: 2,
            body: 'STRrrrrrr',
            is_read: false,
            date: '07:02:2023 20:30',
            user_id: 2
        },
        {
            id: 3,
            body: 'Hey hey hey you are',
            is_read: false,
            date: '07:02:2023 20:30',
            user_id: 2
        },
    ]}),
    sendMessage: async () => ({
            id: 4,
            body: '-|-_-|-/',
            is_read: false,
            date: '07:02:2023 20:30',
            user_id: 2
        })
}

let counter = 1;
