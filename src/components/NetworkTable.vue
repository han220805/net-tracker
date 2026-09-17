<script setup lang="ts">
import { ref, computed } from "vue";
import {
  Search,
  Copy,
  Check,
  Globe,
  Radio,
  ChevronDown,
  ChevronRight,
  AppWindow,
  Layers,
  List,
  ArrowUpDown,
  Flame,
  ArrowDown,
  ArrowUp,
  HardDrive,
  Zap,
  Skull,
  ShieldAlert
} from "lucide-vue-next";
import Badge from "@/components/ui/Badge.vue";
import Input from "@/components/ui/Input.vue";
import Card from "@/components/ui/Card.vue";
import ConfirmModal from "@/components/ui/ConfirmModal.vue";
import { formatBytes, formatSpeed } from "@/lib/utils";
import type { NetworkConnection, BlockedIpRecord } from "@/types/network";

const props = defineProps<{
  connections: NetworkConnection[];
  isLoading?: boolean;
  blockedIps?: BlockedIpRecord[];
}>();

const emit = defineEmits<{
  (e: "kill-process", payload: { pid: number; name: string }): void;
  (e: "block-ip", payload: { ip: string; hostname?: string; processName?: string }): void;
  (e: "unblock-ip", payload: { ip: string }): void;
}>();

const searchQuery = ref("");
const selectedProtocol = ref<string>("ALL");
const selectedState = ref<string>("ALL");
const viewMode = ref<"grouped" | "flat">("grouped");
const expandedApps = ref<Record<string, boolean>>({});
const copiedIp = ref<string | null>(null);

function isIpBlocked(ip: string): boolean {
  if (!props.blockedIps || !ip) return false;
  return props.blockedIps.some((b) => b.ip === ip);
}

// Confirmation Dialog State
const confirmModal = ref<{
  open: boolean;
  type: "kill" | "block" | "unblock";
  title: string;
  description: string;
  targetName: string;
  targetDetail?: string;
  actionType: "danger" | "warning" | "success";
  confirmText: string;
  payload: any;
}>({
  open: false,
  type: "kill",
  title: "",
  description: "",
  targetName: "",
  targetDetail: "",
  actionType: "danger",
  confirmText: "",
  payload: null,
});

function promptKillProcess(pid: number, name: string) {
  confirmModal.value = {
    open: true,
    type: "kill",
    title: "Hentikan Proses (Kill Process)",
    description: "Apakah Anda yakin ingin mematikan aplikasi ini? Seluruh aktivitas dan soket jaringan yang dimiliki proses ini akan langsung diputus oleh sistem Windows.",
    targetName: name,
    targetDetail: `PID: ${pid}`,
    actionType: "danger",
    confirmText: "Hentikan Proses",
    payload: { pid, name },
  };
}

function promptBlockIp(ip: string, hostname?: string, processName?: string) {
  confirmModal.value = {
    open: true,
    type: "block",
    title: "Blokir Alamat IP di Firewall",
    description: "Apakah Anda yakin ingin memblokir IP ini? Windows Defender Firewall akan memutus semua koneksi masuk (inbound) dan keluar (outbound) ke alamat ini.",
    targetName: ip,
    targetDetail: hostname && hostname !== ip ? `Domain: ${hostname} (${processName || 'App'})` : `Aplikasi: ${processName || 'Unknown'}`,
    actionType: "warning",
    confirmText: "Blokir IP Ini",
    payload: { ip, hostname, processName },
  };
}

function promptUnblockIp(ip: string, hostname?: string) {
  confirmModal.value = {
    open: true,
    type: "unblock",
    title: "Buka Blokir IP (Release)",
    description: "Apakah Anda ingin membuka kembali blokir firewall untuk IP ini? Komputer Anda akan dapat terhubung kembali ke alamat tersebut.",
    targetName: ip,
    targetDetail: hostname ? `Domain: ${hostname}` : undefined,
    actionType: "success",
    confirmText: "Buka Blokir",
    payload: { ip },
  };
}

function executeConfirmedAction() {
  const m = confirmModal.value;
  if (!m.open) return;
  m.open = false;

  if (m.type === "kill") {
    emit("kill-process", m.payload);
  } else if (m.type === "block") {
    emit("block-ip", m.payload);
  } else if (m.type === "unblock") {
    emit("unblock-ip", m.payload);
  }
}

