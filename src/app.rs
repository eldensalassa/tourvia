use std::collections::HashMap;

use crate::database::Database;
use crate::domain::match_model::Match;
use crate::domain::participant::Participant;
use crate::domain::round::Round;
use crate::domain::tournament::{Tournament, TournamentStatus, TournamentType};
use crate::services::{
    bracket_generator::BracketGeneratorService,
    match_service::MatchService,
    tournament_service::TournamentService,
};
use crate::domain::repositories::*;
use std::sync::Arc;
use crate::ui;

use rand::seq::SliceRandom;

// ─── View Enum ──────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Dashboard,
    TournamentForm,
    TournamentDetail,
    GlobalRoster,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TournamentTab {
    Overview,
    Participants,
    Bracket,
    Standings,
}


// ─── Application State ─────────────────────────────

pub struct AppServices {
    pub tournament: TournamentService,
    pub match_service: MatchService,
    pub bracket_generator: BracketGeneratorService,
}

pub struct TourviaApp {
    pub db: Arc<Database>,
    pub services: AppServices,
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

    // Status message
    pub notifications: crate::ui::notifications::NotificationQueue,

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

    // Global Roster
    pub global_rosters: Vec<crate::domain::roster::Roster>,
    pub new_roster_name: String,
    pub new_roster_game: String, // Kept for the dropdown selection state
    pub new_roster_logo: Option<Vec<u8>>,
    pub active_roster: Option<crate::domain::roster::Roster>,
    pub roster_members: Vec<crate::domain::roster::RosterMember>,
    pub new_member_name: String,
    pub new_member_photo: Option<Vec<u8>>,
    pub member_photo_textures: std::collections::HashMap<String, egui::TextureHandle>,
    
    // Games
    pub global_games: Vec<crate::domain::game::Game>,
    pub new_game_name: String,
}

impl TourviaApp {
    pub fn new(db_instance: Database) -> Self {
        let db = Arc::new(db_instance);
        
        let services = AppServices {
            tournament: TournamentService::new(db.clone(), db.clone(), db.clone(), db.clone()),
            match_service: MatchService::new(db.clone(), db.clone(), db.clone()),
            bracket_generator: BracketGeneratorService::new(db.clone(), db.clone()),
        };

        let tournaments = services.tournament.load_all_tournaments().unwrap_or_default();

        let mut app = Self {
            db,
            services,
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
            notifications: crate::ui::notifications::NotificationQueue::new(),
            search_query: String::new(),
            confirm_delete: None,
            show_stats: false,
            bulk_add_text: String::new(),
            show_bulk_add: false,
            logo_textures: HashMap::new(),
            logos_loaded_for: None,
            bracket_zoom: 1.0,
            global_rosters: Vec::new(),
            new_roster_name: String::new(),
            new_roster_game: String::new(),
            new_roster_logo: None,
            active_roster: None,
            roster_members: Vec::new(),
            new_member_name: String::new(),
            new_member_photo: None,
            member_photo_textures: std::collections::HashMap::new(),
            global_games: Vec::new(),
            new_game_name: String::new(),
        };
        app.load_games();
        app.load_rosters();
        app
    }

    pub fn load_rosters(&mut self) {
        use crate::domain::repositories::RosterRepository;
        match self.db.get_rosters() {
            Ok(r) => self.global_rosters = r,
            Err(e) => self.notifications.error(format!("Failed to load rosters: {}", e)),
        }
    }

    pub fn open_roster(&mut self, idx: usize) {
        if idx < self.global_rosters.len() {
            self.active_roster = Some(self.global_rosters[idx].clone());
            self.new_member_name.clear();
            self.new_member_photo = None;
            self.member_photo_textures.clear();
            self.load_active_roster_members();
        }
    }

    pub fn close_roster(&mut self) {
        self.active_roster = None;
        self.roster_members.clear();
        self.member_photo_textures.clear();
        self.new_member_name.clear();
        self.new_member_photo = None;
    }

