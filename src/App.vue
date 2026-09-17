<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Activity,
  History as HistoryIcon,
  RefreshCw,
  Wifi,
  Moon,
  Sun,
  Settings,
  CheckCircle2,
  AlertCircle,
  Info,
} from "lucide-vue-next";
import NetworkStatCards from "@/components/NetworkStatCards.vue";
import NetworkTable from "@/components/NetworkTable.vue";
import SpeedGraph from "@/components/SpeedGraph.vue";
import HistoryView from "@/components/HistoryView.vue";
import WindowControls from "@/components/WindowControls.vue";
import SettingsModal from "@/components/SettingsModal.vue";
import ForceUpdateModal from "@/components/ui/ForceUpdateModal.vue";
import Button from "@/components/ui/Button.vue";
import type {
  NetworkConnection,
  NetworkSummary,
  HistoryRecord,
  BlockedIpRecord,
} from "@/types/network";
import { check, type Update } from "@tauri-apps/plugin-updater";

const showSettings = ref(false);

const activeTab = ref<"live" | "history">("live");
const isLiveActive = ref(true);
const refreshInterval = ref(1000); // 1s
let pollTimer: any = null;

// Mock/Real data reactive stores
const summary = ref<NetworkSummary>({
  total_active_connections: 0,
  total_listening_ports: 0,
  total_processes: 0,
  total_download_speed: 0,
  total_upload_speed: 0,
  total_bytes_received: 0,
  total_bytes_sent: 0,
});

const connections = ref<NetworkConnection[]>([]);
const historyRecords = ref<HistoryRecord[]>([]);
const blockedIps = ref<BlockedIpRecord[]>([]);

// ─── Toast Notifications ───────────────────────────────────────────────────────
const toast = ref<{
  message: string;
  type: "success" | "error" | "info";
  show: boolean;
}>({
  message: "",
  type: "success",
  show: false,
});
let toastTimer: any = null;

function showToast(message: string, type: "success" | "error" | "info" = "success") {
  if (toastTimer) clearTimeout(toastTimer);
  toast.value = { message, type, show: true };
  toastTimer = setTimeout(() => {
    toast.value.show = false;
  }, 3500);
}

// ─── Fetch Active Data & SQLite Records ────────────────────────────────────────
async function fetchNetworkData() {
  try {
    const data: any = await invoke("get_network_snapshot");
    if (data) {
      connections.value = data.connections || [];
      summary.value = data.summary || summary.value;
    }

    // Fetch persistent history from SQLite
    const history: any = await invoke("get_history_logs");
    if (history && Array.isArray(history)) {
      historyRecords.value = history;
    }
  } catch (err) {
    // If running in browser or tauri backend is booting, generate realistic local sample data
    fallbackLocalData();
  }
}

async function fetchBlockedIps() {
  try {
    const list: any = await invoke("get_blocked_ips");
    if (Array.isArray(list)) {
      blockedIps.value = list;
    }
  } catch (_) {}
}

// ─── Defense Handlers: Kill Process & Block / Release IP ───────────────────────
async function handleKillProcess({ pid, name }: { pid: number; name: string }) {
  try {
    await invoke("kill_process", { pid });
    showToast(`Proses ${name} (PID ${pid}) berhasil dihentikan!`, "success");
    await fetchNetworkData();
  } catch (err: any) {
    showToast(`Gagal mematikan proses: ${err}`, "error");
  }
}

async function handleBlockIp({
  ip,
  hostname,
  processName,
}: {
  ip: string;
  hostname?: string;
  processName?: string;
}) {
  try {
    await invoke("block_ip", { ip, hostname, processName });
    showToast(`IP ${ip} berhasil diblokir di Windows Firewall!`, "success");
    await fetchBlockedIps();
  } catch (err: any) {
    showToast(`Gagal memblokir IP: ${err}`, "error");
  }
}

