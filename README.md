<div align="center">
  <img src="src/assets/logo.png" alt="Tourvia Logo" width="120" />
  <h1>Tourvia</h1>
  <p><strong>Tournament Visualization and Administration System</strong></p>
  <p>A fast, native desktop application for esports and sports tournament management, built with Rust and egui.</p>

  [![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](#)
  [![egui](https://img.shields.io/badge/egui-GUI-%23ff69b4.svg?style=for-the-badge)](#)
  [![SQLite](https://img.shields.io/badge/sqlite-%2307405e.svg?style=for-the-badge&logo=sqlite&logoColor=white)](#)
</div>

---

Tourvia adalah sebuah aplikasi *Desktop Native* yang dikembangkan menggunakan bahasa pemrograman **Rust** dan framework antarmuka **egui**. Aplikasi ini dirancang untuk memudahkan proses pengelolaan turnamen *esports* atau olahraga lainnya secara visual dan interaktif.

## 🚀 Fitur Utama

- **🏆 Sistem Bracket Cerdas**: Mendukung **Double Elimination** (Gugur Ganda dengan Upper/Lower Bracket) dan **Round Robin** (Setengah Kompetisi). Otomatis menghitung rute pemenang, tim kalah, dan pembagian *BYE*.
- **👥 Global Roster & Manajemen Tim**: Daftarkan tim sekali ke dalam sistem *Global Roster* dan gunakan kembali di berbagai turnamen. Dukungan penuh untuk *upload* logo tim beresolusi tinggi (PNG/JPG).
- **📊 Klasemen & Statistik Real-time**: Hitung skor klasemen otomatis untuk format *Round Robin* (Wins, Losses, Game Diff). Dashboard interaktif menampilkan rasio kemenangan dan total pertandingan.
- **📸 Ekspor Bracket ke PNG**: Bagikan hasil turnamen Anda! Ekspor bagan turnamen raksasa Anda ke dalam format gambar PNG resolusi tinggi hanya dengan satu klik.
- **🔍 Visualisasi Navigasi**: Area diagram (*Bracket*) dilengkapi navigasi gerak bebas (pan) dan *zoom-in / zoom-out* untuk mempermudah pengecekan bagan turnamen skala besar.
- **💾 Database Lokal Tangguh**: Seluruh data tersimpan aman menggunakan SQLite lokal (`tourvia.db`), menjamin data Anda tetap utuh meski aplikasi ditutup.
- **🎨 Tema Modern**: UI/UX yang dipoles elegan dengan kombinasi warna "Refined Bronze & Dark Zinc" untuk pengalaman profesional.

## 🛠️ Persyaratan Sistem (*Prerequisites*)

Karena aplikasi ini dikompilasi ke sistem *Native*, Anda memerlukan *toolchain* bahasa Rust ter-install di sistem komputer Anda.

1. **Rust & Cargo**: Unduh dan install melalui [rustup.rs](https://rustup.rs/).
2. **OS Kompatibel**: Windows, macOS, atau Linux dengan dukungan *Graphics API* standar (Vulkan, DirectX, Metal, atau OpenGL).

## 📥 Cara Instalasi dan Kompilasi

1. Buka Terminal atau Command Prompt, lalu *clone* / masuk ke dalam folder proyek ini.
2. Jalankan perintah kompilasi (*build*):
   ```bash
   cargo build --release
   ```
   *(Catatan: Gunakan flag `--release` untuk mendapatkan performa dan grafis yang lebih mulus dan optimal).*

## 🎮 Cara Penggunaan Aplikasi

Setelah instalasi selesai, jalankan aplikasi menggunakan perintah berikut:

```bash
cargo run
```

### 1. Dashboard & Roster Global
- Di layar **Dashboard**, Anda dapat melihat daftar turnamen yang sedang berjalan.
- Kunjungi tab **Global Roster** untuk mendaftarkan nama tim dan mengunggah logo tim Anda sebelum turnamen dimulai. Tim di Roster ini akan tersedia secara global di semua turnamen.

### 2. Memulai Turnamen Baru
- Klik tombol **New Tournament**, berikan Nama, Game yang dilombakan, dan pilih format (Double Elimination atau Round Robin).
- Buka turnamen, lalu di tab **Participants**, klik "Add Participant" untuk memasukkan tim dari *Global Roster* ke dalam turnamen.
- Klik **"Generate Bracket"** untuk mengacak bagan.

### 3. Eksekusi Pertandingan (Bracket View)
- Pindah ke tab **Bracket**. Gunakan scroll mouse atau tombol zoom `+`/`-` untuk mengatur ukuran tampilan.
- Klik pada salah satu kotak pertandingan yang berstatus *In Progress* atau *Pending* untuk membuka popup **Match Details**.
- Input skor akhir lalu tekan **Submit Match Result**. Tim yang menang otomatis akan maju ke babak selanjutnya.

### 4. Ekspor Gambar
- Di tab Bracket, Anda dapat menekan ikon **Save/Export** (ikon disket) di pojok kiri atas untuk menyimpan tampilan bagan turnamen ke dalam format gambar `.png`.

## 📂 Struktur Direktori (*Codebase*)

Aplikasi ini menggunakan pola arsitektur *Domain-Driven* yang disederhanakan:

- `src/main.rs`: Titik masuk utama yang me-*load* jendela antarmuka eframe.
- `src/app.rs`: Konteks manajemen *State* aplikasi utama (menjembatani UI dan Services).
- `src/ui/`: Komponen antarmuka (*front-end*).
  - `bracket_view.rs`: Algoritma rendering grafis koordinat absolut untuk bagan pertandingan.
  - `match_panel.rs`: Komponen Modal / *Popup* pengisian skor.
  - `dashboard.rs` & `global_roster.rs`: Tampilan menu utama.
  - `theme.rs`: *Design System* warna dan konfigurasi Egui.
- `src/services/`: Logika bisnis dan algoritma.
  - `bracket_generator.rs`: Menghasilkan susunan logis bagan turnamen.
  - `match_service.rs`: Menghitung alur kemajuan tim dan klasemen juara.
- `src/domain/`: Representasi model data (Structs).
- `src/database/`: Layanan koneksi database ke file lokal menggunakan `rusqlite`.
- `src/utils/`: Utilitas tambahan seperti logika ekspor gambar PNG.

---
<div align="center">
  <i>Dibuat oleh Tim Tourvia - Final Project Desktop - 2026</i>
</div>
