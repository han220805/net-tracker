# Net Tracker

> **See every packet. Know every app.**

Monitor jaringan komputer secara realtime — lihat aplikasi mana yang terhubung ke internet, seberapa besar bandwidth yang dipakai, dan simpan history koneksi ke SQLite untuk forensik.

![Net Tracker Banner](src-tauri/icons/icon.png)

## Download

Lihat halaman [**Releases**](https://github.com/han220805/net-tracker/releases) untuk download installer terbaru.

| Platform | File |
|---|---|
| Windows 10/11 (x64) | `net-tracker_*_x64-setup.exe` |

## Fitur

- **Realtime Monitor** — Lihat semua koneksi aktif, aplikasi, hostname, IP, port, dan kecepatan download/upload per proses
- **Speed Graph** — Grafik kecepatan jaringan yang diperbarui tiap detik
- **History Log** — Semua koneksi tersimpan di SQLite lokal, bisa difilter berdasarkan rentang waktu (1 jam, 1 hari, 1 minggu, dll.)
- **Top Hostnames** — Lihat domain mana yang paling banyak diakses beserta aplikasi yang mengaksesnya
- **Dark / Light Mode** — Toggle theme, tersimpan otomatis
- **Settings** — Konfigurasi retensi data (7 hari hingga Selamanya), cleanup otomatis saat startup
- **Custom Window** — Frameless window dengan drag region dan window controls kustom

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Rust + Tauri v2
- **Database**: SQLite (via rusqlite)
- **Styling**: Tailwind CSS

## Development

```bash
# Install dependencies
bun install

# Run in development mode
bun run desktop

# Build installer
bun run desktop:build
```

## Requirements (Development)

- [Bun](https://bun.sh)
- [Rust](https://rustup.rs) (stable)
- Windows 10/11

## License

MIT
