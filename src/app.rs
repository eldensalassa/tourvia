use std::collections::HashMap;

use crate::database::Database;
use crate::domain::match_model::Match;
use crate::domain::participant::Participant;
use crate::domain::round::Round;
use crate::domain::tournament::{Tournament, TournamentStatus, TournamentType};
use crate::services::{bracket_generator, match_service, tournament_service};
use crate::ui;

use rand::seq::SliceRandom;

// ─── View Enum ──────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Dashboard,
    TournamentForm,
    TournamentDetail,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TournamentTab {
    Overview,
    Participants,
    Bracket,
    Standings,
    Scoreboard,
}

#[derive(Debug, Clone)]
pub enum MessageType {
    Success,
    Error,
    Info,
}

// ─── Application State ─────────────────────────────

pub struct TourviaApp {
    pub db: Database,
    pub current_view: View,
    pub active_tab: TournamentTab,
    pub theme_applied: bool,

    // Data
    pub tournaments: Vec<Tournament>,
    pub active_tournament: Option<Tournament>,
    pub participants: Vec<Participant>,
    pub rounds: Vec<Round>,
    pub matches: Vec<Match>,
    pub selected_match: Option<String>,
    pub champion_name: Option<String>,

    // Form state
    pub new_tournament_name: String,
    pub new_tournament_type: TournamentType,
    pub new_tournament_description: String,
    pub new_tournament_game: String,
    pub new_participant_name: String,

    // Score input (Modal)
    pub show_match_modal: bool,
    pub score_input: [String; 2],
    pub score_submitted: bool,

    // Status message
    pub status_message: Option<(String, MessageType)>,

    // Dashboard
    pub search_query: String,
    pub confirm_delete: Option<usize>,

    // Stats
    pub show_stats: bool,

    // Bulk add
    pub bulk_add_text: String,
    pub show_bulk_add: bool,

    // Logo texture cache
    pub logo_textures: HashMap<String, egui::TextureHandle>,
    pub logos_loaded_for: Option<String>,

    // Zoom
    pub bracket_zoom: f32,
}

impl TourviaApp {
    pub fn new(db: Database) -> Self {
        let tournaments = tournament_service::load_all_tournaments(&db).unwrap_or_default();

        Self {
            db,
            current_view: View::Dashboard,
            active_tab: TournamentTab::Overview,
            theme_applied: false,
            tournaments,
            active_tournament: None,
            participants: Vec::new(),
            rounds: Vec::new(),
            matches: Vec::new(),
            selected_match: None,
            champion_name: None,
            new_tournament_name: String::new(),
            new_tournament_type: TournamentType::SingleElimination,
            new_tournament_description: String::new(),
            new_tournament_game: String::new(),
            new_participant_name: String::new(),
            show_match_modal: false,
            score_input: [String::new(), String::new()],
            score_submitted: false,
            status_message: None,
            search_query: String::new(),
            confirm_delete: None,
            show_stats: false,
            bulk_add_text: String::new(),
            show_bulk_add: false,
            logo_textures: HashMap::new(),
            logos_loaded_for: None,
            bracket_zoom: 1.0,
        }
    }

    // ─── Navigation ─────────────────────────────────

    pub fn go_to_dashboard(&mut self) {
        self.current_view = View::Dashboard;
        self.active_tournament = None;
        self.status_message = None;
        self.confirm_delete = None;
        self.refresh_tournaments();
    }

    pub fn show_tournament_form(&mut self) {
        self.current_view = View::TournamentForm;
        self.new_tournament_name.clear();
        self.new_tournament_type = TournamentType::SingleElimination;
        self.new_tournament_description.clear();
        self.new_tournament_game.clear();
        self.status_message = None;
    }

    pub fn open_tournament(&mut self, idx: usize) {
        if idx < self.tournaments.len() {
            let tournament = self.tournaments[idx].clone();
            self.active_tournament = Some(tournament.clone());
            self.current_view = View::TournamentDetail;
            self.active_tab = TournamentTab::Overview;
            self.status_message = None;
            self.selected_match = None;
            self.show_match_modal = false;
            self.score_input = [String::new(), String::new()];
            self.score_submitted = false;
            self.bracket_zoom = 1.0;
            self.load_tournament_data(&tournament.id);
        }
    }

