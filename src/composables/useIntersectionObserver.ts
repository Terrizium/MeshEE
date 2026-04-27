// composables/useScrollLoader.ts
import { ref, watch, onUnmounted, type Ref } from 'vue';

export function useIntersectionObserver(
  loaderRef: Ref<{ $el: HTMLElement } | HTMLElement | null>,
  onLoadMore: () => Promise<void> | void,
  isEnabled: Ref<boolean>
) {
  const isLoading = ref(false);
  let observer: IntersectionObserver | null = null;

  const getTarget = () => {
    const val = loaderRef.value;
    if (!val) return null;
    // Если передали компонент с $el — берём его, иначе считаем, что это DOM-элемент
    return '$el' in val ? val.$el : val;
  };

  const stopObserving = () => {
    if (observer) {
      observer.disconnect();
      observer = null;
    }
  };

  const startObserving = () => {
    const target = getTarget();
    if (!target || !isEnabled.value) return;

    stopObserving();
    observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && !isLoading.value && isEnabled.value) {
          isLoading.value = true;
          Promise.resolve(onLoadMore())
            .finally(() => {
              isLoading.value = false;
              // После загрузки переподключаем наблюдение (спиннер мог временно исчезнуть)
              startObserving();
            });
        }
      },
      { threshold: 0.1 }
    );
    observer.observe(target);
  };

  // Следим за изменением loaderRef, enabled или самого элемента (через MutationObserver не нужно)
  watch([loaderRef, isEnabled], () => {
    startObserving();
  }, { immediate: true, flush: 'post' });

  onUnmounted(stopObserving);

  return { isLoading };
}