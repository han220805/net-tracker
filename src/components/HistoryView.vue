<script setup lang="ts">
import { ref, computed } from "vue";
import { Search, Trash2, Globe, Clock } from "lucide-vue-next";
import Card from "@/components/ui/Card.vue";
import Input from "@/components/ui/Input.vue";
import Button from "@/components/ui/Button.vue";
import { formatBytes } from "@/lib/utils";
import type { HistoryRecord } from "@/types/network";

const props = defineProps<{
  history: HistoryRecord[];
}>();

const emit = defineEmits<{
  (e: "clear-history"): void;
}>();

const historySearch = ref("");

const filteredHistory = computed(() => {
  if (!historySearch.value.trim()) return props.history;
  const q = historySearch.value.toLowerCase();
  return props.history.filter(
    (h) =>
      h.hostname.toLowerCase().includes(q) ||
      h.remote_address.includes(q) ||
      h.process_name.toLowerCase().includes(q)
  );
});

// Calculate Top Domains from History
const topDomains = computed(() => {
  const counts: Record<string, { count: number; bytes: number }> = {};
  props.history.forEach((h) => {
    const key = h.hostname || h.remote_address;
    if (!counts[key]) counts[key] = { count: 0, bytes: 0 };
    counts[key].count++;
    counts[key].bytes += h.total_bytes;
  });

  return Object.entries(counts)
    .sort((a, b) => b[1].bytes - a[1].bytes)
    .slice(0, 5)
    .map(([domain, data]) => ({ domain, ...data }));
});

function formatTime(ts: number) {
  const d = new Date(ts);
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}
</script>

<template>
  <div class="flex flex-col gap-4">
    <!-- Top Destination Insights -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <Card class="p-4 border-zinc-800 md:col-span-2">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <Globe class="w-4 h-4 text-cyan-400" />
            <span class="text-xs font-semibold uppercase tracking-wider text-zinc-300">Top Visited Hostnames & Data</span>
          </div>
          <span class="text-xs text-zinc-500">By total bandwidth</span>
        </div>
        <div class="space-y-2">
          <div
            v-for="item in topDomains"
            :key="item.domain"
            class="flex items-center justify-between p-2 rounded-lg bg-zinc-950/60 border border-zinc-800/80 text-xs"
          >
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-cyan-400"></span>
              <span class="font-mono text-zinc-200 font-medium truncate max-w-xs">{{ item.domain }}</span>
            </div>
            <div class="flex items-center gap-4 text-zinc-400 font-mono">
              <span>{{ item.count }} hits</span>
              <span class="text-cyan-400 font-semibold">{{ formatBytes(item.bytes) }}</span>
            </div>
          </div>
          <div v-if="topDomains.length === 0" class="text-zinc-500 text-xs py-4 text-center">
            No history collected yet. Start using the network to populate records.
          </div>
        </div>
      </Card>

      <Card class="p-4 border-zinc-800 flex flex-col justify-between">
        <div>
          <div class="flex items-center gap-2 mb-2">
            <Clock class="w-4 h-4 text-purple-400" />
            <span class="text-xs font-semibold uppercase tracking-wider text-zinc-300">Database Persistence</span>
          </div>
          <p class="text-xs text-zinc-400 leading-relaxed">
            All outbound socket events and DNS queries are stored locally in SQLite database for forensics and network auditing.
          </p>
        </div>
        <div class="mt-4 pt-4 border-t border-zinc-800 flex items-center justify-between">
          <span class="text-xs text-zinc-500 font-mono">{{ history.length }} total events</span>
          <Button variant="destructive" size="sm" @click="emit('clear-history')">
            <Trash2 class="w-3.5 h-3.5" /> Clear History
          </Button>
        </div>
      </Card>
    </div>

    <!-- History Table -->
    <Card class="p-4 border-zinc-800">
      <div class="flex items-center justify-between gap-4 mb-4">
        <div class="relative w-full sm:w-80">
          <Search class="absolute left-3 top-2.5 h-4 w-4 text-zinc-500" />
          <Input
            v-model="historySearch"
            placeholder="Search history by hostname, IP, app..."
            class="pl-9 h-9 bg-zinc-950/70"
          />
        </div>
        <div class="text-xs text-zinc-400">
          Showing <span class="font-mono text-white">{{ filteredHistory.length }}</span> logged sessions
        </div>
      </div>

      <div class="rounded-lg border border-zinc-800/80 overflow-hidden overflow-x-auto bg-zinc-950/40">
        <table class="w-full text-left text-xs border-collapse">
          <thead class="bg-zinc-900/60 border-b border-zinc-800 text-zinc-400 font-medium">
            <tr>
              <th class="py-2.5 px-3">Time</th>
              <th class="py-2.5 px-3">Application</th>
              <th class="py-2.5 px-3">Protocol</th>
              <th class="py-2.5 px-3">Destination Hostname / IP</th>
              <th class="py-2.5 px-3">Port</th>
              <th class="py-2.5 px-3">Duration</th>
              <th class="py-2.5 px-3 text-right">Data Exchanged</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-zinc-800/60 font-mono">
            <tr
              v-for="record in filteredHistory"
              :key="record.id"
              class="hover:bg-zinc-900/40 transition-colors"
            >
              <td class="py-2.5 px-3 text-zinc-400">{{ formatTime(record.timestamp) }}</td>
              <td class="py-2.5 px-3 font-sans font-semibold text-zinc-200">{{ record.process_name }}</td>
              <td class="py-2.5 px-3">
                <span class="text-[10px] font-bold text-cyan-400">{{ record.protocol }}</span>
              </td>
              <td class="py-2.5 px-3">
                <div class="text-cyan-300 font-semibold">{{ record.hostname || record.remote_address }}</div>
                <div class="text-[10px] text-zinc-500">{{ record.remote_address }}</div>
              </td>
              <td class="py-2.5 px-3 text-zinc-400">{{ record.remote_port }}</td>
              <td class="py-2.5 px-3 text-zinc-400">{{ record.duration_seconds }}s</td>
              <td class="py-2.5 px-3 text-right text-cyan-400 font-semibold">
                {{ formatBytes(record.total_bytes) }}
              </td>
            </tr>
            <tr v-if="filteredHistory.length === 0">
              <td colspan="7" class="text-center py-8 text-zinc-500 font-sans">
                No history entries recorded yet.
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </Card>
  </div>
</template>
