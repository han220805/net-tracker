<script setup lang="ts">
import { DownloadCloud, RefreshCw, AlertCircle, Sparkles, ExternalLink } from "lucide-vue-next";
import { openUrl } from "@tauri-apps/plugin-opener";

const props = defineProps<{
  open: boolean;
  currentVersion: string;
  newVersion: string;
  releaseNotes?: string;
  isDownloading: boolean;
  progressPercent: number;
  downloadedBytes: number;
  totalBytes: number;
  errorMessage?: string | null;
}>();

const emit = defineEmits<{
  (e: "start-update"): void;
}>();

function formatSize(bytes: number): string {
  if (!bytes || bytes === 0) return "0 MB";
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function openGithubRelease() {
  openUrl("https://github.com/han220805/net-tracker/releases/latest").catch(() => {
    window.open("https://github.com/han220805/net-tracker/releases/latest", "_blank");
  });
}
</script>

<template>
  <Transition name="fade">
    <div
      v-if="open"
      class="fixed inset-0 z-[200] flex items-center justify-center p-4 select-none"
    >
      <!-- Deep backdrop blur & dark gradient overlay (locks UI) -->
      <div class="absolute inset-0 bg-[#06080e]/85 backdrop-blur-md"></div>

      <!-- Modal Card -->
      <div
        class="relative z-10 w-full max-w-lg rounded-2xl border border-cyan-500/40 bg-zinc-950/95 p-6 shadow-2xl shadow-cyan-950/40 text-zinc-100 overflow-hidden"
      >
        <!-- Glow accent -->
        <div class="absolute -top-16 -right-16 w-36 h-36 bg-cyan-500/15 rounded-full blur-3xl pointer-events-none"></div>

        <!-- Header -->
        <div class="flex items-start gap-4">
          <div class="w-12 h-12 rounded-2xl bg-gradient-to-tr from-cyan-600 to-blue-600 flex items-center justify-center text-white shadow-lg shadow-cyan-500/25 shrink-0">
            <Sparkles class="w-6 h-6" />
          </div>

          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <h2 class="text-base font-bold text-white tracking-wide">Pembaruan Versi Wajib</h2>
              <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-cyan-500/20 text-cyan-300 border border-cyan-500/40">
                v{{ newVersion }}
              </span>
            </div>
            <p class="text-xs text-zinc-400 mt-1 leading-relaxed">
              Versi terbaru Net Tracker telah tersedia. Untuk menjaga stabilitas, keamanan, dan kompatibilitas firewall, silakan perbarui aplikasi sekarang.
            </p>
          </div>
        </div>

        <!-- Version Comparison Badge -->
        <div class="mt-4 p-3 rounded-xl bg-zinc-900/90 border border-zinc-800 flex items-center justify-between text-xs">
          <div class="flex items-center gap-2">
            <span class="text-zinc-500">Versi Saat Ini:</span>
            <span class="font-mono text-zinc-400 font-semibold">v{{ currentVersion }}</span>
          </div>
          <div class="text-cyan-400 font-bold">➔</div>
          <div class="flex items-center gap-2">
            <span class="text-zinc-500">Versi Terbaru:</span>
            <span class="font-mono text-emerald-400 font-bold">v{{ newVersion }}</span>
          </div>
        </div>

        <!-- Release Notes Box (if any) -->
        <div v-if="releaseNotes" class="mt-3 p-3 rounded-xl bg-zinc-900/50 border border-zinc-800/80 max-h-32 overflow-y-auto text-xs text-zinc-300 leading-relaxed font-sans">
          <div class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider mb-1">Catatan Rilis:</div>
          <div class="whitespace-pre-line text-zinc-300">{{ releaseNotes }}</div>
        </div>

        <!-- Download Progress Bar (When downloading) -->
        <div v-if="isDownloading" class="mt-4 space-y-2">
          <div class="flex items-center justify-between text-xs">
            <span class="text-cyan-300 font-medium flex items-center gap-1.5">
              <RefreshCw class="w-3.5 h-3.5 animate-spin text-cyan-400" />
              Mengunduh pembaruan...
            </span>
            <span class="font-mono text-cyan-400 font-bold">{{ progressPercent }}%</span>
          </div>

          <!-- Progress track -->
          <div class="w-full h-2.5 rounded-full bg-zinc-800 overflow-hidden border border-zinc-700/60 p-0.5">
            <div
              class="h-full rounded-full bg-gradient-to-r from-cyan-500 to-blue-500 transition-all duration-300 shadow-sm shadow-cyan-400/50"
              :style="{ width: `${progressPercent}%` }"
            ></div>
          </div>

          <div class="flex items-center justify-between text-[11px] text-zinc-500 font-mono">
            <span>{{ formatSize(downloadedBytes) }} / {{ formatSize(totalBytes) }}</span>
            <span>Memasang otomatis setelah selesai</span>
          </div>
        </div>

        <!-- Error Alert (If download/check fails) -->
        <div v-if="errorMessage" class="mt-4 p-3 rounded-xl bg-rose-950/50 border border-rose-800/60 text-xs text-rose-300 flex items-start gap-2.5">
          <AlertCircle class="w-4 h-4 text-rose-400 shrink-0 mt-0.5" />
          <div class="flex-1">
            <div class="font-semibold text-rose-200">Gagal mengunduh pembaruan</div>
            <div class="text-[11px] text-rose-300/80 mt-0.5">{{ errorMessage }}</div>
          </div>
        </div>

        <!-- Action Footer -->
        <div class="mt-6 pt-4 border-t border-zinc-800/80 flex items-center justify-between gap-3">
          <button
            @click="openGithubRelease"
            class="text-xs text-zinc-400 hover:text-cyan-300 flex items-center gap-1 transition-colors cursor-pointer"
            title="Buka halaman rilis di browser"
          >
            <ExternalLink class="w-3.5 h-3.5" /> Unduh Manual di GitHub
          </button>

          <button
            @click="emit('start-update')"
            :disabled="isDownloading"
            class="px-5 py-2 rounded-xl text-xs font-bold bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500 text-white transition-all shadow-lg shadow-cyan-500/25 flex items-center gap-2 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <DownloadCloud v-if="!isDownloading" class="w-4 h-4" />
            <RefreshCw v-else class="w-4 h-4 animate-spin" />
            {{ isDownloading ? 'Mengunduh...' : (errorMessage ? 'Coba Lagi' : 'Perbarui Sekarang') }}
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.25s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