    pub fn load_active_roster_members(&mut self) {
        if let Some(r) = &self.active_roster {
            use crate::domain::repositories::RosterRepository;
            match self.db.get_roster_members(&r.id) {
                Ok(members) => self.roster_members = members,
                Err(e) => self.notifications.error(e),
            }
        }
    }

    pub fn ensure_member_photos_loaded(&mut self, ctx: &egui::Context) {
        for m in &self.roster_members {
            if m.profile_picture.is_some() && !self.member_photo_textures.contains_key(&m.id) {
                if let Some(data) = &m.profile_picture {
                    if let Some(texture) = Self::decode_logo(ctx, &m.id, data) {
                        self.member_photo_textures.insert(m.id.clone(), texture);
                    }
                }
            }
        }
    }

    pub fn load_games(&mut self) {
        use crate::domain::repositories::GameRepository;
        match self.db.get_games() {
            Ok(g) => self.global_games = g,
            Err(e) => self.notifications.error(format!("Failed to load games: {}", e)),
        }
    }

    // ─── Navigation ─────────────────────────────────

    pub fn go_to_dashboard(&mut self) {
        self.current_view = View::Dashboard;
        self.active_tournament = None;
        
        self.confirm_delete = None;
        self.refresh_tournaments();
    }

    pub fn show_tournament_form(&mut self) {
        self.current_view = View::TournamentForm;
        self.new_tournament_name.clear();
        self.new_tournament_type = TournamentType::SingleElimination;
        self.new_tournament_description.clear();
        self.new_tournament_game.clear();
        
    }

    pub fn open_tournament(&mut self, idx: usize) {
        if idx < self.tournaments.len() {
            let tournament = self.tournaments[idx].clone();
            self.active_tournament = Some(tournament.clone());
            self.current_view = View::TournamentDetail;
            self.active_tab = TournamentTab::Overview;
            
            self.selected_match = None;
            self.show_match_modal = false;
            self.score_input = [String::new(), String::new()];
            self.bracket_zoom = 1.0;
            self.load_tournament_data(&tournament.id);
        }
    }

    // ─── Data Loading ───────────────────────────────

    pub fn refresh_tournaments(&mut self) {
        self.tournaments = self.services.tournament.load_all_tournaments().unwrap_or_default();
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

        self.champion_name = self.services.match_service.get_champion(tournament_id).unwrap_or(None);

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

    pub fn decode_logo(ctx: &egui::Context, name: &str, data: &[u8]) -> Option<egui::TextureHandle> {
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
        match self.services.tournament.create_tournament(
            &self.new_tournament_name,
            self.new_tournament_type.clone(),
            &self.new_tournament_description,
            &self.new_tournament_game,
        ) {
            Ok(tournament) => {
                self.notifications.success(format!("Tournament '{}' created!", tournament.name));
                self.active_tournament = Some(tournament.clone());
                self.current_view = View::TournamentDetail;
                self.active_tab = TournamentTab::Overview;
                self.participants.clear();
                self.rounds.clear();
                self.matches.clear();
                self.refresh_tournaments();
            }
            Err(e) => {
                self.notifications.error(e);
            }
        }
    }

    pub fn delete_tournament_at(&mut self, idx: usize) {
        if idx < self.tournaments.len() {
            let id = self.tournaments[idx].id.clone();
            if let Err(e) = self.services.tournament.delete_tournament(&id) {
                self.notifications.error(e);
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
        match self.services.tournament.reset_bracket(&tid) {
            Ok(_) => {
                self.load_tournament_data(&tid);
                self.refresh_tournaments();
                if let Ok(Some(t)) = self.db.get_tournament(&tid) {
                    self.active_tournament = Some(t);
                }
                self.notifications.info("Bracket reset to Draft.");
            }
            Err(e) => {
                self.notifications.error(e);
            }
        }
    }

    pub fn export_json(&mut self) {
        let tid = match &self.active_tournament {
            Some(t) => t.id.clone(),
            None => return,
        };
        match self.services.tournament.export_tournament_json(&tid) {
            Ok(json) => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_file_name("tournament_export.json")
                    .save_file()
                {
                    match std::fs::write(&path, &json) {
                        Ok(_) => {
                            self.notifications.success(format!("Exported to {}", path.display()));
                        }
                        Err(e) => {
                            self.notifications.error(format!("Failed to write file: {}", e));
                        }
                    }
                }
            }
            Err(e) => {
                self.notifications.error(e);
            }
        }
    }

    pub fn import_json(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(json_str) => {
                    match self.services.tournament.import_tournament_json(&json_str) {
                        Ok(tid) => {
                            self.notifications.success("Tournament imported successfully!");
                            self.refresh_tournaments();
                            if let Some(idx) = self.tournaments.iter().position(|t| t.id == tid) {
                                self.open_tournament(idx);
                            }
                        }
                        Err(e) => {
                            self.notifications.error(e);
                        }
                    }
                }
                Err(e) => {
                    self.notifications.error(format!("Failed to read file: {}", e));
                }
            }
        }
    }

