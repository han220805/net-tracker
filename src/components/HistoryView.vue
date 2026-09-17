<script setup lang="ts">
import { ref, computed } from "vue";
import { Search, Trash2, Globe, Clock, AppWindow, Check, Copy, CalendarDays, ShieldAlert } from "lucide-vue-next";
import Card from "@/components/ui/Card.vue";
import Input from "@/components/ui/Input.vue";
import Button from "@/components/ui/Button.vue";
import ConfirmModal from "@/components/ui/ConfirmModal.vue";
import { formatBytes } from "@/lib/utils";
import type { HistoryRecord, BlockedIpRecord } from "@/types/network";

const props = defineProps<{
  history: HistoryRecord[];
  blockedIps?: BlockedIpRecord[];
}>();

const emit = defineEmits<{
  (e: "clear-history"): void;
  (e: "block-ip", payload: { ip: string; hostname?: string; processName?: string }): void;
  (e: "unblock-ip", payload: { ip: string }): void;
}>();

const historySearch = ref("");
const selectedAppFilter = ref<string>("ALL");
const copiedIp = ref<string | null>(null);

function isIpBlocked(ip: string): boolean {
  if (!props.blockedIps || !ip) return false;
  return props.blockedIps.some((b) => b.ip === ip);
}

// Confirmation Dialog State
const confirmModal = ref<{
  open: boolean;
  type: "block" | "unblock";
  title: string;
  description: string;
  targetName: string;
  targetDetail?: string;
  actionType: "warning" | "success";
  confirmText: string;
  payload: any;
}>({
  open: false,
  type: "block",
  title: "",
  description: "",
  targetName: "",
  targetDetail: "",
  actionType: "warning",
  confirmText: "",
  payload: null,
});

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

  if (m.type === "block") {
    emit("block-ip", m.payload);
  } else if (m.type === "unblock") {
    emit("unblock-ip", m.payload);
  }
}

// ─── Time range filter ─────────────────────────────────────────────────────────
const TIME_RANGES = [
  { label: "1 Jam",   value: "1h",  ms: 1 * 60 * 60 * 1000 },
  { label: "6 Jam",   value: "6h",  ms: 6 * 60 * 60 * 1000 },
  { label: "1 Hari",  value: "1d",  ms: 24 * 60 * 60 * 1000 },
  { label: "3 Hari",  value: "3d",  ms: 3 * 24 * 60 * 60 * 1000 },
  { label: "1 Minggu",value: "1w",  ms: 7 * 24 * 60 * 60 * 1000 },
  { label: "1 Bulan", value: "1mo", ms: 30 * 24 * 60 * 60 * 1000 },
  { label: "Semua",   value: "all", ms: 0 },
] as const;

const selectedTimeRange = ref<string>("all");

// Unique applications from history for quick filter
const uniqueApps = computed(() => {
  const apps = new Set<string>();
  props.history.forEach((h) => {
    if (h.process_name) apps.add(h.process_name);
  });
  return Array.from(apps).sort();
});

// Apply time range → app filter → search query
const filteredHistory = computed(() => {
  const now = Date.now();
  const range = TIME_RANGES.find((r) => r.value === selectedTimeRange.value);
  const cutoff = range && range.ms > 0 ? now - range.ms : 0;

  let list = props.history;

  // 1. Time range filter
  if (cutoff > 0) {
    list = list.filter((h) => h.timestamp >= cutoff);
  }

  // 2. App filter
  if (selectedAppFilter.value !== "ALL") {
    list = list.filter((h) => h.process_name === selectedAppFilter.value);
  }

  // 3. Search query
  if (!historySearch.value.trim()) return list;
  const q = historySearch.value.toLowerCase();
  return list.filter(
    (h) =>
      h.hostname.toLowerCase().includes(q) ||
      h.remote_address.includes(q) ||
      h.process_name.toLowerCase().includes(q) ||
      String(h.remote_port).includes(q)
  );
});

