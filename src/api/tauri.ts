import { Api, Chat, Message, Paginate } from "../types";
import { User } from "../types";

export const tauri: Api = {
    login: async (login, password) => getApi().login(login, password),
    getUser: async () => getApi().getUser(),
    getChats: async () => getApi().getChats(),
    getChat: async (id, meta) => getApi().getChat(id, meta),
    sendMessage: async (id, msg) => getApi().sendMessage(id, msg)
}

function getApi(): Api {
  return mockApi;
}

const mockApi: Api = {
    login: async (): Promise<User> => ({
            id: 1,
            login: 'Stiven'
    }),
    getUser: async (): Promise<User> => ({
            id: 1,
            login: 'Stiven'
    }),
    getChats: async (): Promise<Chat[]> => ([
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
    getChat: async (): Promise<{meta: Paginate; messages: Message[]}> => ({
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
