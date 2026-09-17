<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  X,
  Database,
  Palette,
  Clock,
  CheckCircle2,
  ShieldAlert,
  ShieldCheck,
  RefreshCw,
  Sparkles
} from "lucide-vue-next";
import type { BlockedIpRecord } from "@/types/network";

const props = defineProps<{
  open: boolean;
  isDark: boolean;
  blockedIps?: BlockedIpRecord[];
  currentVersion?: string;
  updateAvailable?: boolean;
  newVersion?: string;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  "update:isDark": [value: boolean];
  "unblock-ip": [ip: string];
  "check-update": [];
}>();

const activeTab = ref<"general" | "blocked" | "updates">("general");

// ─── Retention options ────────────────────────────────────────────────────────
const RETENTION_OPTIONS = [
  { label: "7 hari", value: "7" },
  { label: "30 hari", value: "30" },
  { label: "90 hari", value: "90" },
  { label: "6 bulan", value: "180" },
  { label: "1 tahun", value: "365" },
  { label: "Selamanya", value: "0" },
];

const retentionDays = ref("30");
const isSaving = ref(false);
const savedFeedback = ref(false);

// ─── Load settings from DB on open ───────────────────────────────────────────
onMounted(async () => {
  try {
    const stored = await invoke<string | null>("get_setting", { key: "retention_days" });
    if (stored !== null && stored !== undefined) {
      retentionDays.value = stored;
    }
  } catch (_) {}
});

watch(() => props.open, async (val) => {
  if (val) {
    try {
      const stored = await invoke<string | null>("get_setting", { key: "retention_days" });
      if (stored !== null && stored !== undefined) {
        retentionDays.value = stored;
      }
    } catch (_) {}
  }
});

// ─── Theme toggle ─────────────────────────────────────────────────────────────
function toggleTheme() {
  const newVal = !props.isDark;
  emit("update:isDark", newVal);
  localStorage.setItem("theme", newVal ? "dark" : "light");
}

// ─── Save settings ────────────────────────────────────────────────────────────
async function saveSettings() {
  isSaving.value = true;
  try {
    await invoke("set_setting", { key: "retention_days", value: retentionDays.value });
    savedFeedback.value = true;
    setTimeout(() => (savedFeedback.value = false), 2000);
  } catch (_) {}
  isSaving.value = false;
}

function formatDate(ts: number) {
  return new Date(ts).toLocaleString([], {
    dateStyle: "short",
    timeStyle: "short",
  });
}

function close() {
  emit("update:open", false);
}
</script>