    // ─── Participant Actions ────────────────────────

    pub fn add_participant(&mut self) {
        let name = self.new_participant_name.trim().to_string();
        if name.is_empty() {
            self.notifications.error("Name cannot be empty.");
            return;
        }

        let tid = if let Some(ref tournament) = self.active_tournament {
            tournament.id.clone()
        } else { return; };

        match self.db.participant_exists(&tid, &name) {
            Ok(true) => {
                self.notifications.error(format!("'{}' already exists.", name));
                return;
            }
            Err(e) => {
                self.notifications.error(format!("Database error: {}", e));
                return;
            }
            _ => {}
        }

        let seed = self.participants.len() as i32 + 1;
        let participant = crate::domain::participant::Participant::new(tid.clone(), name.clone(), seed);

        match self.db.create_participant(&participant) {
            Ok(_) => {
                // If the team is in the global roster and has a logo, copy it.
                if let Some(roster_team) = self.global_rosters.iter().find(|r| r.name == participant.name) {
                    if let Some(ref logo_data) = roster_team.logo_data {
                        use crate::domain::repositories::ParticipantRepository;
                        let _ = self.db.set_participant_logo(&participant.id, logo_data);
                    }
                }

                self.new_participant_name.clear();
                self.load_tournament_data(&tid);
                let count = self.participants.len();
                let _ = self.db.update_tournament_participant_count(&tid, count);
                self.notifications.success(format!("'{}' added", participant.name));
                self.refresh_tournaments();
            }
            Err(e) => {
                self.notifications.error(format!("Failed to add: {}", e));
            }
        }
    }

