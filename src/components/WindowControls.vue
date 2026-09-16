<script setup lang="ts">
import { ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X, Copy } from "lucide-vue-next";

const isMaximized = ref(false);

async function minimizeWindow() {
  try {
    const appWindow = getCurrentWindow();
    await appWindow.minimize();
  } catch (e) {
    console.error("Window minimize error:", e);
  }
}

async function toggleMaximizeWindow() {
  try {
    const appWindow = getCurrentWindow();
    await appWindow.toggleMaximize();
    isMaximized.value = await appWindow.isMaximized();
  } catch (e) {
    console.error("Window maximize error:", e);
  }
}

async function closeWindow() {
  try {
    const appWindow = getCurrentWindow();
    await appWindow.close();
  } catch (e) {
    console.error("Window close error:", e);
  }
}
</script>

<template>
  <div class="flex items-center gap-1 select-none z-50">
    <!-- Minimize -->
    <button
      @click="minimizeWindow"
      class="w-7 h-7 rounded-md flex items-center justify-center text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white hover:bg-zinc-200/80 dark:hover:bg-zinc-800/80 transition-colors cursor-pointer"
      title="Minimize"
    >
      <Minus class="w-3.5 h-3.5" />
    </button>

    <!-- Maximize / Restore -->
    <button
      @click="toggleMaximizeWindow"
      class="w-7 h-7 rounded-md flex items-center justify-center text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white hover:bg-zinc-200/80 dark:hover:bg-zinc-800/80 transition-colors cursor-pointer"
      title="Maximize / Restore"
    >
      <Copy v-if="isMaximized" class="w-3 h-3" />
      <Square v-else class="w-3 h-3" />
    </button>

    <!-- Close -->
    <button
      @click="closeWindow"
      class="w-7 h-7 rounded-md flex items-center justify-center text-zinc-500 hover:text-white dark:text-zinc-400 dark:hover:text-white hover:bg-rose-600 transition-colors cursor-pointer"
      title="Close"
    >
      <X class="w-3.5 h-3.5" />
    </button>
  </div>
</template>