// Sorting state
type SortOption = "bandwidth" | "total_data" | "connections" | "destinations" | "name";
const sortBy = ref<SortOption>("bandwidth"); // Default: highest live bandwidth!
const sortDirection = ref<"desc" | "asc">("desc");

// Group connections by application executable name
interface AppGroup {
  name: string;
  pids: number[];
  connections: NetworkConnection[];
  activeCount: number;
  listenCount: number;
  totalDownloadSpeed: number;
  totalUploadSpeed: number;
  totalBytesIn: number;
  totalBytesOut: number;
  uniqueDestinations: string[];
}

const filteredConnections = computed(() => {
  return props.connections.filter((conn) => {
    if (selectedProtocol.value !== "ALL" && conn.protocol !== selectedProtocol.value) {
      return false;
    }
    if (selectedState.value !== "ALL" && conn.state !== selectedState.value) {
      return false;
    }
    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase();
      const matchProcess = conn.process_name.toLowerCase().includes(q);
      const matchIp = conn.remote_address.toLowerCase().includes(q);
      const matchHost = conn.hostname.toLowerCase().includes(q);
      const matchPort = conn.remote_port.toString().includes(q) || conn.local_port.toString().includes(q);
      const matchPid = conn.pid.toString().includes(q);
      return matchProcess || matchIp || matchHost || matchPort || matchPid;
    }
    return true;
  });
});

const groupedApps = computed<AppGroup[]>(() => {
  const map: Record<string, AppGroup> = {};

  filteredConnections.value.forEach((conn) => {
    const key = conn.process_name || "Unknown App";
    if (!map[key]) {
      map[key] = {
        name: key,
        pids: [],
        connections: [],
        activeCount: 0,
        listenCount: 0,
        totalDownloadSpeed: 0,
        totalUploadSpeed: 0,
        totalBytesIn: 0,
        totalBytesOut: 0,
        uniqueDestinations: [],
      };
    }

    const group = map[key];
    if (!group.pids.includes(conn.pid)) {
      group.pids.push(conn.pid);
    }
    group.connections.push(conn);
    if (conn.state === "ESTABLISHED") group.activeCount++;
    if (conn.state === "LISTEN") group.listenCount++;
    group.totalDownloadSpeed += conn.download_speed;
    group.totalUploadSpeed += conn.upload_speed;
    group.totalBytesIn += conn.bytes_received;
    group.totalBytesOut += conn.bytes_sent;

    const dest = conn.hostname || conn.remote_address;
    if (dest && dest !== "*" && dest !== "0.0.0.0" && !group.uniqueDestinations.includes(dest)) {
      group.uniqueDestinations.push(dest);
    }
  });

  const list = Object.values(map);

  return list.sort((a, b) => {
    if (sortBy.value === "bandwidth") {
      const speedA = a.totalDownloadSpeed + a.totalUploadSpeed;
      const speedB = b.totalDownloadSpeed + b.totalUploadSpeed;
      return sortDirection.value === "desc" ? speedB - speedA : speedA - speedB;
    }
    if (sortBy.value === "total_data") {
      const dataA = a.totalBytesIn + a.totalBytesOut;
      const dataB = b.totalBytesIn + b.totalBytesOut;
      return sortDirection.value === "desc" ? dataB - dataA : dataA - dataB;
    }
    if (sortBy.value === "connections") {
      return sortDirection.value === "desc"
        ? b.connections.length - a.connections.length
        : a.connections.length - b.connections.length;
    }
    if (sortBy.value === "destinations") {
      return sortDirection.value === "desc"
        ? b.uniqueDestinations.length - a.uniqueDestinations.length
        : a.uniqueDestinations.length - b.uniqueDestinations.length;
    }
    if (sortBy.value === "name") {
      return sortDirection.value === "asc"
        ? a.name.localeCompare(b.name)
        : b.name.localeCompare(a.name);
    }
    return 0;
  });
});

function toggleSort(option: SortOption) {
  if (sortBy.value === option) {
    sortDirection.value = sortDirection.value === "desc" ? "asc" : "desc";
  } else {
    sortBy.value = option;
    sortDirection.value = "desc";
  }
}

function toggleAppExpand(appName: string) {
  expandedApps.value[appName] = !expandedApps.value[appName];
}

function expandAll() {
  groupedApps.value.forEach(app => {
    expandedApps.value[app.name] = true;
  });
}

