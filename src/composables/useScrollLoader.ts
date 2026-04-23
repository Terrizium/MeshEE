import { ref, watch, onUnmounted, type Ref } from 'vue';

export interface UseScrollLoaderOptions {
  /** Селектор элемента-спиннера внутри контейнера (по умолчанию '.chat-spinner') */
  spinnerSelector?: string;
  /** Порог срабатывания (0-1), по умолчанию 0.1 */
  threshold?: number;
  /** Отступы root'а (аналогично rootMargin в IntersectionObserver) */
  rootMargin?: string;
  /** Включена ли пагинация (можно временно отключить), по умолчанию true */
  enabled?: Ref<boolean>;
}

/**
 * Композабл для подгрузки данных при появлении спиннера в скроллируемой области.
 * @param containerRef - Ref на скроллируемый контейнер (полученный через useTemplateRef)
 * @param onLoadMore - Функция загрузки (должна возвращать Promise)
 * @param options - Дополнительные настройки
 * @returns Объект с состоянием загрузки и функцией принудительного сброса
 */
export function useScrollLoader(
  containerRef: Ref<HTMLElement | null>,
  onLoadMore: () => Promise<void> | void,
  options: UseScrollLoaderOptions = {}
) {
  const {
    spinnerSelector = '.chat-spinner',
    threshold = 0.1,
    rootMargin = '0px',
    enabled = ref(true),
  } = options;

  const isLoading = ref(false);
  let observer: IntersectionObserver | null = null;
  let currentSpinner: Element | null = null;

  // Функция остановки наблюдения
  const stopObserving = () => {
    if (observer) {
      observer.disconnect();
      observer = null;
    }
  };

  // Функция запуска наблюдения за текущим спиннером
  const startObserving = () => {
    if (!enabled.value) return;
    const container = containerRef.value;
    if (!container) return;

    // Ищем спиннер внутри контейнера
    const spinner = container.querySelector(spinnerSelector);
    if (!spinner) return;

    // Если спиннер уже наблюдаем и он не изменился — ничего не делаем
    if (observer && currentSpinner === spinner) return;

    // Останавливаем старое наблюдение
    stopObserving();

    currentSpinner = spinner;
    observer = new IntersectionObserver(
      (entries) => {
        entries.forEach(async (entry) => {
          // Если элемент виден, не идет загрузка и пагинация включена
          if (entry.isIntersecting && !isLoading.value && enabled.value) {
            isLoading.value = true;
            try {
              await onLoadMore();
            } catch (error) {
              console.error('Ошибка при загрузке:', error);
            } finally {
              isLoading.value = false;
              // После завершения загрузки переподключаем наблюдение
              // (спиннер мог быть заменён или временно скрыт)
              if (enabled.value) {
                restartObserving();
              }
            }
          }
        });
      },
      {
        root: container,
        threshold,
        rootMargin,
      }
    );

    observer.observe(spinner);
  };

  // Перезапуск наблюдения (например, после изменения DOM)
  const restartObserving = () => {
    stopObserving();
    startObserving();
  };

  // Отслеживаем изменения контейнера, его готовности и включённости
  watch(
    [containerRef, () => enabled.value],
    () => {
      if (enabled.value) {
        restartObserving();
      } else {
        stopObserving();
      }
    },
    { immediate: true, flush: 'post' }
  );

  // Дополнительно: если внутри контейнера меняется DOM (например, подгрузились новые сообщения),
  // переподключаем наблюдатель, чтобы найти обновлённый спиннер
  let mutationObserver: MutationObserver | null = null;
  if (typeof MutationObserver !== 'undefined') {
    watch(
      containerRef,
      (container) => {
        if (mutationObserver) {
          mutationObserver.disconnect();
          mutationObserver = null;
        }
        if (container && enabled.value) {
          mutationObserver = new MutationObserver(() => {
            // Если загрузка не активна – обновляем наблюдение
            if (!isLoading.value) {
              restartObserving();
            }
          });
          mutationObserver.observe(container, { childList: true, subtree: true });
        }
      },
      { immediate: true }
    );
  }

  // Очистка при размонтировании
  onUnmounted(() => {
    stopObserving();
    if (mutationObserver) {
      mutationObserver.disconnect();
      mutationObserver = null;
    }
  });

  return {
    isLoading,          // текущее состояние загрузки
    restartObserving,   // принудительно перезапустить наблюдение
  };
}