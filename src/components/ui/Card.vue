<script setup lang="ts">
import { cn } from "@/lib/utils";

const props = defineProps<{
  class?: string;
}>();
</script>

<template>
  <div
    :class="
      cn(
        'rounded-xl border border-zinc-800/80 bg-zinc-900/40 backdrop-blur-md text-zinc-100 shadow-lg',
        props.class
      )
    "
  >
    <slot />
  </div>
</template>
