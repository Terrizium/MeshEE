<template>
  <div
    class="gradient-input-wrapper"
    :class="{
      'is-focused': isFocused,
      'is-disabled': disabled,
      'has-error': error,
    }"
  >
    <div v-if="$slots.prepend" class="input-prepend">
      <slot name="prepend" />
    </div>

    <input
      ref="inputRef"
      :value="modelValue"
      :type="type"
      :placeholder="placeholder"
      :disabled="disabled"
      :readonly="readonly"
      :required="required"
      :name="name"
      :autocomplete="autocomplete"
      class="gradient-input"
      @input="onInput"
      @focus="isFocused = true"
      @blur="isFocused = false"
      v-bind="$attrs"
    />

    <div v-if="$slots.append" class="input-append">
      <slot name="append" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

interface Props {
  modelValue: string | number | null;
  type?: string;
  placeholder?: string;
  disabled?: boolean;
  readonly?: boolean;
  required?: boolean;
  name?: string;
  autocomplete?: string;
  error?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  type: 'text',
  placeholder: '',
  disabled: false,
  readonly: false,
  required: false,
  name: undefined,
  autocomplete: 'off',
  error: false,
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: string | number): void;
  (e: 'input', event: Event): void;
  (e: 'focus', event: FocusEvent): void;
  (e: 'blur', event: FocusEvent): void;
}>();

const inputRef = ref<HTMLInputElement | null>(null);
const isFocused = ref(false);

const onInput = (event: Event) => {
  const target = event.target as HTMLInputElement;
  let value: string | number = target.value;
  if (props.type === 'number') {
    value = value === '' ? '' : Number(value);
  }
  emit('update:modelValue', value);
  emit('input', event);
};

defineExpose({
  focus: () => inputRef.value?.focus(),
  blur: () => inputRef.value?.blur(),
});
</script>

<style scoped>
.gradient-input-wrapper {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  background: #121217; 
  border: 1px solid #2a2a35;
  border-radius: 1rem;
  padding: 0 0.75rem;
  transition: all 0.2s ease;
  width: 100%;
  box-sizing: border-box;
  backdrop-filter: blur(2px); 
}

.gradient-input-wrapper.is-focused {
  border-color: transparent;
  background: #0f0f14;
  box-shadow: 0 0 0 2px rgba(117, 121, 255, 0.5), 0 0 0 4px rgba(178, 36, 239, 0.2);
}

.gradient-input-wrapper.has-error {
  border-color: #ff5c5c;
  box-shadow: 0 0 0 1px #ff5c5c80;
}
.gradient-input-wrapper.has-error.is-focused {
  box-shadow: 0 0 0 2px #ff5c5c80, 0 0 0 4px rgba(255, 92, 92, 0.2);
}

.gradient-input-wrapper.is-disabled {
  opacity: 0.6;
  background: #0a0a0e;
  cursor: not-allowed;
}

.gradient-input {
  flex: 1;
  background: transparent;
  border: none;
  padding: 0.75rem 0;
  font-size: 0.95rem;
  font-family: inherit;
  color: #f0f0f0;
  outline: none;
  width: 100%;
  min-width: 0; 
}

.gradient-input::placeholder {
  color: #5a5a6e;
  font-weight: 400;
}

.gradient-input:disabled {
  cursor: not-allowed;
}

.input-prepend,
.input-append {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #8a8aa0;
  font-size: 1.1rem;
  transition: color 0.2s;
  flex-shrink: 0;
}

.gradient-input-wrapper.is-focused .input-prepend,
.gradient-input-wrapper.is-focused .input-append {
  color: #a78bfa;
}

.input-prepend :deep(svg),
.input-append :deep(svg) {
  width: 1.2rem;
  height: 1.2rem;
  fill: currentColor;
}
</style>