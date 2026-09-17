<script setup lang="ts">
import { Skull, ShieldAlert, ShieldCheck, X } from "lucide-vue-next";

const props = withDefaults(
  defineProps<{
    open: boolean;
    title: string;
    description: string;
    targetName?: string;
    targetDetail?: string;
    actionType?: "danger" | "warning" | "success";
    confirmText?: string;
    cancelText?: string;
  }>(),
  {
    actionType: "danger",
    confirmText: "Konfirmasi",
    cancelText: "Batal",
  }
);

const emit = defineEmits<{
  (e: "confirm"): void;
  (e: "close"): void;
}>();
</script>

<template>
  <Transition name="fade">
    <div
      v-if="open"
      class="fixed inset-0 z-[120] flex items-center justify-center p-4"
      @click.self="emit('close')"
    >
      <!-- Backdrop blur -->
      <div class="absolute inset-0 bg-black/70 backdrop-blur-sm"></div>

      <!-- Modal Card -->
      <div
        class="relative z-10 w-full max-w-md rounded-2xl border border-zinc-800 bg-zinc-950/95 p-5 shadow-2xl shadow-black/80 text-zinc-100"
      >
        <div class="flex items-start gap-3.5">
          <!-- Icon variant -->
          <div
            v-if="actionType === 'danger'"
            class="w-10 h-10 rounded-xl bg-rose-950/70 border border-rose-800/60 flex items-center justify-center text-rose-400 shrink-0"
          >
            <Skull class="w-5 h-5" />
          </div>
          <div
            v-else-if="actionType === 'warning'"
            class="w-10 h-10 rounded-xl bg-amber-950/70 border border-amber-800/60 flex items-center justify-center text-amber-400 shrink-0"
          >
            <ShieldAlert class="w-5 h-5" />
          </div>
          <div
            v-else
            class="w-10 h-10 rounded-xl bg-emerald-950/70 border border-emerald-800/60 flex items-center justify-center text-emerald-400 shrink-0"
          >
            <ShieldCheck class="w-5 h-5" />
          </div>

          <div class="flex-1 min-w-0">
            <h3 class="text-sm font-bold text-zinc-100">{{ title }}</h3>
            <p class="text-xs text-zinc-400 mt-1 leading-relaxed">
              {{ description }}
            </p>

            <!-- Target Box -->
            <div
              v-if="targetName"
              class="mt-3 p-2.5 rounded-lg bg-zinc-900/90 border border-zinc-800/80 font-mono text-xs flex flex-col gap-0.5"
            >
              <div class="text-zinc-200 font-semibold truncate">{{ targetName }}</div>
              <div v-if="targetDetail" class="text-[11px] text-zinc-400 truncate">
                {{ targetDetail }}
              </div>
            </div>
          </div>

          <button
            @click="emit('close')"
            class="text-zinc-500 hover:text-white p-1 rounded-lg hover:bg-zinc-900 transition-colors"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- Footer Actions -->
        <div class="mt-5 pt-3 border-t border-zinc-800/80 flex items-center justify-end gap-2.5">
          <button
            @click="emit('close')"
            class="px-3.5 py-1.5 rounded-lg text-xs font-medium text-zinc-400 hover:text-white hover:bg-zinc-900 border border-zinc-800 transition-colors cursor-pointer"
          >
            {{ cancelText }}
          </button>
          <button
            @click="emit('confirm')"
            :class="[
              'px-4 py-1.5 rounded-lg text-xs font-bold transition-all shadow-md cursor-pointer',
              actionType === 'danger'
                ? 'bg-rose-600 hover:bg-rose-500 text-white shadow-rose-600/20'
                : actionType === 'warning'
                ? 'bg-amber-600 hover:bg-amber-500 text-white shadow-amber-600/20'
                : 'bg-emerald-600 hover:bg-emerald-500 text-white shadow-emerald-600/20'
            ]"
          >
            {{ confirmText }}
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