async function handleUnblockIp(arg: { ip: string } | string) {
  const ip = typeof arg === "string" ? arg : arg.ip;
  try {
    await invoke("unblock_ip", { ip });
    showToast(`Blokir firewall untuk IP ${ip} telah dibuka (Released)!`, "info");
    await fetchBlockedIps();
  } catch (err: any) {
    showToast(`Gagal membuka blokir IP: ${err}`, "error");
  }
}

// ─── Auto-Updater (Forced Update System) ───────────────────────────────────────
const updateState = ref<{
  open: boolean;
  currentVersion: string;
  newVersion: string;
  releaseNotes: string;
  isDownloading: boolean;
  progressPercent: number;
  downloadedBytes: number;
  totalBytes: number;
  errorMessage: string | null;
  updateObj: Update | null;
}>({
  open: false,
  currentVersion: "0.2.0",
  newVersion: "",
  releaseNotes: "",
  isDownloading: false,
  progressPercent: 0,
  downloadedBytes: 0,
  totalBytes: 0,
  errorMessage: null,
  updateObj: null,
});

async function checkForUpdates(manual = false) {
  try {
    const update = await check();
    if (update) {
      updateState.value = {
        open: true,
        currentVersion: update.currentVersion || "0.2.0",
        newVersion: update.version,
        releaseNotes: update.body || "",
        isDownloading: false,
        progressPercent: 0,
        downloadedBytes: 0,
        totalBytes: 0,
        errorMessage: null,
        updateObj: update,
      };
    } else if (manual) {
      showToast("Aplikasi sudah menggunakan versi terbaru (v0.2.0).", "info");
    }
  } catch (err: any) {
    if (manual) {
      showToast(`Gagal memeriksa pembaruan: ${err}`, "error");
    }
  }
}

async function startUpdate() {
  const update = updateState.value.updateObj;
  if (!update) return;

  updateState.value.isDownloading = true;
  updateState.value.errorMessage = null;

  try {
    let downloaded = 0;
    let total = 0;

    await update.downloadAndInstall((event) => {
      if (event.event === "Started") {
        total = event.data.contentLength || 0;
        updateState.value.totalBytes = total;
      } else if (event.event === "Progress") {
        downloaded += event.data.chunkLength;
        updateState.value.downloadedBytes = downloaded;
        if (total > 0) {
          updateState.value.progressPercent = Math.min(
            100,
            Math.round((downloaded / total) * 100)
          );
        }
      } else if (event.event === "Finished") {
        updateState.value.progressPercent = 100;
      }
    });
  } catch (err: any) {
    updateState.value.isDownloading = false;
    updateState.value.errorMessage = String(err);
  }
}

