<script setup lang="ts">
import { ref, computed } from "vue";
import {
  Search,
  Copy,
  Check,
  Globe,
  Radio,
  ArrowDown,
  ArrowUp,
  Server,
} from "lucide-vue-next";
import Badge from "@/components/ui/Badge.vue";
import Input from "@/components/ui/Input.vue";
import Card from "@/components/ui/Card.vue";
import { formatBytes, formatSpeed } from "@/lib/utils";
import type { NetworkConnection } from "@/types/network";

const props = defineProps<{
  connections: NetworkConnection[];
  isLoading?: boolean;
}>();

const searchQuery = ref("");
const selectedProtocol = ref<string>("ALL");
const selectedState = ref<string>("ALL");
const copiedIp = ref<string | null>(null);

const filteredConnections = computed(() => {
  return props.connections.filter((conn) => {
    // Protocol filter
    if (selectedProtocol.value !== "ALL" && conn.protocol !== selectedProtocol.value) {
      return false;
    }

    // State filter
    if (selectedState.value !== "ALL" && conn.state !== selectedState.value) {
      return false;
    }

    // Search query filter (Process, Remote Address, Hostname, Port)
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
    <div class="flex flex-col sm:flex-row items-center justify-between gap-3">
      <!-- Search Bar -->
      <div class="relative w-full sm:w-80">
        <Search class="absolute left-3 top-2.5 h-4 w-4 text-zinc-500" />
        <Input
          v-model="searchQuery"
          placeholder="Filter by app, IP, hostname, or port..."
          class="pl-9 h-9 bg-zinc-950/70"
        />
      </div>

      <!-- Filters -->
      <div class="flex items-center gap-2 w-full sm:w-auto justify-between sm:justify-end">
        <!-- Protocol Pills -->
        <div class="flex items-center bg-zinc-950 p-1 rounded-lg border border-zinc-800 text-xs">
          <button
            v-for="proto in ['ALL', 'TCP', 'UDP']"
            :key="proto"
            @click="selectedProtocol = proto"
            :class="[
              'px-2.5 py-1 rounded-md transition-all font-medium',
              selectedProtocol === proto
                ? 'bg-cyan-600 text-white shadow-sm'
                : 'text-zinc-400 hover:text-white'
            ]"
          >
            {{ proto }}
          </button>
        </div>

        <!-- State Pills -->
        <div class="flex items-center bg-zinc-950 p-1 rounded-lg border border-zinc-800 text-xs">
          <button
            v-for="st in ['ALL', 'ESTABLISHED', 'LISTEN']"
            :key="st"
            @click="selectedState = st"
            :class="[
              'px-2.5 py-1 rounded-md transition-all font-medium',
              selectedState === st
                ? 'bg-zinc-800 text-zinc-100 shadow-sm'
                : 'text-zinc-400 hover:text-white'
            ]"
          >
            {{ st === 'ESTABLISHED' ? 'Active' : st === 'LISTEN' ? 'Listen' : 'All' }}
          </button>
        </div>

        <span class="text-xs text-zinc-500 font-mono pl-2">
          {{ filteredConnections.length }} / {{ connections.length }}
        </span>
      </div>
    </div>

    <!-- Table Container -->
    <div class="rounded-lg border border-zinc-800/80 overflow-hidden overflow-x-auto bg-zinc-950/40">
      <table class="w-full text-left text-xs border-collapse">
        <thead class="bg-zinc-900/60 border-b border-zinc-800 text-zinc-400 font-medium">
          <tr>
            <th class="py-2.5 px-3">Process / Application</th>
            <th class="py-2.5 px-3">Proto</th>
            <th class="py-2.5 px-3">Local Address</th>
            <th class="py-2.5 px-3">Remote Destination (IP / Hostname)</th>
            <th class="py-2.5 px-3">Live Rate</th>
            <th class="py-2.5 px-3">Total Data</th>
            <th class="py-2.5 px-3">State</th>
            <th class="py-2.5 px-3 text-right">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-zinc-800/60 font-mono">
          <tr
            v-for="conn in filteredConnections"
            :key="conn.id"
            class="hover:bg-zinc-900/40 transition-colors group"
          >
            <!-- Process & PID -->
            <td class="py-2.5 px-3 font-sans">
              <div class="flex items-center gap-2">
                <div class="p-1 rounded bg-zinc-800/70 border border-zinc-700/60 text-zinc-300">
                  <Server class="w-3.5 h-3.5 text-cyan-400" />
                </div>
                <div>
                  <div class="font-semibold text-zinc-200 flex items-center gap-1.5">
                    {{ conn.process_name }}
                  </div>
                  <div class="text-[10px] text-zinc-500">PID: {{ conn.pid }}</div>
                </div>
              </div>
            </td>

            <!-- Protocol -->
            <td class="py-2.5 px-3">
              <span
                :class="[
                  'px-1.5 py-0.5 rounded text-[10px] font-bold',
                  conn.protocol === 'TCP' ? 'bg-cyan-950 text-cyan-400 border border-cyan-800/50' : 'bg-amber-950 text-amber-400 border border-amber-800/50'
                ]"
              >
                {{ conn.protocol }}
              </span>
            </td>

            <!-- Local Address -->
            <td class="py-2.5 px-3 text-zinc-400">
              {{ conn.local_address }}:<span class="text-zinc-300 font-semibold">{{ conn.local_port }}</span>
            </td>

            <!-- Remote Destination (Hostname + IP) -->
            <td class="py-2.5 px-3">
              <div class="flex flex-col">
                <div class="text-cyan-300 font-semibold flex items-center gap-1.5">
                  <Globe class="w-3 h-3 text-cyan-400 shrink-0" />
                  <span class="truncate max-w-[240px]" :title="conn.hostname">{{ conn.hostname || conn.remote_address }}</span>
                </div>
                <div class="text-[10px] text-zinc-500 flex items-center gap-1">
                  <span>{{ conn.remote_address }}:{{ conn.remote_port }}</span>
                  <span v-if="conn.country" class="text-zinc-400">({{ conn.country }})</span>
                </div>
              </div>
            </td>

            <!-- Live Rate -->
            <td class="py-2.5 px-3">
              <div class="flex flex-col gap-0.5 text-[11px]">
                <span class="text-cyan-400 flex items-center gap-1">
                  <ArrowDown class="w-3 h-3" /> {{ formatSpeed(conn.download_speed) }}
                </span>
                <span class="text-emerald-400 flex items-center gap-1">
                  <ArrowUp class="w-3 h-3" /> {{ formatSpeed(conn.upload_speed) }}
                </span>
              </div>
            </td>

            <!-- Total Bytes -->
            <td class="py-2.5 px-3 text-zinc-400">
              <div class="flex flex-col text-[10px]">
                <span>↓ {{ formatBytes(conn.bytes_received) }}</span>
                <span>↑ {{ formatBytes(conn.bytes_sent) }}</span>
              </div>
            </td>

            <!-- State -->
            <td class="py-2.5 px-3">
              <Badge :variant="getStateBadgeVariant(conn.state)">
                {{ conn.state }}
              </Badge>
            </td>

            <!-- Action -->
            <td class="py-2.5 px-3 text-right">
              <div class="flex items-center justify-end gap-1">
                <button
                  @click="copyToClipboard(conn.remote_address)"
                  class="p-1 rounded text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors"
                  :title="copiedIp === conn.remote_address ? 'Copied!' : 'Copy Remote IP'"
                >
                  <Check v-if="copiedIp === conn.remote_address" class="w-3.5 h-3.5 text-emerald-400" />
                  <Copy v-else class="w-3.5 h-3.5" />
                </button>
              </div>
            </td>
          </tr>

          <!-- Empty State -->
          <tr v-if="filteredConnections.length === 0">
            <td colspan="8" class="text-center py-10 text-zinc-500 font-sans">
              <Radio class="w-8 h-8 mx-auto mb-2 opacity-40 animate-pulse" />
              <p>No active network connections match your search filter.</p>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </Card>
</template>
