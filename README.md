# <p align="center">🌐 Net Tracker</p>
<p align="center">
  <strong>Monitor lalu lintas internet komputer Anda secara realtime, transparan, dan privat.</strong>
</p>

<p align="center">
  <a href="https://github.com/han220805/net-tracker/releases/latest">
    <img src="https://img.shields.io/github/v/release/han220805/net-tracker?color=0078D6&label=Download%20Latest%20Version&logo=windows&logoColor=white" alt="Download Latest">
  </a>
  <img src="https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011%20(64--bit)-blue" alt="Windows Support">
  <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License">
  <img src="https://img.shields.io/badge/100%25-Private%20%26%20Local-emerald" alt="Privacy First">
</p>

<p align="center">
  <a href="https://github.com/han220805/net-tracker/releases/latest">
    <img src="https://img.shields.io/badge/⚡_DOWNLOAD_SEKARANG-(Free_for_Windows)-0284c7?style=for-the-badge" alt="Download Now" height="38">
  </a>
</p>

---

## 🧐 Pernahkah Anda Bertanya-tanya?

- *“Kenapa internet mendadak lemot? Aplikasi mana yang diam-diam sedot kuota?”*
- *“Apakah ada malware atau software mencurigakan yang koneksi ke server asing?”*
- *“Berapa kecepatan upload/download aplikasi saya saat ini?”*

**Net Tracker** hadir untuk menjawabnya. Cukup buka aplikasi, dan Anda langsung melihat seluruh aktivitas jaringan di PC Anda tanpa konfigurasi yang rumit.

---

## 🌟 Kenapa Memilih Net Tracker?

| 🚀 **Sangat Ringan & Cepat** | 🛡️ **100% Privat & Aman** | 📊 **Visual & Mudah Dipahami** |
| :--- | :--- | :--- |
| Dibuat dengan Rust + Tauri v2, tidak membebani RAM/CPU komputer Anda dibanding aplikasi berbasis Electron. | Data koneksi hanya disimpan di komputer Anda (SQLite lokal). Nol telemetri, tanpa upload data kemanapun. | Grafik kecepatan dinamis, pengelompokan per aplikasi, dan pencarian instan yang memanjakan mata. |

---

## ⚡ Fitur Andalan

```
  ┌────────────────────────────────────────────────────────────────────────┐
  │  Realtime Monitor  │  History Log  │ 🔴 Live 1s │ 🌙 Mode │ ⚙️ Setting │
  └────────────────────────────────────────────────────────────────────────┘
```

- 📡 **Pantau Koneksi Realtime**  
  Lihat setiap aplikasi (Chrome, Discord, Spotify, Game, dll.), IP tujuan, negara, domain (hostname DNS), dan kecepatan download/upload secara live per detik.

- 📈 **Grafik Bandwidth Dinamis**  
  Pantau lonjakan jaringan dengan grafik visual yang responsif dan ringkasan pemakaian total.

- 🗂️ **Mode Tampilan Cerdas (Grouped & Flat)**  
  Koneksi otomatis dirangkum per aplikasi agar tidak berantakan. Ingin melihat detail per soket port? Tinggal buka dropdown atau pilih mode Flat.

- 🕒 **History Log & Pelacakan Forensik**  
  Mencatat riwayat koneksi ke database SQLite lokal. Filter riwayat dengan mudah (1 jam terakhir, 1 hari, 1 minggu, hingga 1 bulan) atau cari domain/IP tertentu.

- 🧹 **Auto-Cleanup & Retensi Otomatis**  
  Bebas dari penumpukan data. Tentukan masa simpan riwayat (misal: 7 hari atau 30 hari) di menu pengaturan, aplikasi akan otomatis merapikannya.

- 🎨 **Tampilan Modern (Dark & Light Mode)**  
  Tampilan frameless yang elegan dengan dukungan tema gelap dan terang yang nyaman di mata.

---

## 📥 Cara Download & Install (Mudah)

Tidak perlu paham teknis coding! Cukup ikuti 3 langkah praktis ini:

1. **Unduh Installer**:  
   👉 [**Klik di sini untuk Download Versi Terbaru**](https://github.com/han220805/net-tracker/releases/latest)  
   *(Pilih file `net-tracker_*_x64-setup.exe`)*
2. **Jalankan Installer**:  
   Buka file `.exe` yang telah diunduh, lalu ikuti langkah instalasi singkat (Next ➡️ Install).
3. **Mulai Gunakan**:  
   Buka **Net Tracker** dari Desktop atau Start Menu. Selesai! 🎉

> 💡 **Kebutuhan Sistem**:
> - Windows 10 atau Windows 11 (64-bit).
> - WebView2 Runtime sudah tertanam langsung (Offline Installer), jadi aplikasi langsung siap pakai tanpa download tambahan.

---

## 🧭 Cara Penggunaan

### 1. Memantau Jaringan
- Begitu dibuka, tab **Realtime Monitor** langsung menampilkan aplikasi yang aktif memakai jaringan.
- Jika ingin menganalisis atau mencatat data pada momen tertentu, klik tombol **Live 1s** untuk mengubahnya menjadi **Paused**.

### 2. Mencari Riwayat Koneksi
- Klik tab **History Log** di bagian atas.
- Ketik nama domain (misal: `github.com`) atau aplikasi (misal: `spotify.exe`) pada kotak pencarian untuk melihat kapan saja koneksi terjadi dan seberapa banyak data yang terkirim.

### 3. Mengatur Penyimpanan & Tema
- Klik ikon **⚙️ (Settings)** di pojok kanan atas untuk memilih tema atau masa retensi log (7 hari, 30 hari, 90 hari, dst.).

---

## 🛠️ Untuk Pengembang (Developers)

Jika Anda developer yang ingin memodifikasi atau berkontribusi pada kode sumber:

<details>
<summary><b>Klik untuk melihat panduan Build & Run dari Source Code</b></summary>

### Prasyarat
- [Bun](https://bun.sh)
- [Rust & Cargo](https://rustup.rs) (stable)
- Windows C++ Build Tools / MinGW

### Setup & Development
```bash
# 1. Clone repository
git clone https://github.com/han220805/net-tracker.git
cd net-tracker

# 2. Pasang dependensi
bun install

# 3. Jalankan aplikasi mode dev
bun run desktop

# 4. Build installer rilis
bun run desktop:build
```

### Tech Stack
- **Frontend**: Vue 3, Vite, Tailwind CSS, Lucide Icons, Radix Vue
- **Backend Desktop**: Rust, Tauri v2, Windows IP Helper API (Win32), SQLite (rusqlite)

</details>

---

## 🔐 Keamanan & Privasi Terjamin

Aplikasi ini dibuat dengan prinsip **Privacy by Design**:
- **Nol Telemetri**: Tidak ada analitik pihak ketiga.
- **Lokal Sepenuhnya**: Semua log aktivitas tersimpan aman di direktori aplikasi lokal pengguna.
- **Open Source**: Kode terbuka dan transparan di bawah lisensi MIT.

---

## 🤝 Kontribusi & Dukungan

Menemukan bug atau punya ide fitur baru?
- Laporkan atau beri saran di [**GitHub Issues**](https://github.com/han220805/net-tracker/issues).
- Jangan lupa beri bintang ⭐ pada repositori ini jika Anda merasa aplikasi ini bermanfaat!

---

<p align="center">
  Dilisensikan di bawah <a href="LICENSE">MIT License</a>. Dibuat dengan ❤️ untuk pengguna yang peduli transparansi jaringan.
</p>

