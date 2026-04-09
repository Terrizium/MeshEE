import { getCurrentWindow } from '@tauri-apps/api/window';

export const useTauriWindow = () => {
     async function closeWindow() {
      const appWindow = getCurrentWindow();
      await appWindow.close();
    }
    async function minimizeWindow() {
      const appWindow = getCurrentWindow();
      await appWindow.minimize();
    }
    async function toggleMaximizeWindow() {
      const appWindow = getCurrentWindow();
      await appWindow.toggleMaximize();
    }

    return {
        closeWindow,
        minimizeWindow,
        toggleMaximizeWindow
    }
}