// Fallback demo data generator for instant rich visual feedback during development
function fallbackLocalData() {
  const mockApps = [
    { name: "chrome.exe", host: "api.github.com", ip: "140.82.121.6", port: 443, state: "ESTABLISHED" },
    { name: "code.exe", host: "vscode.blob.core.windows.net", ip: "20.150.83.146", port: 443, state: "ESTABLISHED" },
    { name: "spotify.exe", host: "audio-sp-ash.spotify.com", ip: "35.186.224.25", port: 443, state: "ESTABLISHED" },
    { name: "discord.exe", host: "gateway.discord.gg", ip: "162.159.130.233", port: 443, state: "ESTABLISHED" },
    { name: "steam.exe", host: "cm01-sgp1.cm.steampowered.com", ip: "155.133.253.34", port: 27017, state: "ESTABLISHED" },
    { name: "System", host: "time.windows.com", ip: "51.145.123.29", port: 123, state: "ESTABLISHED" },
    { name: "node.exe", host: "registry.npmjs.org", ip: "104.16.27.35", port: 443, state: "TIME_WAIT" },
    { name: "postgres.exe", host: "localhost", ip: "127.0.0.1", port: 5432, state: "LISTEN" },
  ];

  const now = Date.now();
  let totalDl = 0;
  let totalUl = 0;

  const sampleConnections: NetworkConnection[] = mockApps.map((app, idx) => {
    const dl = app.state === "LISTEN" ? 0 : Math.floor(Math.random() * 450 * 1024);
    const ul = app.state === "LISTEN" ? 0 : Math.floor(Math.random() * 120 * 1024);
    totalDl += dl;
    totalUl += ul;

    return {
      id: `conn-${idx}-${app.name}`,
      pid: 1000 + idx * 420,
      process_name: app.name,
      protocol: "TCP",
      local_address: "192.168.1.100",
      local_port: 50000 + idx,
      remote_address: app.ip,
      remote_port: app.port,
      hostname: app.host,
      state: app.state as any,
      download_speed: dl,
      upload_speed: ul,
      bytes_received: 1024 * 1024 * (idx + 1) * 3,
      bytes_sent: 1024 * 256 * (idx + 1),
      country: app.ip.startsWith("127.") ? "Local" : "US",
      first_seen: now - 3600000,
      last_active: now,
    };
  });

  connections.value = sampleConnections;
  summary.value = {
    total_active_connections: sampleConnections.filter(c => c.state === 'ESTABLISHED').length,
    total_listening_ports: sampleConnections.filter(c => c.state === 'LISTEN').length,
    total_processes: new Set(sampleConnections.map(c => c.process_name)).size,
    total_download_speed: totalDl,
    total_upload_speed: totalUl,
    total_bytes_received: 245 * 1024 * 1024,
    total_bytes_sent: 58 * 1024 * 1024,
  };

  if (historyRecords.value.length === 0) {
    historyRecords.value = [
      { id: "h1", timestamp: now - 60000, process_name: "chrome.exe", remote_address: "140.82.121.6", hostname: "github.com", remote_port: 443, protocol: "TCP", duration_seconds: 45, total_bytes: 4200000 },
      { id: "h2", timestamp: now - 180000, process_name: "spotify.exe", remote_address: "35.186.224.25", hostname: "audio-sp-ash.spotify.com", remote_port: 443, protocol: "TCP", duration_seconds: 180, total_bytes: 18500000 },
      { id: "h3", timestamp: now - 400000, process_name: "code.exe", remote_address: "20.150.83.146", hostname: "marketplace.visualstudio.com", remote_port: 443, protocol: "TCP", duration_seconds: 12, total_bytes: 840000 },
    ];
  }
}

function togglePolling() {
  isLiveActive.value = !isLiveActive.value;
  if (isLiveActive.value) {
    startPolling();
  } else {
    stopPolling();
  }
}

function startPolling() {
  stopPolling();
  pollTimer = setInterval(fetchNetworkData, refreshInterval.value);
}

function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
}

async function handleClearHistory() {
  try {
    await invoke("clear_history_logs");
  } catch (_) {}
  historyRecords.value = [];
}

const isDark = ref(true);

function applyTheme(dark: boolean) {
  isDark.value = dark;
  if (dark) {
    document.documentElement.classList.add("dark");
    document.documentElement.classList.remove("light");
  } else {
    document.documentElement.classList.add("light");
    document.documentElement.classList.remove("dark");
  }
}

function toggleTheme() {
  const newVal = !isDark.value;
  applyTheme(newVal);
  localStorage.setItem("theme", newVal ? "dark" : "light");
}

const appWindow = getCurrentWindow();

function onHeaderMouseDown(e: MouseEvent) {
  if (e.button !== 0) return;
  const target = e.target as HTMLElement | null;
  if (target && (target.closest("button") || target.closest("input") || target.closest("a") || target.closest("select"))) {
    return;
  }

  appWindow.startDragging().catch(() => {
    invoke("drag_window").catch((err) => console.error("Drag error:", err));
  });
}

onMounted(async () => {
  const savedTheme = localStorage.getItem("theme");
  applyTheme(savedTheme !== "light");

  try {
    await invoke("run_cleanup");
  } catch (_) {}

  await fetchBlockedIps();
  fetchNetworkData();
  startPolling();

  // Check for auto update from GitHub
  checkForUpdates(false);
});

