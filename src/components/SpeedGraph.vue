<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import Card from "@/components/ui/Card.vue";
import { formatSpeed } from "@/lib/utils";

const props = defineProps<{
  downloadSpeed: number;
  uploadSpeed: number;
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const historyLength = 40;
const downloadHistory = ref<number[]>(new Array(historyLength).fill(0));
const uploadHistory = ref<number[]>(new Array(historyLength).fill(0));

function updateData() {
  downloadHistory.value.push(props.downloadSpeed);
  downloadHistory.value.shift();

  uploadHistory.value.push(props.uploadSpeed);
  uploadHistory.value.shift();
}

function draw() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const w = canvas.width;
  const h = canvas.height;

  ctx.clearRect(0, 0, w, h);

  // Find max value to scale graph
  const maxVal = Math.max(
    ...downloadHistory.value,
    ...uploadHistory.value,
    1024 * 50 // Minimum 50 KB/s ceiling
  );

  // Draw grid lines
  ctx.strokeStyle = "rgba(255, 255, 255, 0.05)";
  ctx.lineWidth = 1;
  for (let i = 1; i <= 3; i++) {
    const y = (h / 4) * i;
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(w, y);
    ctx.stroke();
  }

  // Helper to draw series
  const drawSeries = (data: number[], strokeColor: string, fillColor: string | CanvasGradient) => {
    ctx.beginPath();
    const step = w / (historyLength - 1);

    for (let i = 0; i < data.length; i++) {
      const x = i * step;
      const y = h - (data[i] / maxVal) * (h - 10) - 5;
      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    }

    ctx.strokeStyle = strokeColor;
    ctx.lineWidth = 2;
    ctx.stroke();

    // Fill gradient
    ctx.lineTo(w, h);
    ctx.lineTo(0, h);
    ctx.closePath();
    ctx.fillStyle = fillColor;
    ctx.fill();
  };

  // Draw Download (Cyan)
  const cyanGrad = ctx.createLinearGradient(0, 0, 0, h);
  cyanGrad.addColorStop(0, "rgba(6, 182, 212, 0.25)");
  cyanGrad.addColorStop(1, "rgba(6, 182, 212, 0.0)");
  drawSeries(downloadHistory.value, "#06b6d4", cyanGrad);

  // Draw Upload (Emerald)
  const emeraldGrad = ctx.createLinearGradient(0, 0, 0, h);
  emeraldGrad.addColorStop(0, "rgba(16, 185, 129, 0.2)");
  emeraldGrad.addColorStop(1, "rgba(16, 185, 129, 0.0)");
  drawSeries(uploadHistory.value, "#10b981", emeraldGrad);
}

let intervalTimer: ReturnType<typeof setInterval> | null = null;

onMounted(() => {
  if (canvasRef.value) {
    canvasRef.value.width = canvasRef.value.parentElement?.clientWidth || 600;
    canvasRef.value.height = 100;
  }

  intervalTimer = setInterval(() => {
    updateData();
    draw();
  }, 1000);

  draw();
});

onUnmounted(() => {
  if (intervalTimer) clearInterval(intervalTimer);
});
</script>

<template>
  <Card class="p-4 overflow-hidden border-zinc-800">
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-4">
        <span class="text-xs font-semibold uppercase tracking-wider text-zinc-400">Bandwidth Activity (Realtime)</span>
        <div class="flex items-center gap-3 text-xs">
          <span class="flex items-center gap-1.5 text-cyan-400">
            <span class="w-2 h-2 rounded-full bg-cyan-400"></span> Download: {{ formatSpeed(downloadSpeed) }}
          </span>
          <span class="flex items-center gap-1.5 text-emerald-400">
            <span class="w-2 h-2 rounded-full bg-emerald-400"></span> Upload: {{ formatSpeed(uploadSpeed) }}
          </span>
        </div>
      </div>
      <span class="text-[11px] text-zinc-500 font-mono">Window: 40s</span>
    </div>
    <div class="w-full relative h-[100px]">
      <canvas ref="canvasRef" class="w-full h-full block"></canvas>
    </div>
  </Card>
</template>