    // ─── Data Loading ───────────────────────────────

    fn refresh_tournaments(&mut self) {
        self.tournaments = tournament_service::load_all_tournaments(&self.db).unwrap_or_default();
    }

    fn load_tournament_data(&mut self, tournament_id: &str) {
        self.participants = self
            .db
            .get_participants_by_tournament(tournament_id)
            .unwrap_or_default();

        self.rounds = self
            .db
            .get_rounds_by_tournament(tournament_id)
            .unwrap_or_default();

        self.matches = self
            .db
            .get_matches_by_tournament(tournament_id)
            .unwrap_or_default();

        self.champion_name = match_service::get_champion(&self.db, tournament_id).unwrap_or(None);

        if self.logos_loaded_for.as_deref() != Some(tournament_id) {
            self.logo_textures.clear();
            self.logos_loaded_for = Some(tournament_id.to_string());
        }
    }

    pub fn ensure_logos_loaded(&mut self, ctx: &egui::Context) {
        for p in &self.participants {
            if p.has_logo && !self.logo_textures.contains_key(&p.id) {
                if let Ok(Some(data)) = self.db.get_participant_logo(&p.id) {
                    if let Some(texture) = Self::decode_logo(ctx, &p.id, &data) {
                        self.logo_textures.insert(p.id.clone(), texture);
                    }
                }
            }
        }
    }

    fn decode_logo(ctx: &egui::Context, name: &str, data: &[u8]) -> Option<egui::TextureHandle> {
        let img = image::load_from_memory(data).ok()?;
        let rgba = img.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
        Some(ctx.load_texture(
            format!("logo_{}", name),
            color_image,
            egui::TextureOptions::LINEAR,
        ))
    }

    // ─── Tournament Actions ─────────────────────────

    pub fn create_tournament(&mut self) {
        match tournament_service::create_tournament(
            &self.db,
            &self.new_tournament_name,
            self.new_tournament_type.clone(),
            &self.new_tournament_description,
            &self.new_tournament_game,
        ) {
            Ok(tournament) => {
                self.status_message = Some((
                    format!("Tournament '{}' created!", tournament.name),
                    MessageType::Success,
                ));
                self.active_tournament = Some(tournament.clone());
                self.current_view = View::TournamentDetail;
                self.active_tab = TournamentTab::Overview;
                self.participants.clear();
                self.rounds.clear();
                self.matches.clear();
                self.refresh_tournaments();
            }
            Err(e) => {
                self.status_message = Some((e, MessageType::Error));
            }
        }
    }

    pub fn delete_tournament_at(&mut self, idx: usize) {
        if idx < self.tournaments.len() {
            let id = self.tournaments[idx].id.clone();
            if let Err(e) = tournament_service::delete_tournament(&self.db, &id) {
                self.status_message = Some((e, MessageType::Error));
            } else {
                self.confirm_delete = None;
                self.refresh_tournaments();
            }
        }
    }

    pub fn reset_bracket(&mut self) {
        let tid = match &self.active_tournament {
            Some(t) => t.id.clone(),
            None => return,
        };
        match tournament_service::reset_bracket(&self.db, &tid) {
            Ok(_) => {
                self.load_tournament_data(&tid);
                self.refresh_tournaments();
                if let Ok(Some(t)) = self.db.get_tournament(&tid) {
                    self.active_tournament = Some(t);
                }
                self.status_message = Some(("Bracket reset to Draft.".to_string(), MessageType::Info));
            }
            Err(e) => {
                self.status_message = Some((e, MessageType::Error));
            }
        }
    }

