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
    login: (login: string, password: string) => Promise<User>;
    getChats: () => Promise<Chat[]>;
    getChat: (id: number, meta: PaginateRequest) => Promise<{messages: Message[]; meta: PaginateResponse}>
    sendMessage: (id: number, msg: string) => Promise<Message>
}