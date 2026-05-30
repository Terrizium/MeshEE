import { Ref, ref } from "vue";
import { User } from "../types";
import { tauri } from "../api/tauri";
import { useError } from "./useError";

const user: Ref<User | null> = ref(null);

export const useAuth = () => {

    async function login(login: string, password: string) {
        const { data, execute } = tauri.login(login, password);
        try {
            await execute({ name: login, password });
            if (data.value) {
                user.value = data.value;
                // Инициализируем P2P после успешного логина
                await initP2p();
            }
        } catch (e) {
            useError(e);
            throw e;
        }
    }

    async function initP2p() {
        try {
            const profileData = await tauri.loadProfile();
            if (profileData?.device_id) {
                await tauri.initP2p(profileData.device_id);
                console.log('P2P initialized successfully');
            }
        } catch (e) {
            console.error('Failed to initialize P2P:', e);
            useError(e);
        }
    }
    
    async function logout() {
        user.value = null;
    }
    
    return {
        user,
        login,
        logout,
        initP2p
    }
}

