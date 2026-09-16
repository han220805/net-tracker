<script setup lang="ts">
import { ArrowDown, ArrowUp, Activity, Layers } from "lucide-vue-next";
import { formatSpeed, formatBytes } from "@/lib/utils";
import Card from "@/components/ui/Card.vue";
import type { NetworkSummary } from "@/types/network";

defineProps<{
  summary: NetworkSummary;
}>();
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
    <!-- Download Speed Card -->
    <Card class="p-4 relative overflow-hidden group hover:border-cyan-500/50 transition-all">
      <div class="absolute -right-4 -top-4 w-20 h-20 bg-cyan-500/10 rounded-full blur-xl group-hover:bg-cyan-500/20 transition-all"></div>
      <div class="flex items-center justify-between">
        <span class="text-xs font-medium text-zinc-400 uppercase tracking-wider">Download Speed</span>
        <div class="p-1.5 rounded-md bg-cyan-500/15 text-cyan-400">
          <ArrowDown class="w-4 h-4 animate-bounce" />
        </div>
      </div>
      <div class="mt-2 flex items-baseline gap-2">
        <span class="text-2xl font-bold tracking-tight text-white font-mono">
          {{ formatSpeed(summary.total_download_speed) }}
        </span>
      </div>
      <div class="mt-1 text-xs text-zinc-500">
        Total in: <span class="font-mono text-zinc-400">{{ formatBytes(summary.total_bytes_received) }}</span>
      </div>
    </Card>

    <!-- Upload Speed Card -->
    <Card class="p-4 relative overflow-hidden group hover:border-emerald-500/50 transition-all">
      <div class="absolute -right-4 -top-4 w-20 h-20 bg-emerald-500/10 rounded-full blur-xl group-hover:bg-emerald-500/20 transition-all"></div>
      <div class="flex items-center justify-between">
        <span class="text-xs font-medium text-zinc-400 uppercase tracking-wider">Upload Speed</span>
        <div class="p-1.5 rounded-md bg-emerald-500/15 text-emerald-400">
          <ArrowUp class="w-4 h-4 animate-bounce" />
        </div>
      </div>
      <div class="mt-2 flex items-baseline gap-2">
        <span class="text-2xl font-bold tracking-tight text-white font-mono">
          {{ formatSpeed(summary.total_upload_speed) }}
        </span>
      </div>
      <div class="mt-1 text-xs text-zinc-500">
        Total out: <span class="font-mono text-zinc-400">{{ formatBytes(summary.total_bytes_sent) }}</span>
      </div>
    </Card>

    <!-- Active Connections Card -->
    <Card class="p-4 relative overflow-hidden group hover:border-blue-500/50 transition-all">
      <div class="absolute -right-4 -top-4 w-20 h-20 bg-blue-500/10 rounded-full blur-xl group-hover:bg-blue-500/20 transition-all"></div>
      <div class="flex items-center justify-between">
        <span class="text-xs font-medium text-zinc-400 uppercase tracking-wider">Active Sockets</span>
        <div class="p-1.5 rounded-md bg-blue-500/15 text-blue-400">
          <Activity class="w-4 h-4" />
        </div>
      </div>
      <div class="mt-2 flex items-baseline gap-2">
        <span class="text-2xl font-bold tracking-tight text-white font-mono">
          {{ summary.total_active_connections }}
        </span>
        <span class="text-xs text-zinc-400">established</span>
      </div>
      <div class="mt-1 text-xs text-zinc-500">
        Listening ports: <span class="font-mono text-zinc-400">{{ summary.total_listening_ports }}</span>
      </div>
    </Card>

    <!-- Unique Processes Card -->
    <Card class="p-4 relative overflow-hidden group hover:border-purple-500/50 transition-all">
      <div class="absolute -right-4 -top-4 w-20 h-20 bg-purple-500/10 rounded-full blur-xl group-hover:bg-purple-500/20 transition-all"></div>
      <div class="flex items-center justify-between">
        <span class="text-xs font-medium text-zinc-400 uppercase tracking-wider">Active Apps</span>
        <div class="p-1.5 rounded-md bg-purple-500/15 text-purple-400">
          <Layers class="w-4 h-4" />
        </div>
      </div>
      <div class="mt-2 flex items-baseline gap-2">
        <span class="text-2xl font-bold tracking-tight text-white font-mono">
          {{ summary.total_processes }}
        </span>
        <span class="text-xs text-zinc-400">executables</span>
      </div>
      <div class="mt-1 text-xs text-emerald-400 flex items-center gap-1">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-ping"></span>
        Kernel monitor online
      </div>
    </Card>
  </div>
</template>
