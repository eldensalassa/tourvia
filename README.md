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

Tourvia is a *Native Desktop* application developed using the **Rust** programming language and the **egui** framework. This application is designed to visually and interactively streamline the management process of *esports* or other sports tournaments.

## 🚀 Key Features

- **🏆 Smart Bracket System**: Fully supports **Single Elimination**. **Double Elimination** (with Upper/Lower Brackets) and **Round Robin** formats. Automatically routes match winners, losers, and calculates *BYE* scenarios.
- **👥 Global Roster & Team Management**: Register teams once in the *Global Roster* system and reuse them across multiple tournaments. Full support for uploading high-resolution team logos (PNG/JPG).
- **🌐 Built-in Web Image Scraper**: No need to download logos manually! Tourvia includes an integrated image search scraper to directly find and apply team logos from the web straight into your roster.
- **📊 Real-time Standings & Statistics**: Automatically calculates standings for the *Round Robin* format (Wins, Losses, Game Differentials). Interactive dashboard displays win ratios and total matches.
- **📸 High-Quality Bracket Export**: Share your tournament results! Export your giant tournament brackets into high-resolution PNG image formats with just a single click.
- **🔍 Pan & Zoom Navigation**: The diagram area (*Bracket*) is equipped with free-pan navigation and *zoom-in / zoom-out* controls to easily traverse large-scale tournament brackets.
- **💾 Secure Local Database**: All data is securely stored using a local SQLite database (`tourvia.db`). The database is natively stored in your OS's safe AppData/Local directory (e.g. `AppData/Roaming/Tourvia/data/` on Windows) ensuring data persistence.
- **🎨 Modern Theme**: Elegantly polished UI/UX featuring a "Refined Bronze & Dark Zinc" color palette for a professional administrative experience.

## 🛠️ System Prerequisites

Because this application is compiled natively, you need the Rust toolchain installed on your computer.

1. **Rust & Cargo**: Download and install via [rustup.rs](https://rustup.rs/).
2. **Compatible OS**: Windows, macOS, or Linux with standard *Graphics API* support (Vulkan, DirectX, Metal, or OpenGL).

## 📦 Pre-built Binaries (Easiest Method)

Don't want to compile from source? You can download the pre-compiled, ready-to-run binaries for your operating system directly from the **[GitHub Releases](../../releases)** page.
## 📥 Installation and Compilation

1. Open your Terminal or Command Prompt, then clone and navigate into this project directory.
2. Run the build command:
   ```bash
   cargo build --release
   ```
   *(Note: Use the `--release` flag to ensure optimal performance and smoother graphics rendering).*

## 🎮 How to Use the Application

Once the installation is complete, run the application using the following command:

```bash
cargo run
```

### 1. Dashboard & Global Roster
- On the **Dashboard** screen, you can view the list of currently active tournaments.
- Visit the **Global Roster** tab to register team names and upload team logos before the tournament starts. Teams in this roster are globally accessible across all tournaments.

### 2. Starting a New Tournament
- Click the **New Tournament** button, provide a Name, the Game being played, and select the format (Single Elimination, Double Elimination or Round Robin).
- Open the tournament, then navigate to the **Participants** tab. Click "Add Participant" to import teams from the *Global Roster* into your tournament.
- Click **"Generate Bracket"** to randomize and create the bracket.

### 3. Match Execution (Bracket View)
- Switch to the **Bracket** tab. Use your mouse scroll or the `+`/`-` zoom buttons to adjust the view size.
- Click on any match box that is *In Progress* or *Pending* to open the **Match Details** popup.
- Input the final score and click **Submit Match Result**. The winning team will automatically advance to the next round.

### 4. Image Export
- In the Bracket tab, you can click the **Save/Export** icon (floppy disk) in the top-left corner to export and save the tournament bracket view as a `.png` image format.

## 📂 Codebase Directory Structure

This application uses a simplified *Domain-Driven* architectural pattern:

- `src/main.rs`: The main entry-point that loads the eframe user interface window.
- `src/app.rs`: The core application *State* management context (bridges the UI and Services).
- `src/ui/`: Front-end user interface components.
  - `bracket_view.rs`: The absolute coordinate graphical rendering algorithm for match brackets.
  - `match_panel.rs`: Modal / Popup component for score input.
  - `dashboard.rs` & `global_roster.rs`: Main menu and global entity views.
  - `theme.rs`: *Design System* containing colors and Egui configuration.
- `src/services/`: Core business logic and algorithms.
  - `bracket_generator.rs`: Generates the logical structure of the tournament brackets.
  - `match_service.rs`: Computes team progression and calculates champion standings.
- `src/domain/`: Data model representations (Structs).
- `src/database/`: Database connection service to the local file using `rusqlite`.
- `src/utils/`: Additional utilities such as the PNG image export logic.

---
<div align="center">
  <i>Created by the Tourvia Team</i>
</div>