function collapseAll() {
  expandedApps.value = {};
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
  copiedIp.value = text;
  setTimeout(() => {
    copiedIp.value = null;
  }, 2000);
}

function getStateBadgeVariant(state: string) {
  switch (state) {
    case "ESTABLISHED":
      return "success";
    case "LISTEN":
      return "default";
    case "TIME_WAIT":
    case "CLOSE_WAIT":
      return "warning";
    case "CLOSED":
      return "destructive";
    default:
      return "secondary";
  }
}
</script>

<template>
  <Card class="p-4 border-zinc-800 flex flex-col gap-4">
    <!-- Toolbar -->
    <div class="flex flex-col lg:flex-row items-stretch lg:items-center justify-between gap-3">
      <!-- Search Bar -->
      <div class="relative w-full lg:w-72">
        <Search class="absolute left-3 top-2.5 h-4 w-4 text-zinc-500" />
        <Input
          v-model="searchQuery"
          placeholder="Filter by app name, domain, IP, or port..."
          class="pl-9 h-9 bg-zinc-950/70"
        />
      </div>

      <!-- Filters & Sort -->
      <div class="flex flex-wrap items-center gap-2 justify-between lg:justify-end">
        <!-- View Mode: Group by App vs Flat List -->
        <div class="flex items-center bg-zinc-950 p-1 rounded-lg border border-zinc-800 text-xs">
          <button
            @click="viewMode = 'grouped'"
            :class="[
              'flex items-center gap-1.5 px-2.5 py-1 rounded-md transition-all font-medium cursor-pointer',
              viewMode === 'grouped'
                ? 'bg-cyan-600 text-white shadow-sm'
                : 'text-zinc-400 hover:text-white'
            ]"
          >
            <Layers class="w-3.5 h-3.5" />
            By App
          </button>
          <button
            @click="viewMode = 'flat'"
            :class="[
              'flex items-center gap-1.5 px-2.5 py-1 rounded-md transition-all font-medium cursor-pointer',
              viewMode === 'flat'
                ? 'bg-cyan-600 text-white shadow-sm'
                : 'text-zinc-400 hover:text-white'
            ]"
          >
            <List class="w-3.5 h-3.5" />
            All Sockets
          </button>
        </div>

        <!-- Sort Selection (Only for Grouped Mode) -->
        <div v-if="viewMode === 'grouped'" class="flex items-center bg-zinc-950 p-1 rounded-lg border border-zinc-800 text-xs">
          <span class="text-[11px] text-zinc-500 px-1 flex items-center gap-1">
            <ArrowUpDown class="w-3 h-3" /> Sort:
          </span>
          <button
            @click="toggleSort('bandwidth')"
            :class="[
              'px-2 py-1 rounded-md transition-all font-medium flex items-center gap-1 cursor-pointer',
              sortBy === 'bandwidth'
                ? 'bg-zinc-800 text-cyan-300 shadow-sm font-semibold'
                : 'text-zinc-400 hover:text-white'
            ]"
            title="Sort by Current Live Bandwidth Speed"
          >
            <Zap class="w-3 h-3 text-cyan-400" />
            Kecepatan
          </button>
          <button
            @click="toggleSort('total_data')"
            :class="[
              'px-2 py-1 rounded-md transition-all font-medium flex items-center gap-1 cursor-pointer',
              sortBy === 'total_data'
                ? 'bg-zinc-800 text-cyan-300 shadow-sm font-semibold'
                : 'text-zinc-400 hover:text-white'
            ]"
            title="Sort by Total Transferred Data"
          >
            <HardDrive class="w-3 h-3 text-emerald-400" />
            Total Data
          </button>
          <button
            @click="toggleSort('connections')"
            :class="[
              'px-2 py-1 rounded-md transition-all font-medium flex items-center gap-1 cursor-pointer',
              sortBy === 'connections'
                ? 'bg-zinc-800 text-cyan-300 shadow-sm font-semibold'
                : 'text-zinc-400 hover:text-white'
            ]"
            title="Sort by Total Sockets / Connections"
          >
            <Flame class="w-3 h-3 text-amber-400" />
            Soket
          </button>
          <button
            @click="toggleSort('name')"
            :class="[
              'px-2 py-1 rounded-md transition-all font-medium cursor-pointer',
              sortBy === 'name'
                ? 'bg-zinc-800 text-cyan-300 shadow-sm font-semibold'
                : 'text-zinc-400 hover:text-white'
            ]"
            title="Sort Alphabetically (A-Z)"
          >
            A-Z
          </button>
        </div>

        <!-- Protocol Pills -->
        <div class="flex items-center bg-zinc-950 p-1 rounded-lg border border-zinc-800 text-xs">
          <button
            v-for="proto in ['ALL', 'TCP', 'UDP']"
            :key="proto"
            @click="selectedProtocol = proto"
            :class="[
              'px-2 py-1 rounded-md transition-all font-medium cursor-pointer',
              selectedProtocol === proto
                ? 'bg-zinc-800 text-zinc-100 shadow-sm'
                : 'text-zinc-400 hover:text-white'
            ]"
          >
            {{ proto }}
          </button>
        </div>

        <!-- Expand / Collapse All -->
        <div v-if="viewMode === 'grouped'" class="flex items-center gap-1 pl-1">
          <button
            @click="expandAll"
            class="text-[11px] text-zinc-400 hover:text-cyan-400 px-1.5 py-0.5 rounded hover:bg-zinc-900 transition-colors cursor-pointer"
          >
            Expand All
          </button>
          <span class="text-zinc-600">/</span>
          <button
            @click="collapseAll"
            class="text-[11px] text-zinc-400 hover:text-cyan-400 px-1.5 py-0.5 rounded hover:bg-zinc-900 transition-colors cursor-pointer"
          >
            Collapse
          </button>
        </div>
      </div>
    </div>

    <!-- VIEW MODE 1: GROUPED BY APPLICATION BREAKDOWN (SCROLLABLE) -->
    <div
      v-if="viewMode === 'grouped'"
      class="space-y-3 max-h-[calc(100vh-340px)] min-h-[400px] overflow-y-auto pr-1"
    >
      <div
        v-for="app in groupedApps"
        :key="app.name"
        class="rounded-xl border border-zinc-800/90 bg-zinc-950/70 overflow-hidden transition-all shadow-sm hover:border-zinc-700/80"
      >
        <!-- App Header Card -->
        <div
          @click="toggleAppExpand(app.name)"
          class="p-3.5 flex flex-col sm:flex-row sm:items-center justify-between gap-3 cursor-pointer hover:bg-zinc-900/50 transition-colors select-none"
        >
          <!-- Left: App Icon & Name & PIDs -->
          <div class="flex items-center gap-3">
            <div class="text-zinc-400 transition-transform duration-200">
              <ChevronDown v-if="expandedApps[app.name]" class="w-4 h-4 text-cyan-400" />
              <ChevronRight v-else class="w-4 h-4" />
            </div>

            <div class="w-10 h-10 rounded-lg bg-cyan-950/60 border border-cyan-800/40 flex items-center justify-center text-cyan-300 shadow-sm shrink-0">
              <AppWindow class="w-5 h-5" />
            </div>

            <div>
              <div class="flex items-center gap-2">
                <span class="font-bold text-sm text-zinc-100 font-sans tracking-wide">
                  {{ app.name }}
                </span>
                <span class="text-[10px] px-1.5 py-0.5 rounded bg-zinc-800/80 text-zinc-400 font-mono">
                  PID: {{ app.pids.join(', ') }}
                </span>
              </div>
              <div class="text-xs text-zinc-400 mt-0.5 flex flex-wrap items-center gap-2">
                <span class="text-cyan-400 font-semibold">
                  {{ app.connections.length }} soket
                </span>
                <span class="text-zinc-600">•</span>
                <span class="truncate max-w-sm text-zinc-400" :title="app.uniqueDestinations.join(', ')">
                  {{ app.uniqueDestinations.length > 0 ? app.uniqueDestinations.slice(0, 3).join(', ') : 'Local ports' }}
                  <span v-if="app.uniqueDestinations.length > 3" class="text-zinc-500 font-medium">
                    +{{ app.uniqueDestinations.length - 3 }} lainnya
                  </span>
                </span>
              </div>
            </div>
          </div>

          <!-- Right: Bandwidth Speed & Total Data & Badges -->
          <div class="flex items-center justify-between sm:justify-end gap-4 shrink-0 pl-7 sm:pl-0">
            <!-- Live Speed Pill -->
            <div class="flex flex-col items-end text-xs font-mono">
              <div class="flex items-center gap-2">
                <span class="text-cyan-400 flex items-center gap-1 font-semibold">
                  <ArrowDown class="w-3 h-3" /> {{ formatSpeed(app.totalDownloadSpeed) }}
                </span>
                <span class="text-emerald-400 flex items-center gap-1 font-semibold">
                  <ArrowUp class="w-3 h-3" /> {{ formatSpeed(app.totalUploadSpeed) }}
                </span>
              </div>
              <!-- Total Data Counter -->
              <div class="text-[10px] text-zinc-400 flex items-center gap-1.5 mt-0.5">
                <span class="text-zinc-500">Total:</span>
                <span>↓ {{ formatBytes(app.totalBytesIn) }}</span>
                <span class="text-zinc-600">/</span>
                <span>↑ {{ formatBytes(app.totalBytesOut) }}</span>
              </div>
            </div>

            <!-- Status Badges -->
            <div class="flex items-center gap-1.5">
              <Badge v-if="app.activeCount > 0" variant="success">
                {{ app.activeCount }} active
              </Badge>
              <Badge v-if="app.listenCount > 0" variant="secondary">
                {{ app.listenCount }} listen
              </Badge>
            </div>

            <!-- Kill Process Button on App Header -->
            <button
              v-if="app.pids.length > 0 && app.pids[0] !== 0 && app.pids[0] !== 4"
              @click.stop="promptKillProcess(app.pids[0], app.name)"
              class="px-2.5 py-1 rounded-lg text-xs font-semibold bg-rose-950/50 hover:bg-rose-900/80 text-rose-400 border border-rose-800/60 flex items-center gap-1.5 transition-all cursor-pointer shadow-sm hover:text-white"
              title="Hentikan Proses (Kill Process)"
            >
              <Skull class="w-3.5 h-3.5" />
              <span>Kill</span>
            </button>
          </div>
        </div>

        <!-- Accordion Breakdown: Connection Destinations for this App -->
        <div
          v-if="expandedApps[app.name]"
          class="border-t border-zinc-800/80 bg-[#080a11] p-3 max-h-96 overflow-y-auto overflow-x-auto"
        >
          <table class="w-full text-left text-xs border-collapse font-mono">
            <thead>
              <tr class="text-zinc-500 text-[11px] border-b border-zinc-800/60 pb-2">
                <th class="pb-2 px-2 font-medium">Proto</th>
                <th class="pb-2 px-2 font-medium">Local Endpoint</th>
                <th class="pb-2 px-2 font-medium">Destination (Domain / IP Tujuan)</th>
                <th class="pb-2 px-2 font-medium">Port</th>
                <th class="pb-2 px-2 font-medium">Live Speed</th>
                <th class="pb-2 px-2 font-medium">Total Data</th>
                <th class="pb-2 px-2 font-medium">State</th>
                <th class="pb-2 px-2 text-right font-medium">Action</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-zinc-800/40">
              <tr
                v-for="conn in app.connections"
                :key="conn.id"
                class="hover:bg-zinc-900/40 transition-colors"
              >
                <!-- Protocol -->
                <td class="py-2 px-2">
                  <span
                    :class="[
                      'px-1.5 py-0.5 rounded text-[10px] font-bold',
                      conn.protocol === 'TCP' ? 'bg-cyan-950 text-cyan-400 border border-cyan-800/50' : 'bg-amber-950 text-amber-400 border border-amber-800/50'
                    ]"
                  >
                    {{ conn.protocol }}
                  </span>
                </td>

                <!-- Local Endpoint -->
                <td class="py-2 px-2 text-zinc-400">
                  {{ conn.local_address }}:<span class="text-zinc-300 font-semibold">{{ conn.local_port }}</span>
                </td>

                <!-- Remote Destination (Hostname / IP) -->
                <td class="py-2 px-2">
                  <div class="flex items-center gap-1.5">
                    <Globe class="w-3.5 h-3.5 text-cyan-400 shrink-0" />
                    <span class="font-semibold text-cyan-300 truncate max-w-xs" :title="conn.hostname || conn.remote_address">
                      {{ conn.hostname || conn.remote_address }}
                    </span>
                    <span v-if="conn.hostname && conn.hostname !== conn.remote_address" class="text-[10px] text-zinc-500">
                      ({{ conn.remote_address }})
                    </span>
                  </div>
                </td>

                <!-- Port -->
                <td class="py-2 px-2 text-zinc-300">
                  <span v-if="conn.remote_port > 0">{{ conn.remote_port }}</span>
                  <span v-else class="text-zinc-600">-</span>
                </td>

                <!-- Live Speed -->
                <td class="py-2 px-2">
                  <div class="flex flex-col text-[10px]">
                    <span class="text-cyan-400 flex items-center gap-0.5">
                      <ArrowDown class="w-2.5 h-2.5" /> {{ formatSpeed(conn.download_speed) }}
                    </span>
                    <span class="text-emerald-400 flex items-center gap-0.5">
                      <ArrowUp class="w-2.5 h-2.5" /> {{ formatSpeed(conn.upload_speed) }}
                    </span>
                  </div>
                </td>

                <!-- Total Bytes -->
                <td class="py-2 px-2 text-zinc-400">
                  <div class="flex flex-col text-[10px]">
                    <span>↓ {{ formatBytes(conn.bytes_received) }}</span>
                    <span>↑ {{ formatBytes(conn.bytes_sent) }}</span>
                  </div>
                </td>

                <!-- State -->
                <td class="py-2 px-2">
                  <Badge :variant="getStateBadgeVariant(conn.state)">
                    {{ conn.state }}
                  </Badge>
                </td>

                <!-- Action -->
                <td class="py-2 px-2 text-right">
                  <div class="flex items-center justify-end gap-1">
                    <!-- Block / Release Button for valid external IP -->
                    <template v-if="conn.remote_address && conn.remote_address !== '*' && conn.remote_address !== '0.0.0.0' && !conn.remote_address.startsWith('127.')">
                      <button
                        v-if="isIpBlocked(conn.remote_address)"
                        @click.stop="promptUnblockIp(conn.remote_address, conn.hostname)"
                        class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-rose-950 text-rose-300 border border-rose-800/80 hover:bg-rose-900 transition-colors flex items-center gap-1 cursor-pointer"
                        title="IP ini sedang DIBLOKIR di Firewall. Klik untuk buka blokir (Release)"
                      >
                        <ShieldAlert class="w-3 h-3 text-rose-400" />
                        <span>BLOCKED</span>
                      </button>
                      <button
                        v-else
                        @click.stop="promptBlockIp(conn.remote_address, conn.hostname, conn.process_name)"
                        class="p-1 rounded text-zinc-400 hover:text-amber-400 hover:bg-zinc-800 transition-colors cursor-pointer"
                        title="Blokir IP ini di Windows Firewall"
                      >
                        <ShieldAlert class="w-3.5 h-3.5" />
                      </button>
                    </template>

                    <button
                      v-if="conn.remote_address && conn.remote_address !== '*'"
                      @click.stop="copyToClipboard(conn.remote_address)"
                      class="p-1 rounded text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors inline-flex items-center cursor-pointer"
                      :title="copiedIp === conn.remote_address ? 'Copied!' : 'Copy IP'"
                    >
                      <Check v-if="copiedIp === conn.remote_address" class="w-3.5 h-3.5 text-emerald-400" />
                      <Copy v-else class="w-3.5 h-3.5" />
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Empty State -->
      <div v-if="groupedApps.length === 0" class="text-center py-12 text-zinc-500">
        <Radio class="w-8 h-8 mx-auto mb-2 opacity-40 animate-pulse" />
        <p>No applications match your filter.</p>
      </div>
    </div>

    <!-- VIEW MODE 2: FLAT SOCKET LIST (SCROLLABLE) -->
    <div
      v-else
      class="rounded-lg border border-zinc-800/80 overflow-hidden overflow-x-auto bg-zinc-950/40 max-h-[calc(100vh-340px)] min-h-[400px] overflow-y-auto"
    >
      <table class="w-full text-left text-xs border-collapse">
        <thead class="bg-zinc-900/60 border-b border-zinc-800 text-zinc-400 font-medium sticky top-0 backdrop-blur-md">
          <tr>
            <th class="py-2.5 px-3">Process / Application</th>
            <th class="py-2.5 px-3">Proto</th>
            <th class="py-2.5 px-3">Local Address</th>
            <th class="py-2.5 px-3">Destination (IP / Hostname)</th>
            <th class="py-2.5 px-3">Live Speed</th>
            <th class="py-2.5 px-3">Total Data</th>
            <th class="py-2.5 px-3">State</th>
            <th class="py-2.5 px-3 text-right">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-zinc-800/60 font-mono">
          <tr
            v-for="conn in filteredConnections"
            :key="conn.id"
            class="hover:bg-zinc-900/40 transition-colors"
          >
            <td class="py-2.5 px-3 font-sans font-semibold text-zinc-200">
              {{ conn.process_name }} <span class="text-[10px] text-zinc-500 font-mono">PID: {{ conn.pid }}</span>
            </td>
            <td class="py-2.5 px-3 text-cyan-400 font-bold">{{ conn.protocol }}</td>
            <td class="py-2.5 px-3 text-zinc-400">{{ conn.local_address }}:{{ conn.local_port }}</td>
            <td class="py-2.5 px-3">
              <div class="text-cyan-300 font-semibold">{{ conn.hostname || conn.remote_address }}</div>
              <div class="text-[10px] text-zinc-500">{{ conn.remote_address }}:{{ conn.remote_port }}</div>
            </td>
            <td class="py-2.5 px-3">
              <div class="flex flex-col text-[10px]">
                <span class="text-cyan-400 flex items-center gap-0.5">
                  <ArrowDown class="w-2.5 h-2.5" /> {{ formatSpeed(conn.download_speed) }}
                </span>
                <span class="text-emerald-400 flex items-center gap-0.5">
                  <ArrowUp class="w-2.5 h-2.5" /> {{ formatSpeed(conn.upload_speed) }}
                </span>
              </div>
            </td>
            <td class="py-2.5 px-3 text-zinc-400">
              <div class="flex flex-col text-[10px]">
                <span>↓ {{ formatBytes(conn.bytes_received) }}</span>
                <span>↑ {{ formatBytes(conn.bytes_sent) }}</span>
              </div>
            </td>
            <td class="py-2.5 px-3">
              <Badge :variant="getStateBadgeVariant(conn.state)">{{ conn.state }}</Badge>
            </td>
            <td class="py-2.5 px-3 text-right">
              <div class="flex items-center justify-end gap-1.5">
                <!-- Kill Process -->
                <button
                  v-if="conn.pid !== 0 && conn.pid !== 4"
                  @click.stop="promptKillProcess(conn.pid, conn.process_name)"
                  class="p-1 rounded text-zinc-400 hover:text-rose-400 hover:bg-rose-950/40 transition-colors cursor-pointer"
                  title="Hentikan Proses (Kill Process)"
                >
                  <Skull class="w-3.5 h-3.5" />
                </button>

                <!-- Block / Release IP -->
                <template v-if="conn.remote_address && conn.remote_address !== '*' && conn.remote_address !== '0.0.0.0' && !conn.remote_address.startsWith('127.')">
                  <button
                    v-if="isIpBlocked(conn.remote_address)"
                    @click.stop="promptUnblockIp(conn.remote_address, conn.hostname)"
                    class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-rose-950 text-rose-300 border border-rose-800/80 hover:bg-rose-900 transition-colors flex items-center gap-1 cursor-pointer"
                    title="Klik untuk buka blokir (Release)"
                  >
                    <ShieldAlert class="w-3 h-3 text-rose-400" />
                    <span>BLOCKED</span>
                  </button>
                  <button
                    v-else
                    @click.stop="promptBlockIp(conn.remote_address, conn.hostname, conn.process_name)"
                    class="p-1 rounded text-zinc-400 hover:text-amber-400 hover:bg-zinc-800 transition-colors cursor-pointer"
                    title="Blokir IP di Firewall"
                  >
                    <ShieldAlert class="w-3.5 h-3.5" />
                  </button>
                </template>

                <button
                  @click="copyToClipboard(conn.remote_address)"
                  class="p-1 rounded text-zinc-400 hover:text-white hover:bg-zinc-800 cursor-pointer"
                  :title="copiedIp === conn.remote_address ? 'Copied!' : 'Copy IP'"
                >
                  <Check v-if="copiedIp === conn.remote_address" class="w-3.5 h-3.5 text-emerald-400" />
                  <Copy v-else class="w-3.5 h-3.5" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Confirmation Modal -->
    <ConfirmModal
      :open="confirmModal.open"
      :title="confirmModal.title"
      :description="confirmModal.description"
      :target-name="confirmModal.targetName"
      :target-detail="confirmModal.targetDetail"
      :action-type="confirmModal.actionType"
      :confirm-text="confirmModal.confirmText"
      @confirm="executeConfirmedAction"
      @close="confirmModal.open = false"
    />
  </Card>
</template>
