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
} from "lucide-vue-next";
import NetworkStatCards from "@/components/NetworkStatCards.vue";
import NetworkTable from "@/components/NetworkTable.vue";
import SpeedGraph from "@/components/SpeedGraph.vue";
import HistoryView from "@/components/HistoryView.vue";
import WindowControls from "@/components/WindowControls.vue";
import Button from "@/components/ui/Button.vue";
import type { NetworkConnection, NetworkSummary, HistoryRecord } from "@/types/network";

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

// Fetch active network connections and history from Tauri backend
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

function toggleTheme() {
  isDark.value = !isDark.value;
  if (isDark.value) {
    document.documentElement.classList.add("dark");
    document.documentElement.classList.remove("light");
  } else {
    document.documentElement.classList.add("light");
    document.documentElement.classList.remove("dark");
  }
}

const appWindow = getCurrentWindow();

function onHeaderMouseDown(e: MouseEvent) {
  // Only trigger on left mouse button and not on interactive buttons / inputs
  if (e.button !== 0) return;
  const target = e.target as HTMLElement | null;
  if (target && (target.closest("button") || target.closest("input") || target.closest("a") || target.closest("select"))) {
    return;
  }

  // Use the native Tauri window startDragging
  appWindow.startDragging().catch(() => {
    // If permission or webview fails, fallback to backend command
    invoke("drag_window").catch((err) => console.error("Drag error:", err));
  });
}

onMounted(() => {
  document.documentElement.classList.add("dark");
  fetchNetworkData();
  startPolling();
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
        <div data-tauri-drag-region class="w-7 h-7 rounded-lg bg-gradient-to-tr from-cyan-600 to-blue-500 flex items-center justify-center shadow-md shadow-cyan-500/20">
          <Wifi class="w-3.5 h-3.5 text-white pointer-events-none" />
        </div>
        <div data-tauri-drag-region>
          <h1 data-tauri-drag-region :class="['text-xs font-bold tracking-tight flex items-center gap-1.5', isDark ? 'text-white' : 'text-zinc-900']">
            Net Tracker
            <span class="text-[9px] font-medium px-1.5 py-0.2 rounded bg-cyan-500/10 text-cyan-500 border border-cyan-500/20 font-mono pointer-events-none">
              v1.0
            </span>
          </h1>
        </div>
      </div>

      <!-- Drag Spacer in between -->
      <div data-tauri-drag-region class="flex-1 h-full cursor-move"></div>

      <!-- Center Tabs -->
      <div class="flex items-center p-1 rounded-lg border cursor-default" :class="isDark ? 'bg-zinc-900/90 border-zinc-800/80' : 'bg-zinc-100 border-zinc-200'">
        <button
          @click="activeTab = 'live'"
          :class="[
            'flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-all cursor-pointer',
            activeTab === 'live'
              ? 'bg-cyan-600 text-white shadow'
              : isDark ? 'text-zinc-400 hover:text-white' : 'text-zinc-500 hover:text-zinc-900'
          ]"
        >
          <Activity class="w-3.5 h-3.5" />
          Realtime Monitor
        </button>
        <button
          @click="activeTab = 'history'"
          :class="[
            'flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-all cursor-pointer',
            activeTab === 'history'
              ? 'bg-cyan-600 text-white shadow'
              : isDark ? 'text-zinc-400 hover:text-white' : 'text-zinc-500 hover:text-zinc-900'
          ]"
        >
          <HistoryIcon class="w-3.5 h-3.5" />
          History Log
        </button>
      </div>

      <!-- Drag Spacer in between -->
      <div data-tauri-drag-region class="flex-1 h-full cursor-move"></div>

      <!-- Controls & Custom Window Buttons -->
      <div class="flex items-center gap-2 cursor-default">
        <!-- Live Status Button -->
        <button
          @click="togglePolling"
          :class="[
            'flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium border transition-all cursor-pointer',
            isLiveActive
              ? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-400'
              : 'border-amber-500/30 bg-amber-500/10 text-amber-400'
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

        <!-- Active Connections Table -->
        <NetworkTable :connections="connections" />
      </template>

      <!-- History Audit Tab -->
      <template v-else>
        <HistoryView :history="historyRecords" @clear-history="handleClearHistory" />
      </template>
    </main>
  </div>
</template>