<template>
  <!-- Backdrop -->
  <Transition name="backdrop">
    <div
      v-if="open"
      class="fixed inset-0 z-[100] flex items-center justify-center p-4 select-none"
      @click.self="close"
    >
      <!-- Blur overlay -->
      <div class="absolute inset-0 backdrop-blur-sm" :class="isDark ? 'bg-black/60' : 'bg-black/20'" />

      <!-- Modal -->
      <Transition name="modal">
        <div
          v-if="open"
          :class="[
            'relative z-10 w-full max-w-lg rounded-2xl border shadow-2xl overflow-hidden flex flex-col max-h-[85vh]',
            isDark
              ? 'bg-zinc-900/95 border-zinc-800 shadow-black/60 text-zinc-100'
              : 'bg-white/95 border-zinc-200 shadow-zinc-300/60 text-zinc-900'
          ]"
        >
          <!-- Header -->
          <div :class="['flex items-center justify-between px-6 py-4 border-b shrink-0', isDark ? 'border-zinc-800' : 'border-zinc-100']">
            <div class="flex items-center gap-2.5">
              <div class="w-8 h-8 rounded-lg bg-gradient-to-tr from-cyan-600 to-blue-500 flex items-center justify-center shadow-md shadow-cyan-500/20 text-white">
                <Database class="w-4 h-4" />
              </div>
              <div>
                <h2 :class="['text-sm font-semibold', isDark ? 'text-white' : 'text-zinc-900']">Pengaturan & Pertahanan</h2>
                <p :class="['text-xs', isDark ? 'text-zinc-500' : 'text-zinc-400']">Konfigurasi, firewall, dan pembaruan</p>
              </div>
            </div>
            <button
              @click="close"
              :class="['w-7 h-7 rounded-lg flex items-center justify-center transition-colors cursor-pointer', isDark ? 'hover:bg-zinc-800 text-zinc-400 hover:text-white' : 'hover:bg-zinc-100 text-zinc-500 hover:text-zinc-900']"
            >
              <X class="w-4 h-4" />
            </button>
          </div>

          <!-- Navigation Tabs -->
          <div :class="['flex items-center px-6 pt-3 border-b gap-4 text-xs font-medium shrink-0', isDark ? 'border-zinc-800 bg-zinc-950/40' : 'border-zinc-100 bg-zinc-50/50']">
            <button
              @click="activeTab = 'general'"
              :class="[
                'pb-2.5 border-b-2 transition-all cursor-pointer font-semibold flex items-center gap-1.5',
                activeTab === 'general'
                  ? 'border-cyan-500 text-cyan-400'
                  : 'border-transparent text-zinc-500 hover:text-zinc-300'
              ]"
            >
              <Database class="w-3.5 h-3.5" />
              Umum & Retensi
            </button>

            <button
              @click="activeTab = 'blocked'"
              :class="[
                'pb-2.5 border-b-2 transition-all cursor-pointer font-semibold flex items-center gap-1.5',
                activeTab === 'blocked'
                  ? 'border-rose-500 text-rose-400'
                  : 'border-transparent text-zinc-500 hover:text-zinc-300'
              ]"
            >
              <ShieldAlert class="w-3.5 h-3.5" />
              IP Terblokir
              <span
                v-if="blockedIps && blockedIps.length > 0"
                class="px-1.5 py-0.2 rounded-full text-[10px] bg-rose-950 text-rose-300 border border-rose-800/80 font-mono"
              >
                {{ blockedIps.length }}
              </span>
            </button>

            <button
              @click="activeTab = 'updates'"
              :class="[
                'pb-2.5 border-b-2 transition-all cursor-pointer font-semibold flex items-center gap-1.5',
                activeTab === 'updates'
                  ? 'border-cyan-500 text-cyan-400'
                  : 'border-transparent text-zinc-500 hover:text-zinc-300'
              ]"
            >
              <Sparkles class="w-3.5 h-3.5" />
              Pembaruan App
              <span
                v-if="updateAvailable"
                class="w-2 h-2 rounded-full bg-cyan-400 animate-pulse"
              ></span>
            </button>
          </div>

          <!-- Body Content (Scrollable) -->
          <div class="px-6 py-5 space-y-6 overflow-y-auto flex-1">
            <!-- TAB 1: GENERAL & RETENTION -->
            <template v-if="activeTab === 'general'">
              <!-- Appearance Section -->
              <section class="space-y-3">
                <div class="flex items-center gap-2">
                  <Palette :class="['w-3.5 h-3.5', isDark ? 'text-cyan-400' : 'text-cyan-600']" />
                  <span :class="['text-xs font-semibold uppercase tracking-wider', isDark ? 'text-zinc-400' : 'text-zinc-500']">Tampilan</span>
                </div>

                <div :class="['flex items-center justify-between p-4 rounded-xl border', isDark ? 'bg-zinc-800/60 border-zinc-700/50' : 'bg-zinc-50 border-zinc-200']">
                  <div>
                    <p :class="['text-sm font-medium', isDark ? 'text-white' : 'text-zinc-900']">Mode Gelap</p>
                    <p :class="['text-xs mt-0.5', isDark ? 'text-zinc-500' : 'text-zinc-400']">
                      {{ isDark ? 'Aktif — tampilan saat ini gelap' : 'Nonaktif — tampilan saat ini terang' }}
                    </p>
                  </div>
                  <!-- Toggle switch -->
                  <button
                    @click="toggleTheme"
                    :class="[
                      'relative w-11 h-6 rounded-full transition-colors duration-300 cursor-pointer flex-shrink-0',
                      isDark ? 'bg-cyan-600' : 'bg-zinc-300'
                    ]"
                    role="switch"
                    :aria-checked="isDark"
                  >
                    <span :class="[
                      'absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow-sm transition-transform duration-300',
                      isDark ? 'translate-x-5' : 'translate-x-0'
                    ]" />
                  </button>
                </div>
              </section>

              <!-- Data Retention Section -->
              <section class="space-y-3">
                <div class="flex items-center gap-2">
                  <Clock :class="['w-3.5 h-3.5', isDark ? 'text-cyan-400' : 'text-cyan-600']" />
                  <span :class="['text-xs font-semibold uppercase tracking-wider', isDark ? 'text-zinc-400' : 'text-zinc-500']">Retensi Data</span>
                </div>

                <div :class="['p-4 rounded-xl border space-y-3', isDark ? 'bg-zinc-800/60 border-zinc-700/50' : 'bg-zinc-50 border-zinc-200']">
                  <div>
                    <p :class="['text-sm font-medium', isDark ? 'text-white' : 'text-zinc-900']">Simpan history selama</p>
                    <p :class="['text-xs mt-0.5', isDark ? 'text-zinc-500' : 'text-zinc-400']">
                      Log lebih lama dari durasi ini akan otomatis dihapus saat app dibuka
                    </p>
                  </div>

                  <!-- Retention grid buttons -->
                  <div class="grid grid-cols-3 gap-2">
                    <button
                      v-for="opt in RETENTION_OPTIONS"
                      :key="opt.value"
                      @click="retentionDays = opt.value"
                      :class="[
                        'py-2 px-3 rounded-lg text-xs font-medium transition-all cursor-pointer border',
                        retentionDays === opt.value
                          ? 'bg-cyan-600 border-cyan-600 text-white shadow-md shadow-cyan-500/20'
                          : isDark
                            ? 'bg-zinc-700/50 border-zinc-600/50 text-zinc-300 hover:bg-zinc-700 hover:text-white'
                            : 'bg-white border-zinc-200 text-zinc-600 hover:bg-zinc-100 hover:text-zinc-900'
                      ]"
                    >
                      {{ opt.label }}
                    </button>
                  </div>

                  <!-- Info pill -->
                  <div :class="['flex items-center gap-1.5 text-xs px-3 py-2 rounded-lg', isDark ? 'bg-zinc-900/60 text-zinc-500' : 'bg-zinc-100 text-zinc-500']">
                    <Database class="w-3 h-3 flex-shrink-0" />
                    <span v-if="retentionDays === '0'">Data history tidak pernah dihapus secara otomatis</span>
                    <span v-else>Data lebih dari <strong class="font-semibold">{{ RETENTION_OPTIONS.find(o => o.value === retentionDays)?.label }}</strong> akan dihapus saat startup</span>
                  </div>
                </div>
              </section>
            </template>

            <!-- TAB 2: BLOCKED IPS -->
            <template v-else-if="activeTab === 'blocked'">
              <div class="space-y-3">
                <div class="flex items-center justify-between">
                  <div>
                    <h3 class="text-xs font-semibold uppercase tracking-wider text-rose-400 flex items-center gap-1.5">
                      <ShieldAlert class="w-3.5 h-3.5" />
                      Daftar IP yang Diblokir di Windows Firewall
                    </h3>
                    <p class="text-xs text-zinc-400 mt-0.5">
                      Seluruh koneksi outbound & inbound ke IP ini diblokir secara lokal.
                    </p>
                  </div>
                </div>

                <div v-if="!blockedIps || blockedIps.length === 0" class="text-center py-10 rounded-xl border border-zinc-800/80 bg-zinc-950/40 text-zinc-500 space-y-2">
                  <ShieldCheck class="w-8 h-8 mx-auto opacity-40 text-emerald-400" />
                  <p class="text-xs">Belum ada IP yang diblokir oleh Net Tracker.</p>
                </div>

                <div v-else class="rounded-xl border border-zinc-800/80 overflow-hidden bg-zinc-950/60 divide-y divide-zinc-800/60 font-mono text-xs">
                  <div
                    v-for="b in blockedIps"
                    :key="b.ip"
                    class="p-3 flex items-center justify-between gap-3 hover:bg-zinc-900/50 transition-colors"
                  >
                    <div>
                      <div class="font-bold text-rose-400 flex items-center gap-2">
                        <span>{{ b.ip }}</span>
                        <span v-if="b.hostname" class="text-[11px] text-zinc-400 font-sans font-medium">({{ b.hostname }})</span>
                      </div>
                      <div class="text-[10px] text-zinc-500 font-sans mt-0.5 flex items-center gap-2">
                        <span v-if="b.process_name">Aplikasi: {{ b.process_name }}</span>
                        <span>•</span>
                        <span>Diblokir: {{ formatDate(b.blocked_at) }}</span>
                      </div>
                    </div>

                    <button
                      @click="emit('unblock-ip', b.ip)"
                      class="px-2.5 py-1 rounded-lg text-xs font-semibold bg-emerald-950/80 hover:bg-emerald-900 text-emerald-300 border border-emerald-800/80 flex items-center gap-1 transition-colors cursor-pointer shrink-0"
                      title="Buka kembali blokir firewall untuk IP ini"
                    >
                      <ShieldCheck class="w-3.5 h-3.5" />
                      <span>Release</span>
                    </button>
                  </div>
                </div>
              </div>
            </template>

            <!-- TAB 3: APP UPDATES -->
            <template v-else-if="activeTab === 'updates'">
              <div class="space-y-4">
                <div class="p-4 rounded-xl border border-zinc-800 bg-zinc-950/60 space-y-3">
                  <div class="flex items-center justify-between">
                    <div>
                      <div class="text-xs text-zinc-400">Versi Terpasang:</div>
                      <div class="font-mono text-base font-bold text-cyan-400">v{{ currentVersion || '0.1.0' }}</div>
                    </div>

                    <button
                      @click="emit('check-update')"
                      class="px-3 py-1.5 rounded-lg text-xs font-semibold bg-cyan-600 hover:bg-cyan-500 text-white flex items-center gap-1.5 transition-all shadow-md shadow-cyan-600/20 cursor-pointer"
                    >
                      <RefreshCw class="w-3.5 h-3.5" />
                      Periksa Pembaruan
                    </button>
                  </div>

                  <div class="text-xs text-zinc-400 pt-2 border-t border-zinc-800/80 leading-relaxed">
                    Net Tracker secara otomatis memeriksa pembaruan baru dari GitHub Releases setiap kali aplikasi dibuka.
                  </div>
                </div>

                <div v-if="updateAvailable" class="p-4 rounded-xl border border-cyan-500/50 bg-cyan-950/30 text-xs space-y-2">
                  <div class="flex items-center gap-2 font-bold text-cyan-300">
                    <Sparkles class="w-4 h-4 text-cyan-400" />
                    Versi baru v{{ newVersion }} telah dirilis!
                  </div>
                  <p class="text-zinc-300">
                    Pembaruan wajib akan dipasang secara otomatis untuk memastikan stabilitas dan keamanan komputer Anda.
                  </p>
                </div>
              </div>
            </template>
          </div>

          <!-- Footer -->
          <div :class="['flex items-center justify-end gap-3 px-6 py-3.5 border-t shrink-0', isDark ? 'border-zinc-800 bg-zinc-950/30' : 'border-zinc-100 bg-zinc-50/50']">
            <Transition name="fade-check">
              <div v-if="savedFeedback" class="flex items-center gap-1.5 text-xs text-emerald-400 mr-auto">
                <CheckCircle2 class="w-3.5 h-3.5" />
                Tersimpan
              </div>
            </Transition>
            <button
              @click="close"
              :class="['px-4 py-1.5 rounded-lg text-xs font-medium border transition-colors cursor-pointer', isDark ? 'border-zinc-700 text-zinc-400 hover:bg-zinc-800 hover:text-white' : 'border-zinc-200 text-zinc-500 hover:bg-zinc-100']"
            >
              Tutup
            </button>
            <button
              v-if="activeTab === 'general'"
              @click="saveSettings"
              :disabled="isSaving"
              class="px-4 py-1.5 rounded-lg text-xs font-semibold bg-cyan-600 hover:bg-cyan-500 text-white transition-colors cursor-pointer disabled:opacity-50"
            >
              {{ isSaving ? 'Menyimpan...' : 'Simpan' }}
            </button>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<style scoped>
/* Backdrop transition */
.backdrop-enter-active,
.backdrop-leave-active {
  transition: opacity 0.2s ease;
}
.backdrop-enter-from,
.backdrop-leave-to {
  opacity: 0;
}

/* Modal pop transition */
.modal-enter-active {
  transition: opacity 0.2s ease, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}
.modal-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(8px);
}

/* Saved feedback fade */
.fade-check-enter-active,
.fade-check-leave-active {
  transition: opacity 0.3s ease;
}
.fade-check-enter-from,
.fade-check-leave-to {
  opacity: 0;
}
</style>