    pub fn bulk_add_participants(&mut self) {
        let tid = match &self.active_tournament { Some(t) => t.id.clone(), None => return };

        let names: Vec<String> = self.bulk_add_text.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();

        if names.is_empty() {
            self.notifications.error("No names provided.");
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
        self.notifications.success(format!("{} participant(s) added.", added));
    }

    pub fn delete_participant(&mut self, idx: usize) {
        if idx < self.participants.len() {
            let p_id = self.participants[idx].id.clone();
            let tid = if let Some(ref tournament) = self.active_tournament { tournament.id.clone() } else { return };

            self.logo_textures.remove(&p_id);

            if let Err(e) = self.db.delete_participant(&p_id) {
                self.notifications.error(format!("Failed to delete: {}", e));
            } else {
                self.load_tournament_data(&tid);
                let count = self.participants.len();
                let _ = self.db.update_tournament_participant_count(&tid, count);
                self.notifications.info("Participant removed.");
                self.refresh_tournaments();
            }
        }
    }

    pub fn move_participant_up(&mut self, idx: usize) {
        if idx == 0 || idx >= self.participants.len() { return; }
        self.move_participant_to(idx, idx - 1);
    }

    pub fn move_participant_down(&mut self, idx: usize) {
        if idx + 1 >= self.participants.len() { return; }
        self.move_participant_to(idx, idx + 1);
    }

    pub fn move_participant_to(&mut self, from_idx: usize, to_idx: usize) {
        if from_idx == to_idx || from_idx >= self.participants.len() || to_idx >= self.participants.len() { return; }
        let tid = match &self.active_tournament { Some(t) => t.id.clone(), None => return };

        let mut new_order = self.participants.clone();
        let item = new_order.remove(from_idx);
        new_order.insert(to_idx, item);

        for (i, p) in new_order.iter_mut().enumerate() {
            let new_seed = (i + 1) as i32;
            if p.seed != new_seed {
                p.seed = new_seed;
                let _ = self.db.update_participant(&p.id, &p.name, p.seed);
            }
        }
        self.load_tournament_data(&tid);
    }

    pub fn process_logo(raw_bytes: &[u8]) -> Result<Vec<u8>, String> {
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
        self.notifications.success("Seeds randomized!");
    }

    // ─── Bracket Actions ────────────────────────────

    pub fn generate_bracket(&mut self) {
        let (tid, ttype) = if let Some(ref tournament) = self.active_tournament {
            (tournament.id.clone(), tournament.tournament_type.clone())
        } else { return; };

        match self.services.bracket_generator.generate_bracket(&tid, &self.participants, &ttype) {
            Ok(_) => {
                let _ = self.services.tournament.update_status(&tid, TournamentStatus::InProgress);
                self.load_tournament_data(&tid);
                self.refresh_tournaments();
                if let Ok(Some(t)) = self.db.get_tournament(&tid) {
                    self.active_tournament = Some(t);
                }
                self.active_tab = TournamentTab::Bracket; // Auto-switch to bracket tab
                self.notifications.success("Bracket generated successfully!");
            }
            Err(e) => {
                self.notifications.error(e);
            }
        }
    }

    // ─── Match Actions ──────────────────────────────

    pub fn select_match(&mut self, match_id: &str) {
        self.selected_match = Some(match_id.to_string());
        self.score_input = [String::new(), String::new()];
        
        self.show_match_modal = true; // Open modal
    }

    pub fn submit_match_score(&mut self) {
        let match_id = match &self.selected_match { Some(id) => id.clone(), None => return };

        let s1: i32 = match self.score_input[0].trim().parse() {
            Ok(v) => v,
            Err(_) => {
                self.notifications.error("Invalid score for Player 1.");
                return;
            }
        };

        let s2: i32 = match self.score_input[1].trim().parse() {
            Ok(v) => v,
            Err(_) => {
                self.notifications.error("Invalid score for Player 2.");
                return;
            }
        };

        match self.services.match_service.submit_score(&match_id, s1, s2) {
            Ok(_) => {
                if let Some(ref tournament) = self.active_tournament {
                    let tid = tournament.id.clone();
                    self.load_tournament_data(&tid);

                    if let Ok(true) = self.services.match_service.is_tournament_complete(&tid) {
                        let _ = self.services.tournament.update_status(&tid, TournamentStatus::Completed);
                        if let Ok(Some(t)) = self.db.get_tournament(&tid) { self.active_tournament = Some(t); }
                        self.refresh_tournaments();
                    }
                }
                self.notifications.success("Score submitted!");
                self.score_input = [String::new(), String::new()];
                self.show_match_modal = false; // Close modal on success
            }
            Err(e) => {
                self.notifications.error(e);
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
                egui::CentralPanel::default().show(ctx, |ui| {
                    crate::ui::dashboard::render(self, ui);
                });
            }
            View::TournamentForm => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    crate::ui::tournament_form::render(self, ui);
                });
            }
            View::GlobalRoster => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    crate::ui::global_roster::render(self, ui);
                });
            }
            View::TournamentDetail => {
                self.ensure_logos_loaded(ctx);
                let ctx_clone = ctx.clone();

                // Top bar with Title & Tabs
                egui::TopBottomPanel::top("top_nav")
                    .frame(egui::Frame::new().fill(ui::theme::BG_PANEL()).inner_margin(egui::Margin::symmetric(24, 16)))
                    .show(ctx, |ui| {
                        
                        // Top line: Back button, Title, Actions
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new(egui::RichText::new("← Dashboard").color(ui::theme::TEXT_MUTED()).size(13.0)).fill(egui::Color32::TRANSPARENT)).clicked() {
                                self.go_to_dashboard();
                                return;
                            }
                            
                            ui.add_space(8.0);
                            
                            if let Some(ref t) = self.active_tournament {
                                ui.label(ui::theme::heading_text(&t.name));
                                ui.add_space(8.0);
                                
                                let (status_color, status_text) = match t.status {
                                    TournamentStatus::Draft => (ui::theme::TEXT_MUTED(), "Draft"),
                                    TournamentStatus::InProgress => (ui::theme::ACCENT_BRONZE(), "In Progress"),
                                    TournamentStatus::Completed => (ui::theme::SUCCESS(), "Completed"),
                                };
                                ui.add(
                                    egui::Button::new(egui::RichText::new(status_text).size(11.0).color(status_color))
                                        .fill(egui::Color32::TRANSPARENT)
                                        .stroke(egui::Stroke::new(1.0, status_color))
                                        .corner_radius(10),
                                );
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(egui::Button::new(egui::RichText::new("📤 Export JSON").size(12.0).color(ui::theme::TEXT_SECONDARY())).fill(ui::theme::BG_CARD())).clicked() {
                                    self.export_json();
                                }
                                if !self.is_draft() {
                                    if ui.add(egui::Button::new(egui::RichText::new("🔄 Reset").size(12.0).color(ui::theme::WARNING())).fill(ui::theme::BG_CARD())).clicked() {
                                        self.reset_bracket();
                                    }
                                }
                            });
                        });

                        ui.add_space(16.0);

                        // Horizontal Tabs (Segmented Control Style)
                        egui::Frame::new()
                            .fill(ui::theme::BG_CARD())
                            .corner_radius(ui::theme::button_rounding())
                            .inner_margin(egui::Margin::same(4))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let tabs = [
                                        (TournamentTab::Overview, "Overview"),
                                        (TournamentTab::Participants, "Participants"),
                                        (TournamentTab::Bracket, "Bracket"),
                                        (TournamentTab::Standings, "Standings"),
                                    ];

                                    for (tab, label) in tabs {
                                        let is_active = self.active_tab == tab;
                                        
                                        let (bg_color, text_color) = if is_active {
                                            (ui::theme::BG_ELEVATED(), ui::theme::ACCENT_BRONZE())
                                        } else {
                                            (egui::Color32::TRANSPARENT, ui::theme::TEXT_MUTED())
                                        };

                                        let text = egui::RichText::new(label)
                                            .size(14.0)
                                            .color(text_color)
                                            .strong();
                                        
                                        let btn = egui::Button::new(text)
                                            .fill(bg_color)
                                            .stroke(egui::Stroke::NONE)
                                            .corner_radius(ui::theme::button_rounding())
                                            .min_size(egui::Vec2::new(120.0, 36.0));
                                        
                                        if ui.add(btn).clicked() {
                                            self.active_tab = tab;
                                        }
                                    }
                                });
                            });
                    });

                // Content area (based on selected tab)
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(ui::theme::BG_DARK()).inner_margin(egui::Margin::same(24)))
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
                        }
                    });

                // Match Modal Popup
                if self.show_match_modal {
                    ui::match_panel::render_modal(self, ctx);
                }
            }
        }
        self.notifications.render(ctx);
    }
}
