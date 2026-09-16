<script setup lang="ts">
import { computed } from "vue";
import { cn } from "@/lib/utils";

const props = withDefaults(
  defineProps<{
    variant?: "default" | "secondary" | "destructive" | "outline" | "ghost" | "link";
    size?: "default" | "sm" | "lg" | "icon";
    class?: string;
    disabled?: boolean;
  }>(),
  {
    variant: "default",
    size: "default",
    disabled: false,
  }
);

const variantClasses = computed(() => {
  switch (props.variant) {
    case "secondary":
      return "bg-zinc-800 text-zinc-100 hover:bg-zinc-700 shadow-sm";
    case "destructive":
      return "bg-rose-600 text-white hover:bg-rose-700 shadow-sm";
    case "outline":
      return "border border-zinc-700 bg-zinc-900/50 hover:bg-zinc-800 text-zinc-200";
    case "ghost":
      return "hover:bg-zinc-800 hover:text-zinc-100 text-zinc-400";
    case "link":
      return "text-cyan-400 underline-offset-4 hover:underline";
    default:
      return "bg-cyan-600 text-white hover:bg-cyan-500 shadow";
  }
});

const sizeClasses = computed(() => {
  switch (props.size) {
    case "sm":
      return "h-8 rounded-md px-3 text-xs";
    case "lg":
      return "h-10 rounded-md px-6 text-sm";
    case "icon":
      return "h-8 w-8 rounded-md p-0 flex items-center justify-center";
    default:
      return "h-9 px-4 py-2 text-sm";
  }
});
</script>

<template>
  <button
    :disabled="props.disabled"
    :class="
      cn(
        'inline-flex items-center justify-center gap-2 font-medium transition-all duration-150 active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 cursor-pointer',
        variantClasses,
        sizeClasses,
        props.class
      )
    "
  >
    <slot />
  </button>
</template>