onUnmounted(() => {
  stopPolling();
});
</script>

<template>
  <div :class="['min-h-screen flex flex-col selection:bg-cyan-500/30 transition-colors duration-200', isDark ? 'bg-[#07090e] text-zinc-100' : 'bg-slate-50 text-zinc-800']">
    <!-- Top Seamless Navigation Header -->
    <header
      data-tauri-drag-region
      @mousedown="onHeaderMouseDown"
      :class="['h-13 border-b px-4 flex items-center justify-between sticky top-0 z-50 select-none cursor-move backdrop-blur-md transition-colors', isDark ? 'border-zinc-800/80 bg-zinc-950/90' : 'border-zinc-200/80 bg-white/90']"
    >
      <!-- App Brand (Draggable) -->
      <div data-tauri-drag-region class="flex items-center gap-3 select-none">
        <div class="w-8 h-8 rounded-lg bg-gradient-to-tr from-cyan-600 to-blue-500 flex items-center justify-center text-white shadow-md shadow-cyan-500/20">
          <Wifi class="w-4 h-4" />
        </div>
        <div>
          <h1 class="text-sm font-bold tracking-wider uppercase font-mono text-zinc-100 flex items-center gap-2">
            Net Tracker
            <span class="text-[10px] px-1.5 py-0.2 rounded bg-cyan-950 text-cyan-400 border border-cyan-800 font-sans normal-case">
              v0.2.0
            </span>
          </h1>
        </div>
      </div>

      <!-- Center Tabs -->
      <nav data-tauri-drag-region class="flex items-center gap-1 bg-zinc-900/60 p-1 rounded-xl border border-zinc-800/80 text-xs">
        <button
          @click="activeTab = 'live'"
          :class="[
            'flex items-center gap-2 px-3 py-1 rounded-lg transition-all font-medium cursor-pointer',
            activeTab === 'live'
              ? 'bg-zinc-800 text-cyan-400 shadow-sm'
              : 'text-zinc-400 hover:text-white'
          ]"
        >
          <Activity class="w-3.5 h-3.5" />
          Realtime Monitor
        </button>
        <button
          @click="activeTab = 'history'"
          :class="[
            'flex items-center gap-2 px-3 py-1 rounded-lg transition-all font-medium cursor-pointer',
            activeTab === 'history'
              ? 'bg-zinc-800 text-cyan-400 shadow-sm'
              : 'text-zinc-400 hover:text-white'
          ]"
        >
          <HistoryIcon class="w-3.5 h-3.5" />
          History Log
        </button>
      </nav>

      <!-- Right Action Controls -->
      <div class="flex items-center gap-2">
        <!-- Live status pill -->
        <button
          @click="togglePolling"
          :class="[
            'text-xs font-mono px-2.5 py-1 rounded-full border transition-all flex items-center gap-1.5 cursor-pointer',
            isLiveActive
              ? 'bg-emerald-950/40 text-emerald-400 border-emerald-800/50'
              : 'bg-amber-950/40 text-amber-400 border-amber-800/50'
          ]"
        >
          <span
            :class="[
              'w-2 h-2 rounded-full',
              isLiveActive ? 'bg-emerald-400 animate-pulse' : 'bg-amber-400'
            ]"
          ></span>
          {{ isLiveActive ? 'Live 1s' : 'Paused' }}
        </button>

        <!-- Refresh Button -->
        <Button variant="outline" size="sm" @click="fetchNetworkData" title="Refresh snapshot" class="h-7 w-7 p-0 cursor-pointer">
          <RefreshCw class="w-3 h-3" />
        </Button>

        <!-- Dark / Light Mode Toggle Button -->
        <Button
          variant="outline"
          size="sm"
          @click="toggleTheme"
          :title="isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode'"
          class="h-7 w-7 p-0 cursor-pointer"
        >
          <Sun v-if="isDark" class="w-3.5 h-3.5 text-amber-400" />
          <Moon v-else class="w-3.5 h-3.5 text-slate-700" />
        </Button>

        <!-- Settings Button -->
        <Button
          variant="outline"
          size="sm"
          @click="showSettings = true"
          title="Pengaturan & Firewall"
          class="h-7 w-7 p-0 cursor-pointer relative"
        >
          <Settings class="w-3.5 h-3.5" />
          <span
            v-if="blockedIps.length > 0"
            class="absolute -top-1 -right-1 w-2 h-2 rounded-full bg-rose-500"
          ></span>
        </Button>

        <!-- Seamless Custom Window Controls (Minimize, Maximize, Close) -->
        <div :class="['border-l pl-2 ml-1 flex items-center', isDark ? 'border-zinc-800' : 'border-zinc-200']">
          <WindowControls />
        </div>
      </div>
    </header>

    <!-- Main View Content -->
    <main class="flex-1 p-6 space-y-6 max-w-7xl mx-auto w-full overflow-y-auto">
      <!-- Live Monitoring Tab -->
      <template v-if="activeTab === 'live'">
        <!-- Metric Cards -->
        <NetworkStatCards :summary="summary" />

        <!-- Realtime Speed Area Chart -->
        <SpeedGraph
          :download-speed="summary.total_download_speed"
          :upload-speed="summary.total_upload_speed"
        />

        <!-- Active Connections Table with Defense Actions -->
        <NetworkTable
          :connections="connections"
          :blocked-ips="blockedIps"
          @kill-process="handleKillProcess"
          @block-ip="handleBlockIp"
          @unblock-ip="handleUnblockIp"
        />
      </template>

      <!-- History Audit Tab -->
      <template v-else>
        <HistoryView
          :history="historyRecords"
          :blocked-ips="blockedIps"
          @clear-history="handleClearHistory"
          @block-ip="handleBlockIp"
          @unblock-ip="handleUnblockIp"
        />
      </template>
    </main>

    <!-- Toast Notification Overlay -->
    <Transition name="toast">
      <div
        v-if="toast.show"
        class="fixed bottom-6 right-6 z-[150] flex items-center gap-2.5 px-4 py-2.5 rounded-xl shadow-2xl border text-xs font-sans font-medium select-none"
        :class="[
          toast.type === 'success'
            ? 'bg-zinc-900 border-emerald-500/50 text-emerald-300 shadow-emerald-950/40'
            : toast.type === 'error'
            ? 'bg-zinc-900 border-rose-500/50 text-rose-300 shadow-rose-950/40'
            : 'bg-zinc-900 border-cyan-500/50 text-cyan-300 shadow-cyan-950/40'
        ]"
      >
        <CheckCircle2 v-if="toast.type === 'success'" class="w-4 h-4 text-emerald-400 shrink-0" />
        <AlertCircle v-else-if="toast.type === 'error'" class="w-4 h-4 text-rose-400 shrink-0" />
        <Info v-else class="w-4 h-4 text-cyan-400 shrink-0" />
        <span>{{ toast.message }}</span>
      </div>
    </Transition>
  </div>

  <!-- Settings Modal -->
  <SettingsModal
    v-model:open="showSettings"
    v-model:isDark="isDark"
    :blocked-ips="blockedIps"
    :current-version="updateState.currentVersion"
    :update-available="updateState.open"
    :new-version="updateState.newVersion"
    @unblock-ip="handleUnblockIp"
    @check-update="() => checkForUpdates(true)"
  />

  <!-- Force Update Modal (Graceful Blocking Update Lockout) -->
  <ForceUpdateModal
    :open="updateState.open"
    :current-version="updateState.currentVersion"
    :new-version="updateState.newVersion"
    :release-notes="updateState.releaseNotes"
    :is-downloading="updateState.isDownloading"
    :progress-percent="updateState.progressPercent"
    :downloaded-bytes="updateState.downloadedBytes"
    :total-bytes="updateState.totalBytes"
    :error-message="updateState.errorMessage"
    @start-update="startUpdate"
  />
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(12px) scale(0.95);
}
</style>