    pub fn export_json(&mut self) {
        let tid = match &self.active_tournament {
            Some(t) => t.id.clone(),
            None => return,
        };
        match tournament_service::export_tournament_json(&self.db, &tid) {
            Ok(json) => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_file_name("tournament_export.json")
                    .save_file()
                {
                    match std::fs::write(&path, &json) {
                        Ok(_) => {
                            self.status_message = Some((
                                format!("Exported to {}", path.display()),
                                MessageType::Success,
                            ));
                        }
                        Err(e) => {
                            self.status_message = Some((
                                format!("Failed to write file: {}", e),
                                MessageType::Error,
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                self.status_message = Some((e, MessageType::Error));
            }
        }
    }

    // ─── Participant Actions ────────────────────────

    pub fn add_participant(&mut self) {
        let name = self.new_participant_name.trim().to_string();
        if name.is_empty() {
            self.status_message = Some(("Name cannot be empty.".to_string(), MessageType::Error));
            return;
        }

        let tid = if let Some(ref tournament) = self.active_tournament {
            tournament.id.clone()
        } else { return; };

        match self.db.participant_exists(&tid, &name) {
            Ok(true) => {
                self.status_message = Some((format!("'{}' already exists.", name), MessageType::Error));
                return;
            }
            Err(e) => {
                self.status_message = Some((format!("Database error: {}", e), MessageType::Error));
                return;
            }
            _ => {}
        }

        let seed = self.participants.len() as i32 + 1;
        let participant = Participant::new(tid.clone(), name.clone(), seed);

        match self.db.create_participant(&participant) {
            Ok(_) => {
                self.new_participant_name.clear();
                self.load_tournament_data(&tid);
                let count = self.participants.len();
                let _ = self.db.update_tournament_participant_count(&tid, count);
                self.status_message = Some((format!("'{}' added", participant.name), MessageType::Success));
                self.refresh_tournaments();
            }
            Err(e) => {
                self.status_message = Some((format!("Failed to add: {}", e), MessageType::Error));
            }
        }
    }

    pub fn bulk_add_participants(&mut self) {
        let tid = match &self.active_tournament { Some(t) => t.id.clone(), None => return };

        let names: Vec<String> = self.bulk_add_text.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();

        if names.is_empty() {
            self.status_message = Some(("No names provided.".to_string(), MessageType::Error));
            return;
        }

        let mut added = 0;
        let mut current_seed = self.participants.len() as i32 + 1;
        for name in &names {
            if self.db.participant_exists(&tid, name).unwrap_or(true) { continue; }
            let p = Participant::new(tid.clone(), name.clone(), current_seed);
            if self.db.create_participant(&p).is_ok() {
                added += 1;
                current_seed += 1;
            }
        }

        self.load_tournament_data(&tid);
        let count = self.participants.len();
        let _ = self.db.update_tournament_participant_count(&tid, count);
        self.refresh_tournaments();
        self.bulk_add_text.clear();
        self.show_bulk_add = false;
        self.status_message = Some((format!("{} participant(s) added.", added), MessageType::Success));
    }

    pub fn delete_participant(&mut self, idx: usize) {
        if idx < self.participants.len() {
            let p_id = self.participants[idx].id.clone();
            let tid = if let Some(ref tournament) = self.active_tournament { tournament.id.clone() } else { return };

            self.logo_textures.remove(&p_id);

            if let Err(e) = self.db.delete_participant(&p_id) {
                self.status_message = Some((format!("Failed to delete: {}", e), MessageType::Error));
            } else {
                self.load_tournament_data(&tid);
                let count = self.participants.len();
                let _ = self.db.update_tournament_participant_count(&tid, count);
                self.status_message = Some(("Participant removed.".to_string(), MessageType::Info));
                self.refresh_tournaments();
            }
        }
    }

    pub fn move_participant_up(&mut self, idx: usize) {
        if idx == 0 || idx >= self.participants.len() { return; }
        let tid = match &self.active_tournament { Some(t) => t.id.clone(), None => return };

        let id_a = self.participants[idx].id.clone();
        let name_a = self.participants[idx].name.clone();
        let seed_a = self.participants[idx].seed;

        let id_b = self.participants[idx - 1].id.clone();
        let name_b = self.participants[idx - 1].name.clone();
        let seed_b = self.participants[idx - 1].seed;

        let _ = self.db.update_participant(&id_a, &name_a, seed_b);
        let _ = self.db.update_participant(&id_b, &name_b, seed_a);
        self.load_tournament_data(&tid);
    }

    pub fn move_participant_down(&mut self, idx: usize) {
        if idx + 1 >= self.participants.len() { return; }
        self.move_participant_up(idx + 1);
    }

    pub fn import_logo_for_participant(&mut self, idx: usize, ctx: &egui::Context) {
        if idx >= self.participants.len() { return; }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG Images", &["png"])
            .set_title("Select Team Logo (PNG)")
            .pick_file()
        {
            let pid = self.participants[idx].id.clone();
            match std::fs::read(&path) {
                Ok(raw_bytes) => {
                    match Self::process_logo(&raw_bytes) {
                        Ok(processed) => {
                            if self.db.set_participant_logo(&pid, &processed).is_ok() {
                                if let Some(tex) = Self::decode_logo(ctx, &pid, &processed) {
                                    self.logo_textures.insert(pid.clone(), tex);
                                }
                                self.participants[idx].has_logo = true;
                                self.status_message = Some(("Logo imported!".to_string(), MessageType::Success));
                            }
                        }
                        Err(e) => {
                            self.status_message = Some((e, MessageType::Error));
                        }
                    }
                }
                Err(e) => {
                    self.status_message = Some((format!("Failed to read file: {}", e), MessageType::Error));
                }
            }
        }
    }

    fn process_logo(raw_bytes: &[u8]) -> Result<Vec<u8>, String> {
        let img = image::load_from_memory(raw_bytes).map_err(|e| format!("Invalid image: {}", e))?;
        let resized = img.resize(128, 128, image::imageops::FilterType::Lanczos3);
        let mut buf = std::io::Cursor::new(Vec::new());
        resized.write_to(&mut buf, image::ImageFormat::Png).map_err(|e| format!("Failed to encode: {}", e))?;
        Ok(buf.into_inner())
    }

    pub fn auto_seed_participants(&mut self) {
        let tid = if let Some(ref tournament) = self.active_tournament { tournament.id.clone() } else { return };
        let mut rng = rand::thread_rng();
        let mut indices: Vec<usize> = (0..self.participants.len()).collect();
        indices.shuffle(&mut rng);

        for (new_seed, &idx) in indices.iter().enumerate() {
            let p = &self.participants[idx];
            let _ = self.db.update_participant(&p.id, &p.name, new_seed as i32 + 1);
        }

        self.load_tournament_data(&tid);
        self.status_message = Some(("Seeds randomized!".to_string(), MessageType::Success));
    }

    // ─── Bracket Actions ────────────────────────────

    pub fn generate_bracket(&mut self) {
        let (tid, ttype) = if let Some(ref tournament) = self.active_tournament {
            (tournament.id.clone(), tournament.tournament_type.clone())
        } else { return; };

        match bracket_generator::generate_bracket(&self.db, &tid, &self.participants, &ttype) {
            Ok(_) => {
                let _ = tournament_service::update_status(&self.db, &tid, TournamentStatus::InProgress);
                self.load_tournament_data(&tid);
                self.refresh_tournaments();
                if let Ok(Some(t)) = self.db.get_tournament(&tid) {
                    self.active_tournament = Some(t);
                }
                self.active_tab = TournamentTab::Bracket; // Auto-switch to bracket tab
                self.status_message = Some(("Bracket generated successfully!".to_string(), MessageType::Success));
            }
            Err(e) => {
                self.status_message = Some((e, MessageType::Error));
            }
        }
    }

    // ─── Match Actions ──────────────────────────────

    pub fn select_match(&mut self, match_id: &str) {
        self.selected_match = Some(match_id.to_string());
        self.score_input = [String::new(), String::new()];
        self.status_message = None;
        self.show_match_modal = true; // Open modal
    }

    pub fn submit_match_score(&mut self) {
        let match_id = match &self.selected_match { Some(id) => id.clone(), None => return };

        let s1: i32 = match self.score_input[0].trim().parse() {
            Ok(v) => v,
            Err(_) => {
                self.status_message = Some(("Invalid score for Player 1.".to_string(), MessageType::Error));
                return;
            }
        };

        let s2: i32 = match self.score_input[1].trim().parse() {
            Ok(v) => v,
            Err(_) => {
                self.status_message = Some(("Invalid score for Player 2.".to_string(), MessageType::Error));
                return;
            }
        };

        match match_service::submit_score(&self.db, &match_id, s1, s2) {
            Ok(_) => {
                // Update skor di self.matches secara lokal supaya live card
                // langsung menampilkan skor baru — TANPA reload (biar card tidak hilang dulu)
                if let Some(m) = self.matches.iter_mut().find(|m| m.id == match_id) {
                    m.score1 = s1;
                    m.score2 = s2;
                }
                self.status_message = Some(("Score submitted!".to_string(), MessageType::Success));
                // Tidak reload, tidak auto-advance — tunggu user klik "Next Match"
            }
            Err(e) => {
                self.status_message = Some((e, MessageType::Error));
            }
        }
    }

    pub fn advance_to_next_match(&mut self) {
        self.score_input = [String::new(), String::new()];
        self.show_match_modal = false;
        self.selected_match = None;
        self.score_submitted = false;
        // Baru di sini reload data lengkap — match selesai hilang dari live, next match muncul
        if let Some(ref tournament) = self.active_tournament {
            let tid = tournament.id.clone();
            self.load_tournament_data(&tid);
            if let Ok(true) = match_service::is_tournament_complete(&self.db, &tid) {
                let _ = tournament_service::update_status(&self.db, &tid, TournamentStatus::Completed);
                if let Ok(Some(t)) = self.db.get_tournament(&tid) { self.active_tournament = Some(t); }
                self.refresh_tournaments();
            }
        }
    }

    // ─── Helpers ────────────────────────────────────

    pub fn is_draft(&self) -> bool {
        self.active_tournament.as_ref().map(|t| t.status == TournamentStatus::Draft).unwrap_or(false)
    }

    pub fn filtered_tournaments(&self) -> Vec<(usize, &Tournament)> {
        let query = self.search_query.to_lowercase();
        self.tournaments.iter().enumerate().filter(|(_, t)| {
            query.is_empty() || t.name.to_lowercase().contains(&query) || t.game_name.to_lowercase().contains(&query) || t.status.as_str().to_lowercase().contains(&query)
        }).collect()
    }
}

// ─── eframe::App Implementation ─────────────────────

impl eframe::App for TourviaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_applied {
            ui::theme::apply_theme(ctx);
            self.theme_applied = true;
        }

        match self.current_view {
            View::Dashboard => {
                egui::CentralPanel::default().show(ctx, |panel_ui| {
                    ui::dashboard::render(self, panel_ui);
                });
            }
            View::TournamentForm => {
                egui::CentralPanel::default().show(ctx, |panel_ui| {
                    ui::tournament_form::render(self, panel_ui);
                });
            }
            View::TournamentDetail => {
                self.ensure_logos_loaded(ctx);
                let ctx_clone = ctx.clone();

                // Top bar with Title & Tabs
                egui::TopBottomPanel::top("top_nav")
                    .frame(egui::Frame::new().fill(ui::theme::BG_PANEL).inner_margin(egui::Margin::symmetric(24, 16)))
                    .show(ctx, |ui| {
                        
                        // Top line: Back button, Title, Actions
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new(egui::RichText::new("← Dashboard").color(ui::theme::TEXT_MUTED).size(13.0)).fill(egui::Color32::TRANSPARENT)).clicked() {
                                self.go_to_dashboard();
                                return;
                            }
                            
                            ui.add_space(8.0);
                            
                            if let Some(ref t) = self.active_tournament {
                                ui.label(ui::theme::heading_text(&t.name));
                                ui.add_space(8.0);
                                
                                let (status_color, status_text) = match t.status {
                                    TournamentStatus::Draft => (ui::theme::TEXT_MUTED, "Draft"),
                                    TournamentStatus::InProgress => (ui::theme::ACCENT_BRONZE, "In Progress"),
                                    TournamentStatus::Completed => (ui::theme::SUCCESS, "Completed"),
                                };
                                ui.add(
                                    egui::Button::new(egui::RichText::new(status_text).size(11.0).color(status_color))
                                        .fill(egui::Color32::TRANSPARENT)
                                        .stroke(egui::Stroke::new(1.0, status_color))
                                        .corner_radius(10),
                                );
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(egui::Button::new(egui::RichText::new("📤 Export JSON").size(12.0).color(ui::theme::TEXT_SECONDARY)).fill(ui::theme::BG_CARD)).clicked() {
                                    self.export_json();
                                }
                                if !self.is_draft() {
                                    if ui.add(egui::Button::new(egui::RichText::new("🔄 Reset").size(12.0).color(ui::theme::WARNING)).fill(ui::theme::BG_CARD)).clicked() {
                                        self.reset_bracket();
                                    }
                                }
                            });
                        });

                        ui.add_space(16.0);

                        // Horizontal Tabs
                        ui.horizontal(|ui| {
                            let tabs = [
                                (TournamentTab::Overview, "Overview"),
                                (TournamentTab::Participants, "Participants"),
                                (TournamentTab::Bracket, "Bracket"),
                                (TournamentTab::Standings, "Standings"),
                                (TournamentTab::Scoreboard, "Scoreboard"),
                            ];

                            for (tab, label) in tabs {
                                let is_active = self.active_tab == tab;
                                let color = if is_active { ui::theme::ACCENT_BRONZE } else { ui::theme::TEXT_SECONDARY };
                                let text = egui::RichText::new(label).size(14.0).color(color).strong();
                                
                                // Tab styling
                                let btn = egui::Button::new(text)
                                    .fill(egui::Color32::TRANSPARENT)
                                    .stroke(egui::Stroke::NONE)
                                    .min_size(egui::Vec2::new(100.0, 30.0));
                                
                                let response = ui.add(btn);
                                if response.clicked() {
                                    self.active_tab = tab;
                                }

                                // Bottom active indicator
                                if is_active {
                                    let rect = response.rect;
                                    ui.painter().line_segment(
                                        [egui::pos2(rect.min.x, rect.max.y + 4.0), egui::pos2(rect.max.x, rect.max.y + 4.0)],
                                        egui::Stroke::new(3.0, ui::theme::ACCENT_BRONZE)
                                    );
                                }
                            }
                        });
                    });

                // Content area (based on selected tab)
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(ui::theme::BG_DARK).inner_margin(egui::Margin::same(24)))
                    .show(ctx, |ui| {
                        match self.active_tab {
                            TournamentTab::Overview => {
                                // Simple overview
                                if let Some(ref t) = self.active_tournament {
                                    ui.label(ui::theme::subheading_text("Game"));
                                    ui.label(ui::theme::body_text(if t.game_name.is_empty() { "Not specified" } else { &t.game_name }));
                                    ui.add_space(16.0);
                                    ui.label(ui::theme::subheading_text("Format"));
                                    ui.label(ui::theme::body_text(t.tournament_type.as_str()));
                                    ui.add_space(16.0);
                                    ui.label(ui::theme::subheading_text("Description"));
                                    ui.label(ui::theme::body_text(if t.description.is_empty() { "No description" } else { &t.description }));
                                }
                            }
                            TournamentTab::Participants => {
                                ui::participant_panel::render(self, ui, &ctx_clone);
                            }
                            TournamentTab::Bracket => {
                                ui::bracket_view::render(self, ui);
                            }
                            TournamentTab::Standings => {
                                ui::stats_panel::render(self, ui);
                            }
                            TournamentTab::Scoreboard => {
                                ui::scoreboard_panel::render(self, ui);
                            }
                        }
                    });

                // Match Modal Popup
                if self.show_match_modal {
                    ui::match_panel::render_modal(self, ctx);
                }
            }
        }
    }
}
