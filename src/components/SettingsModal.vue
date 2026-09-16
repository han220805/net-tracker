<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { X, Database, Palette, Clock, CheckCircle2 } from "lucide-vue-next";

const props = defineProps<{
  open: boolean;
  isDark: boolean;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  "update:isDark": [value: boolean];
}>();

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

function close() {
  emit("update:open", false);
}
</script>

<template>
  <!-- Backdrop -->
  <Transition name="backdrop">
    <div
      v-if="open"
      class="fixed inset-0 z-[100] flex items-center justify-center p-4"
      @click.self="close"
    >
      <!-- Blur overlay -->
      <div class="absolute inset-0 backdrop-blur-sm" :class="isDark ? 'bg-black/60' : 'bg-black/20'" />

      <!-- Modal -->
      <Transition name="modal">
        <div
          v-if="open"
          :class="[
            'relative z-10 w-full max-w-md rounded-2xl border shadow-2xl overflow-hidden',
            isDark
              ? 'bg-zinc-900/95 border-zinc-800 shadow-black/60'
              : 'bg-white/95 border-zinc-200 shadow-zinc-300/60'
          ]"
        >
          <!-- Header -->
          <div :class="['flex items-center justify-between px-6 py-4 border-b', isDark ? 'border-zinc-800' : 'border-zinc-100']">
            <div class="flex items-center gap-2.5">
              <div class="w-8 h-8 rounded-lg bg-gradient-to-tr from-cyan-600 to-blue-500 flex items-center justify-center shadow-md shadow-cyan-500/20">
                <Database class="w-4 h-4 text-white" />
              </div>
              <div>
                <h2 :class="['text-sm font-semibold', isDark ? 'text-white' : 'text-zinc-900']">Pengaturan</h2>
                <p :class="['text-xs', isDark ? 'text-zinc-500' : 'text-zinc-400']">Konfigurasi aplikasi</p>
              </div>
            </div>
            <button
              @click="close"
              :class="['w-7 h-7 rounded-lg flex items-center justify-center transition-colors cursor-pointer', isDark ? 'hover:bg-zinc-800 text-zinc-400 hover:text-white' : 'hover:bg-zinc-100 text-zinc-500 hover:text-zinc-900']"
            >
              <X class="w-4 h-4" />
            </button>
          </div>

          <!-- Body -->
          <div class="px-6 py-5 space-y-6">

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
          </div>

          <!-- Footer -->
          <div :class="['flex items-center justify-end gap-3 px-6 py-4 border-t', isDark ? 'border-zinc-800' : 'border-zinc-100']">
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
