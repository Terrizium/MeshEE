import { Ref, ref } from "vue";
import { User } from "../types";
import { useApi } from "./useApi";
import { useError } from "./useError";


const user: Ref<User | null> = ref(null);

export const useAuth = () => {

    async function login(login: string, password: string) {
        const { login: loginApi } = useApi();
        try {
            const {data, error, execute} = await loginApi()
            await execute({name: login, password});
            user.value = data.value;
        } catch (e) {
            useError(e)
        }

        
    }

    async function logout() {
        user.value = null;
    }
    return {
        user,
        login,
        logout
    }
}