// Calculate Top Domains from **filtered** history (respects time range + app filter)
const topDomains = computed(() => {
  const counts: Record<string, { count: number; bytes: number; apps: Set<string> }> = {};
  filteredHistory.value.forEach((h) => {
    const key = h.hostname || h.remote_address;
    if (!counts[key]) {
      counts[key] = { count: 0, bytes: 0, apps: new Set<string>() };
    }
    counts[key].count++;
    counts[key].bytes += h.total_bytes;
    if (h.process_name) counts[key].apps.add(h.process_name);
  });

  return Object.entries(counts)
    .sort((a, b) => b[1].bytes - a[1].bytes)
    .slice(0, 5)
    .map(([domain, data]) => ({
      domain,
      count: data.count,
      bytes: data.bytes,
      apps: Array.from(data.apps),
    }));
});

function formatTime(ts: number) {
  const d = new Date(ts);
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

function formatDate(ts: number) {
  const d = new Date(ts);
  return d.toLocaleDateString([], { month: 'short', day: 'numeric' });
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
  copiedIp.value = text;
  setTimeout(() => {
    copiedIp.value = null;
  }, 2000);
}
</script>

<template>
  <div class="flex flex-col gap-4">

    <!-- Time Range Filter Bar -->
    <div class="flex items-center gap-2 flex-wrap">
      <div class="flex items-center gap-1.5 text-xs text-zinc-400 mr-1">
        <CalendarDays class="w-3.5 h-3.5 text-cyan-400" />
        <span class="font-medium">Rentang:</span>
      </div>
      <button
        v-for="range in TIME_RANGES"
        :key="range.value"
        @click="selectedTimeRange = range.value"
        :class="[
          'px-3 py-1 rounded-lg text-xs font-medium border transition-all cursor-pointer',
          selectedTimeRange === range.value
            ? 'bg-cyan-600 border-cyan-600 text-white shadow-md shadow-cyan-500/20'
            : 'bg-zinc-900/60 border-zinc-800 text-zinc-400 hover:bg-zinc-800 hover:text-white'
        ]"
      >
        {{ range.label }}
      </button>

      <!-- Record count badge -->
      <span class="ml-auto text-xs text-zinc-500 font-mono">
        <span class="text-cyan-400 font-bold">{{ filteredHistory.length }}</span>
        / {{ history.length }} record
      </span>
    </div>

    <!-- Top Destination Insights -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <Card class="p-4 border-zinc-800 md:col-span-2">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <Globe class="w-4 h-4 text-cyan-400" />
            <span class="text-xs font-semibold uppercase tracking-wider text-zinc-300">Top Visited Hostnames & Data</span>
          </div>
          <span class="text-xs text-zinc-500 font-mono">By total bandwidth</span>
        </div>

        <div class="space-y-2.5">
          <div
            v-for="item in topDomains"
            :key="item.domain"
            class="p-2.5 rounded-lg bg-zinc-950/60 border border-zinc-800/80 hover:border-zinc-700/80 transition-all text-xs flex flex-col sm:flex-row sm:items-center justify-between gap-2"
          >
            <div class="flex flex-col gap-1 min-w-0">
              <div class="flex items-center gap-2 min-w-0">
                <span class="w-2 h-2 rounded-full bg-cyan-400 shrink-0"></span>
                <span class="font-mono text-cyan-300 font-semibold truncate">{{ item.domain }}</span>
              </div>
              <!-- Applications using this domain -->
              <div class="flex items-center gap-1.5 flex-wrap pl-4">
                <span class="text-[10px] text-zinc-500 font-medium">Accessed by:</span>
                <span
                  v-for="app in item.apps"
                  :key="app"
                  class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-zinc-900 border border-zinc-800 text-[10px] text-zinc-300 font-medium font-sans"
                >
                  <AppWindow class="w-2.5 h-2.5 text-cyan-400 shrink-0" />
                  {{ app }}
                </span>
              </div>
            </div>

            <div class="flex items-center gap-4 text-zinc-400 font-mono shrink-0 pl-4 sm:pl-0">
              <span class="text-[11px] bg-zinc-900 px-2 py-0.5 rounded border border-zinc-800 text-zinc-400">{{ item.count }} hits</span>
              <span class="text-cyan-400 font-bold text-xs">{{ formatBytes(item.bytes) }}</span>
            </div>
          </div>

          <div v-if="topDomains.length === 0" class="text-zinc-500 text-xs py-4 text-center">
            Tidak ada data pada rentang waktu yang dipilih.
          </div>
        </div>
      </Card>

      <Card class="p-4 border-zinc-800 flex flex-col justify-between">
        <div>
          <div class="flex items-center gap-2 mb-2">
            <Clock class="w-4 h-4 text-purple-400" />
            <span class="text-xs font-semibold uppercase tracking-wider text-zinc-300">Database Persistence</span>
          </div>
          <p class="text-xs text-zinc-400 leading-relaxed mb-3">
            All outbound socket events and DNS queries are stored locally in SQLite database for forensics and network auditing.
          </p>

          <div class="p-3 rounded-lg bg-zinc-950/70 border border-zinc-800/80 space-y-2 text-xs">
            <div class="flex justify-between text-zinc-400">
              <span>Total Recorded Events:</span>
              <span class="font-mono text-zinc-200 font-semibold">{{ history.length }}</span>
            </div>
            <div class="flex justify-between text-zinc-400">
              <span>Filtered Events:</span>
              <span class="font-mono text-cyan-400 font-semibold">{{ filteredHistory.length }}</span>
            </div>
            <div class="flex justify-between text-zinc-400">
              <span>Distinct Apps Logged:</span>
              <span class="font-mono text-cyan-400 font-semibold">{{ uniqueApps.length }}</span>
            </div>
          </div>
        </div>

        <div class="mt-4 pt-4 border-t border-zinc-800 flex items-center justify-between">
          <span class="text-[11px] text-zinc-500">Local SQLite</span>
          <Button variant="destructive" size="sm" @click="emit('clear-history')">
            <Trash2 class="w-3.5 h-3.5" /> Clear History
          </Button>
        </div>
      </Card>
    </div>

    <!-- History Table -->
    <Card class="p-4 border-zinc-800">
      <div class="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 mb-4">
        <div class="flex items-center gap-2 flex-1 max-w-lg">
          <div class="relative flex-1">
            <Search class="absolute left-3 top-2.5 h-4 w-4 text-zinc-500" />
            <Input
              v-model="historySearch"
              placeholder="Search by app, domain, IP, or port..."
              class="pl-9 h-9 bg-zinc-950/70"
            />
          </div>

          <!-- Application Filter Selector -->
          <select
            v-model="selectedAppFilter"
            class="h-9 px-2.5 py-1 text-xs bg-zinc-900 border border-zinc-800 rounded-md text-zinc-300 focus:outline-none focus:border-cyan-500"
          >
            <option value="ALL">All Apps ({{ uniqueApps.length }})</option>
            <option v-for="app in uniqueApps" :key="app" :value="app">
              {{ app }}
            </option>
          </select>
        </div>

        <div class="text-xs text-zinc-400">
          Showing <span class="font-mono text-cyan-400 font-bold">{{ filteredHistory.length }}</span> logged sessions
        </div>
      </div>

      <div class="rounded-lg border border-zinc-800/80 overflow-hidden overflow-x-auto bg-zinc-950/40">
        <table class="w-full text-left text-xs border-collapse">
          <thead class="bg-zinc-900/80 border-b border-zinc-800 text-zinc-400 font-medium">
            <tr>
              <th class="py-2.5 px-3">Time</th>
              <th class="py-2.5 px-3">Application (Caller)</th>
              <th class="py-2.5 px-3">Protocol</th>
              <th class="py-2.5 px-3">Destination Hostname & IP</th>
              <th class="py-2.5 px-3">Port</th>
              <th class="py-2.5 px-3 text-right">Data Exchanged</th>
              <th class="py-2.5 px-3 text-right">Action</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-zinc-800/60 font-mono">
            <tr
              v-for="record in filteredHistory"
              :key="record.id"
              class="hover:bg-zinc-900/50 transition-colors"
            >
              <!-- Time Column -->
              <td class="py-2.5 px-3 text-zinc-400 whitespace-nowrap">
                <div class="text-zinc-300">{{ formatTime(record.timestamp) }}</div>
                <div class="text-[10px] text-zinc-500 font-sans">{{ formatDate(record.timestamp) }}</div>
              </td>

              <!-- Application / Process Column (Highlighted) -->
              <td class="py-2.5 px-3 whitespace-nowrap font-sans">
                <div class="inline-flex items-center gap-2 px-2.5 py-1 rounded-md bg-zinc-900/90 border border-zinc-800 text-zinc-200 shadow-sm">
                  <div class="w-5 h-5 rounded bg-gradient-to-tr from-cyan-600/30 to-blue-600/30 border border-cyan-500/30 flex items-center justify-center">
                    <AppWindow class="w-3 h-3 text-cyan-400" />
                  </div>
                  <span class="font-semibold text-xs text-white">{{ record.process_name || "Unknown App" }}</span>
                </div>
              </td>

              <!-- Protocol Column -->
              <td class="py-2.5 px-3 whitespace-nowrap">
                <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
                  {{ record.protocol }}
                </span>
              </td>

              <!-- Destination Hostname / IP Column -->
              <td class="py-2.5 px-3">
                <div class="flex items-center gap-1.5">
                  <span class="text-cyan-300 font-semibold">{{ record.hostname || record.remote_address }}</span>
                  <button
                    @click="copyToClipboard(record.remote_address)"
                    class="text-zinc-500 hover:text-white p-0.5 rounded hover:bg-zinc-800 transition-colors"
                    title="Copy IP Address"
                  >
                    <Check v-if="copiedIp === record.remote_address" class="w-3 h-3 text-emerald-400" />
                    <Copy v-else class="w-3 h-3" />
                  </button>
                </div>
                <div class="text-[10px] text-zinc-500 font-mono">{{ record.remote_address }}</div>
              </td>

              <!-- Port Column -->
              <td class="py-2.5 px-3 text-zinc-400 whitespace-nowrap">
                <span class="text-zinc-300 font-medium">{{ record.remote_port }}</span>
              </td>

              <!-- Data Exchanged Column -->
              <td class="py-2.5 px-3 text-right whitespace-nowrap font-semibold text-cyan-400">
                {{ formatBytes(record.total_bytes) }}
              </td>

              <!-- Action Column -->
              <td class="py-2.5 px-3 text-right whitespace-nowrap">
                <template v-if="record.remote_address && record.remote_address !== '*' && record.remote_address !== '0.0.0.0' && !record.remote_address.startsWith('127.')">
                  <button
                    v-if="isIpBlocked(record.remote_address)"
                    @click.stop="promptUnblockIp(record.remote_address, record.hostname)"
                    class="px-2 py-0.5 rounded text-[10px] font-bold bg-rose-950 text-rose-300 border border-rose-800/80 hover:bg-rose-900 transition-colors inline-flex items-center gap-1 cursor-pointer"
                    title="IP ini sedang DIBLOKIR. Klik untuk buka blokir (Release)"
                  >
                    <ShieldAlert class="w-3 h-3 text-rose-400" />
                    <span>BLOCKED</span>
                  </button>
                  <button
                    v-else
                    @click.stop="promptBlockIp(record.remote_address, record.hostname, record.process_name)"
                    class="p-1 rounded text-zinc-400 hover:text-amber-400 hover:bg-zinc-800 transition-colors cursor-pointer inline-flex items-center"
                    title="Blokir IP ini di Windows Defender Firewall"
                  >
                    <ShieldAlert class="w-3.5 h-3.5" />
                  </button>
                </template>
              </td>
            </tr>
            <tr v-if="filteredHistory.length === 0">
              <td colspan="7" class="text-center py-10 text-zinc-500 font-sans">
                Tidak ada record pada rentang waktu yang dipilih.
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
  </div>
</template>
