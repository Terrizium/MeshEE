import { ref, onUnmounted, type Ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn, type Event } from '@tauri-apps/api/event';

// ----------------------------------------------------------------------
// Хук для вызова команд (invoke) с состоянием
// ----------------------------------------------------------------------
interface UseCommandOptions {
  immediate?: boolean;    // вызвать команду сразу
  onSuccess?: (data: any) => void;
  onError?: (err: any) => void;
}

interface UseCommandReturn<T, P> {
  data: Ref<T | null>;
  error: Ref<any | null>;
  pending: Ref<boolean>;
  execute: (payload?: P) => Promise<T | null>;
}

export function useTauriCommand<T = any, P = any>(
  command: string,
  options: UseCommandOptions = {}
): UseCommandReturn<T, P> {
  const data = ref<T | null>(null) as Ref<T | null>;
  const error = ref<any | null>(null);
  const pending = ref(false);

  const execute = async (payload?: P): Promise<T | null> => {
    pending.value = true;
    error.value = null;
    try {
      const result = await invoke<T>(command, payload);
      data.value = result;
      options.onSuccess?.(result);
      return result;
    } catch (err) {
      error.value = err;
      options.onError?.(err);
      return null;
    } finally {
      pending.value = false;
    }
  };

  if (options.immediate) {
    execute();
  }

  return { data, error, pending, execute };
}

// ----------------------------------------------------------------------
// Хук для подписки на события (listen) с автозачисткой
// ----------------------------------------------------------------------
export function useTauriEvent<T = any>(
  eventName: string,
  handler: (payload: T, event: Event<T>) => void
): { unlisten: UnlistenFn } {
  let unlistenFn: UnlistenFn | null = null;

  const startListening = async () => {
    if (unlistenFn) return;
    unlistenFn = await listen<T>(eventName, (event) => {
      handler(event.payload, event);
    });
  };

  startListening();

  onUnmounted(() => {
    if (unlistenFn) {
      unlistenFn();
      unlistenFn = null;
    }
  });

  return {
    unlisten: () => {
      if (unlistenFn) {
        unlistenFn();
        unlistenFn = null;
      }
    },
  };
}

// ----------------------------------------------------------------------
// Хук для одноразового события (once)
// ----------------------------------------------------------------------
export function useTauriEventOnce<T = any>(
  eventName: string,
  handler: (payload: T, event: Event<T>) => void
) {
  let unlistenFn: UnlistenFn | null = null;

  const startListening = async () => {
    unlistenFn = await listen<T>(eventName, (event) => {
      handler(event.payload, event);
      // автоматически отписываемся после первого срабатывания
      if (unlistenFn) {
        unlistenFn();
        unlistenFn = null;
      }
    });
  };

  startListening();

  onUnmounted(() => {
    if (unlistenFn) {
      unlistenFn();
    }
  });
}

// ----------------------------------------------------------------------
// (Продвинутый) Хук для нескольких событий с общим состоянием
// ----------------------------------------------------------------------
export function useTauriEventGroup(events: Record<string, (payload: any) => void>) {
  const unlisteners: UnlistenFn[] = [];

  const startAll = async () => {
    for (const [eventName, handler] of Object.entries(events)) {
      const unlisten = await listen(eventName, (event) => {
        handler(event.payload);
      });
      unlisteners.push(unlisten);
    }
  };

  startAll();

  onUnmounted(() => {
    unlisteners.forEach((unlisten) => unlisten());
  });
}