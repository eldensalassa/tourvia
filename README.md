# Tourvia - Tournament Visualization and Administration System

Tourvia adalah sebuah aplikasi *Desktop Native* yang dikembangkan menggunakan bahasa pemrograman **Rust** dan framework antarmuka **egui**. Aplikasi ini dirancang untuk memudahkan proses pengelolaan turnamen *esports* atau olahraga lainnya secara visual dan interaktif, mulai dari pendaftaran tim, pengundian bracket, hingga pencatatan skor dan penentuan juara.

## 🚀 Fitur Utama

- **Double Elimination Bracket**: Sistem turnamen gugur ganda (Upper & Lower bracket) yang dibuat dengan garis konektor simetris, menampilkan rute kalah, BYE otomatis, dan babak *Grand Final*.
- **Round Robin Bracket**: Sistem turnamen setengah kompetisi (bertemu semua) yang diacak secara otomatis berdasarkan jumlah peserta.
- **Manajemen Peserta & Logo**: Tambahkan peserta secara manual dan unggah logo tim dari komputer lokal (mendukung format gambar standar seperti PNG/JPG).
- **Klasemen Otomatis (Standings)**: Hitung skor dan poin klasemen secara otomatis, sangat berguna dalam format *Round Robin*.
- **Dashboard Statistik**: Lacak total pertandingan, rasio kemenangan, *win rate* tim, dan perkembangan turnamen secara *real-time*.
- **Visualisasi Pan & Zoom**: Area diagram (*Bracket*) dilengkapi navigasi gerak (klik tahan lalu seret) dan tombol _zoom-in_/_zoom-out_ untuk mempermudah pengecekan bagan raksasa.
- **Database Lokal**: Seluruh data dan pengaturan pertandingan disimpan menggunakan SQLite lokal (file `tourvia.db`), menjamin data Anda tetap utuh meski aplikasi ditutup.

---

## 🛠️ Persyaratan Sistem (*Prerequisites*)

Karena aplikasi ini dikompilasi ke sistem *Native*, Anda memerlukan *toolchain* bahasa Rust ter-install di sistem komputer Anda.

1. **Rust & Cargo**: Pastikan Rust sudah ter-install di komputer Anda. Anda bisa mengunduhnya melalui [rustup.rs](https://rustup.rs/).
2. Pastikan Anda menggunakan OS Windows/Linux/macOS dengan dukungan *Graphics API* standar (Vulkan, DirectX, Metal, atau OpenGL) karena `egui` berjalan di atas pemroses grafis.

---

## 📥 Cara Instalasi dan Kompilasi

1. Buka Terminal atau Command Prompt, lalu arahkan (*cd*) ke dalam *folder* root proyek ini:
   
2. Jalankan perintah instalasi dan kompilasi (*build*):
   ```bash
   cargo build --release
   ```
   *(Opsional: Gunakan `--release` untuk mendapatkan performa dan grafis yang lebih optimal)*.

---

## 🎮 Cara Penggunaan Aplikasi

Setelah instalasi selesai, jalankan aplikasi menggunakan perintah berikut:

```bash
cargo run
```

### 1. Memulai Turnamen Baru
- Saat aplikasi terbuka, Anda akan berada di layar **Dashboard**.
- Anda dapat membuat Turnamen Baru atau memuat turnamen yang pernah Anda buat sebelumnya.
- Jika membuat turnamen baru, berikan Nama Turnamen, Game yang dilombakan, dan pilih tipe format (Double Elimination atau Round Robin).

### 2. Mengelola Peserta (Tab "Participants")
- Masuk ke tab **Participants**.
- Isi nama pemain atau tim, lalu tentukan **Seed** (peringkat unggulan). 
- *(Opsional)* Klik kotak gambar logo di sebelah input nama untuk mencari foto/logo tim Anda ke dalam sistem (sistem otomatis akan menyalin logo ini).
- Klik tombol **"Add Participant"**.
- Setelah tim dirasa cukup (minimal 2 tim), klik tombol **"Generate Bracket"**. Peringatan: *Generate Bracket* akan mereset seluruh skor yang sedang berjalan!

### 3. Memantau Jadwal (Tab "Bracket")
- Pindah ke tab **Bracket**.
- **Zoom / Pan**: Anda dapat menekan tombol `+` atau `-` di pojok kanan atas untuk mengatur ukuran diagram. Anda juga bisa menggeser area diagram ke segala arah.
- **Detail Pertandingan**: Klik pada salah satu kotak pertandingan untuk memunculkan Jendela Detil (Match Details).
- **Input Skor**: Di jendela *Match Details*, Anda bisa mencetak skor pada kotak isian "Score" lalu menekan tombol **"Submit Match Result"**. Sistem otomatis akan meloloskan pemenang ke babak berikutnya, atau membuang tim yang kalah ke *Lower Bracket* (jika mode Double Elimination).

### 4. Memantau Klasemen (Tab "Standings")
- Tab ini sangat relevan untuk turnamen bertipe *Round Robin*.
- Data Points, jumlah Menang, Seri, Kalah akan dihitung secara *live* berdasarkan total skor yang sudah disubmit di tab Bracket.

### 5. Selesai Turnamen
- Saat seluruh pertandingan selesai, juara turnamen (Champion) akan otomatis diumumkan di bagian atas tab *Bracket*.

---

## 📂 Struktur Direktori (*Codebase*)

- `src/main.rs`: Titik masuk (entry-point) utama yang me-*load* jendela antarmuka eframe.
- `src/app.rs`: Konteks manajemen *State* aplikasi utama (menjembatani UI dan Database).
- `src/ui/`: Semua komponen antarmuka (*front-end*).
  - `bracket_view.rs`: Sistem algoritma penggambar diagram bagan pertandingan (Konektor Garis, Algoritma Tampilan, dsb).
  - `match_panel.rs`: Komponen Modal / *Popup* pengisian skor.
  - `participant_panel.rs`: Layar administrasi tim.
  - `theme.rs`: *Design System* warna dan ukuran teks.
- `src/services/`: Logika algoritma pembentuk turnamen.
  - `bracket_generator.rs`: Menghasilkan susunan bagan *Double Elimination* dan *Round Robin* otomatis (termasuk BYE dan Seed).
- `src/domain/`: Representasi model data (Structs).
- `src/database/`: Layanan koneksi SQLite menggunakan `rusqlite`.

---
*Dibuat oleh Tim Tourvia - Final Project Desktop - 2026*
