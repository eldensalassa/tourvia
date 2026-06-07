# Product Requirements Document (PRD)
## Feature: Native Desktop Broadcast Overlay (OBS Integration)

**Document Status:** Draft / Proposed
**Product:** Tourvia (Tournament Visualization and Administration System)
**Author:** Tourvia Engineering (Antigravity)
**Date:** June 2026

---

## 1. Executive Summary
Tourvia dirancang untuk menjadi pusat kendali turnamen yang profesional. Untuk mendukung siaran langsung pertandingan (*live streaming*) di *platform* seperti YouTube atau Twitch, penyiar (*broadcaster*) membutuhkan grafis *overlay* (seperti papan skor dan *bracket*) di atas video pertandingan. 

Karena adanya **batasan larangan penggunaan teknologi berbasis web (HTML/CSS/JS)** pada proyek ini, fitur ini akan diimplementasikan 100% menggunakan arsitektur Desktop Native (Rust & `egui`). Solusinya adalah menggunakan **Frameless Transparent Secondary Viewport** yang berjalan pada 60 FPS dan dapat ditangkap langsung oleh OBS Studio dengan transparansi (Alpha Channel) sempurna.

---

## 2. Problem Statement (Pernyataan Masalah)
- Penyiar turnamen membutuhkan cara untuk menampilkan papan skor dan *bracket* secara *real-time* ke penonton tanpa mengganggu layar permainan.
- Solusi industri (Browser Source/Web Overlay) dilarang digunakan dalam lingkup proyek ini.
- Melakukan *Screen Capture* pada jendela aplikasi utama Tourvia terlihat sangat tidak profesional karena terdapat bingkai aplikasi, *background* gelap, dan menu-menu kontrol (tombol admin) yang tidak seharusnya dilihat oleh penonton.

---

## 3. Target Audience (Persona Pengguna)
1. **Tournament Admin / Operator:** Orang yang duduk di depan aplikasi Tourvia untuk menekan tombol skor, mengganti status pertandingan, dan memajukan *bracket*.
2. **Broadcaster / Caster:** Orang yang mengatur OBS Studio untuk menggabungkan video permainan dengan grafis *overlay* Tourvia.

---

## 4. Goals & Non-Goals (Ruang Lingkup)
### ✅ Goals (Tujuan Utama)
1. Menghasilkan *overlay* siaran langsung yang terlihat sekelas turnamen *esports* profesional (bersih, tajam, dan elegan).
2. Mendukung transparansi penuh (kaca/tembus pandang) sehingga OBS bisa menumpuknya di atas video tanpa latar belakang hitam/hijau.
3. Sinkronisasi data *real-time* (ketika Admin mengubah skor di layar kontrol, skor di jendela *overlay* otomatis berubah seketika tanpa penundaan/latensi).
4. Murni berjalan secara *native* di Desktop menggunakan Rust (`eframe`/`egui`).

### ❌ Non-Goals (Di Luar Cakupan)
1. Tidak menggunakan protokol jaringan (*Localhost HTTP/WebSocket*).
2. Tidak membuat animasi transisi 3D yang sangat berat (karena keterbatasan *Immediate Mode GUI*).
3. Tidak mendukung integrasi API eksternal (murni dikontrol langsung dari *database* lokal Tourvia).

---

## 5. User Stories
- **Sebagai Admin,** saya ingin memiliki panel khusus "Broadcast Control" di dalam Tourvia untuk memilih elemen apa yang ingin saya tampilkan ke penonton (Papan Skor Match 1, Bracket Keseluruhan, atau Standings).
- **Sebagai Admin,** saya ingin menekan tombol "Buka Jendela Overlay" yang akan memunculkan jendela baru terpisah dari menu kontrol saya.
- **Sebagai Broadcaster,** saya ingin menambahkan jendela *overlay* Tourvia ke dalam OBS Studio menggunakan fitur *Window Capture (Allow Transparency)* dan langsung melihat hasilnya menyatu dengan siaran saya.

---

## 6. Functional Requirements (Kebutuhan Fungsional)
1. **Multi-Window Support (Viewport Architecture):**
   - Aplikasi harus memanggil `ctx.show_viewport_immediate()` untuk membuka jendela kedua.
   - Konfigurasi jendela kedua **WAJIB**: `decorations: false` (tanpa tombol X atau *minimize* bawaan Windows), `transparent: true`, dan `always_on_top: true`.

2. **Broadcast Control Panel (UI Utama):**
   - Menambahkan menu baru di Sidebar: **"📺 Broadcast"**.
   - Menyediakan tombol *Toggle* (ON/OFF) untuk memunculkan/menutup jendela *overlay*.
   - Menyediakan *Dropdown* pemilih mode: `[Scoreboard Mode]`, `[Bracket Mode]`, `[Custom Lower Third]`.

3. **Data Binding (Sinkronisasi):**
   - Jendela *overlay* hanya bertugas membaca (Read-Only) status dari struktur state utama `TourviaApp` (misalnya `app.active_match`). Perubahan yang disimpan ke *database* SQLite dari jendela utama otomatis terlukis ulang di jendela *overlay*.

---

## 7. Non-Functional Requirements (Kebutuhan Non-Fungsional)
1. **Performance:** Mempertahankan 60 FPS saat me-*render* dua jendela secara bersamaan tanpa lonjakan beban CPU/GPU berlebih.
2. **Cross-Platform:** Transparansi OS (`winit` transparent window) harus berfungsi minimal di Windows 10/11 dan distro Linux utama (Wayland/X11), serta macOS.
3. **Usability:** Antarmuka *overlay* tidak boleh memiliki tombol yang bisa diklik. Hanya menampilkan data visual (*Read-Only View*).

---

## 8. Desain Arsitektur Sistem
Implementasi fitur ini pada ekosistem `eframe`:
```rust
// Konsep Arsitektur Sederhana
if self.show_broadcast_window {
    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_title("Tourvia Broadcast Overlay")
        .with_transparent(true)
        .with_decorations(false)
        .with_always_on_top(true)
        .with_inner_size([1280.0, 720.0]); // Standar resolusi 720p/1080p

    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("broadcast_overlay"),
        viewport_builder,
        |ctx, class| {
            // Render UI Transparan di Sini
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
                .show(ctx, |ui| {
                    match self.broadcast_mode {
                        BroadcastMode::Scoreboard => draw_scoreboard_overlay(ui, &self.active_match),
                        BroadcastMode::Bracket => draw_bracket_overlay(ui, &self.rounds),
                    }
                });
        },
    );
}
```

---

## 9. Metrik Kesuksesan (Success Criteria)
- [ ] Tombol "Buka Overlay" berhasil memunculkan jendela tanpa bingkai.
- [ ] OBS Studio dapat melakukan *Window Capture* pada jendela tersebut.
- [ ] *Background* jendela di OBS Studio tidak hitam/putih, melainkan benar-benar menembus (menampilkan layar/video di belakangnya).
- [ ] Mengubah skor di jendela kendali Admin langsung mengubah angka di OBS secara instan (< 16 milidetik penundaan).

---

## 10. Referensi Desain Industri
Untuk gaya visualnya, *overlay* akan mengadaptasi gaya *clean look*:
- *Scoreboard* berada di posisi tengah-atas layar (*Top Center*).
- Menggunakan *font* yang tebal (seperti Roboto/Inter).
- Memanfaatkan sudut melengkung (*rounded corners* dengan `egui::Rounding`) dan bayangan yang tegas untuk membedakannya dari *background* siaran.
