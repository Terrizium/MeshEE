import { Ref, ref } from "vue";
import { User } from "../types";



const user: Ref<User | null> = ref(null);

export const useAuth = () => {

    async function login(login: string, password: string) {
        return Promise.resolve()
    }

    async function logout() {
        return Promise.resolve()
    }
    return {
        user,
        login,
        logout
    }
}