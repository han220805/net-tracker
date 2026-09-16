<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Activity,
  History as HistoryIcon,
  RefreshCw,
  Wifi,
} from "lucide-vue-next";
import NetworkStatCards from "@/components/NetworkStatCards.vue";
import NetworkTable from "@/components/NetworkTable.vue";
import SpeedGraph from "@/components/SpeedGraph.vue";
import HistoryView from "@/components/HistoryView.vue";
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

// Fetch active network connections from Tauri backend
async function fetchNetworkData() {
  try {
    const data: any = await invoke("get_network_snapshot");
    if (data) {
      connections.value = data.connections || [];
      summary.value = data.summary || summary.value;
      if (data.history) {
        historyRecords.value = data.history;
      }
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

function handleClearHistory() {
  historyRecords.value = [];
}

onMounted(() => {
  fetchNetworkData();
  startPolling();
});

onUnmounted(() => {
  stopPolling();
});
</script>

<template>
  <div class="min-h-screen bg-[#07090e] text-zinc-100 flex flex-col selection:bg-cyan-500/30">
    <!-- Top Navigation Header -->
    <header class="h-14 border-b border-zinc-800/80 bg-zinc-950/70 backdrop-blur-md px-6 flex items-center justify-between sticky top-0 z-50">
      <!-- App Brand -->
      <div class="flex items-center gap-3">
        <div class="w-8 h-8 rounded-lg bg-gradient-to-tr from-cyan-600 to-blue-500 flex items-center justify-center shadow-lg shadow-cyan-500/20">
          <Wifi class="w-4 h-4 text-white" />
        </div>
        <div>
          <h1 class="text-sm font-bold tracking-tight text-white flex items-center gap-2">
            Net Tracker
            <span class="text-[10px] font-medium px-1.5 py-0.5 rounded bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
              v1.0
            </span>
          </h1>
          <p class="text-[11px] text-zinc-400">Endpoint Connection & Hostname Auditor</p>
        </div>
      </div>

      <!-- Center Tabs -->
      <div class="flex items-center bg-zinc-900/80 p-1 rounded-lg border border-zinc-800/80">
        <button
          @click="activeTab = 'live'"
          :class="[
            'flex items-center gap-2 px-3 py-1 rounded-md text-xs font-medium transition-all',
            activeTab === 'live'
              ? 'bg-cyan-600 text-white shadow'
              : 'text-zinc-400 hover:text-white'
          ]"
        >
          <Activity class="w-3.5 h-3.5" />
          Realtime Monitor
        </button>
        <button
          @click="activeTab = 'history'"
          :class="[
            'flex items-center gap-2 px-3 py-1 rounded-md text-xs font-medium transition-all',
            activeTab === 'history'
              ? 'bg-cyan-600 text-white shadow'
              : 'text-zinc-400 hover:text-white'
          ]"
        >
          <HistoryIcon class="w-3.5 h-3.5" />
          History Log
        </button>
      </div>

      <!-- Controls -->
      <div class="flex items-center gap-3">
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
          {{ isLiveActive ? 'Live Polling 1s' : 'Paused' }}
        </button>

        <Button variant="outline" size="sm" @click="fetchNetworkData" title="Refresh snapshot">
          <RefreshCw class="w-3.5 h-3.5" />
        </Button>
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