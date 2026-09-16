<script setup lang="ts">
import { computed } from "vue";
import { cn } from "@/lib/utils";

const props = withDefaults(
  defineProps<{
    variant?: "default" | "secondary" | "destructive" | "outline" | "success" | "warning";
    class?: string;
  }>(),
  {
    variant: "default",
  }
);

const variantClasses = computed(() => {
  switch (props.variant) {
    case "secondary":
      return "border-transparent bg-zinc-800 text-zinc-300 hover:bg-zinc-750";
    case "destructive":
      return "border-transparent bg-rose-500/15 text-rose-400 border border-rose-500/30";
    case "success":
      return "border-transparent bg-emerald-500/15 text-emerald-400 border border-emerald-500/30";
    case "warning":
      return "border-transparent bg-amber-500/15 text-amber-400 border border-amber-500/30";
    case "outline":
      return "text-zinc-300 border-zinc-700";
    default:
      return "border-transparent bg-cyan-500/15 text-cyan-400 border border-cyan-500/30";
  }
});
</script>

<template>
  <div
    :class="
      cn(
        'inline-flex items-center rounded-md border px-2 py-0.5 text-xs font-medium transition-colors focus:outline-none',
        variantClasses,
        props.class
      )
    "
  >
    <slot />
  </div>
</template>
