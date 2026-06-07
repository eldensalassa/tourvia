use egui::{self, RichText, Stroke, Ui, Vec2};

use crate::app::TourviaApp;
use crate::domain::match_model::{Match, MatchStatus};
use crate::domain::tournament::{Tournament, TournamentStatus};
use crate::ui::components::badge::Badge;
use crate::ui::theme;

struct MatchCounts {
    total: usize,
    completed: usize,
    pending: usize,
    in_progress: usize,
    bye: usize,
}

impl MatchCounts {
    fn from_matches(matches: &[Match]) -> Self {
        Self {
            total: matches.len(),
            completed: matches
                .iter()
                .filter(|m| m.status == MatchStatus::Completed)
                .count(),
            pending: matches
                .iter()
                .filter(|m| m.status == MatchStatus::Pending)
                .count(),
            in_progress: matches
                .iter()
                .filter(|m| m.status == MatchStatus::InProgress)
                .count(),
            bye: matches
                .iter()
                .filter(|m| m.status == MatchStatus::Bye)
                .count(),
        }
    }

    fn resolved(&self) -> usize {
        self.completed + self.bye
    }

    fn progress(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.resolved() as f32 / self.total as f32
        }
    }
}

pub fn render(app: &mut TourviaApp, ui: &mut Ui) {
    let Some(tournament) = app.active_tournament.clone() else {
        ui.label(theme::label_text("No active tournament."));
        return;
    };

    let counts = MatchCounts::from_matches(&app.matches);
    let focus_match = focus_match(&app.matches);
    let champion = app.champion.clone();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            render_summary(app, ui, &tournament, &counts, champion.as_ref());
            ui.add_space(14.0);

            render_stats(app, ui, &counts);
            ui.add_space(14.0);

            render_description(ui, &tournament);

            if counts.total > 0 || champion.is_some() {
                ui.add_space(14.0);
                render_match_focus(app, ui, focus_match.as_ref(), champion.as_ref());
            }
        });
}

fn render_summary(
    app: &TourviaApp,
    ui: &mut Ui,
    tournament: &Tournament,
    counts: &MatchCounts,
    champion: Option<&(String, String)>,
) {
    egui::Frame::new()
        .fill(theme::BG_PANEL())
        .stroke(theme::card_stroke())
        .corner_radius(theme::card_rounding())
        .inner_margin(egui::Margin::same(18))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                render_tournament_mark(app, ui, tournament);
                ui.add_space(14.0);

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(theme::subheading_text(&tournament.name).size(24.0));
                        ui.add_space(8.0);
                        status_badge(ui, &tournament.status);
                    });

                    ui.add_space(8.0);
                    ui.horizontal_wrapped(|ui| {
                        metadata_chip(ui, "Game", value_or_default(&tournament.game_name, "Not specified"));
                        metadata_chip(ui, "Format", tournament.tournament_type.as_str());
                        metadata_chip(ui, "Created", created_date(&tournament.created_at));
                    });

                    ui.add_space(8.0);
                    if let Some((name, _id)) = champion {
                        ui.label(
                            RichText::new(format!("Champion: {}", name))
                                .size(14.0)
                                .color(theme::GOLD())
                                .strong(),
                        );
                    } else if counts.total > 0 {
                        ui.label(
                            RichText::new(format!(
                                "{} of {} matches resolved",
                                counts.resolved(),
                                counts.total
                            ))
                            .size(13.0)
                            .color(theme::TEXT_SECONDARY()),
                        );
                    } else {
                        ui.label(
                            RichText::new("Bracket has not been generated yet.")
                                .size(13.0)
                                .color(theme::TEXT_SECONDARY()),
                        );
                    }
                });
            });
        });
}

fn render_tournament_mark(app: &TourviaApp, ui: &mut Ui, tournament: &Tournament) {
    if let Some(texture) = app.tournament_logo_textures.get(&tournament.id) {
        ui.add(
            egui::Image::new(texture)
                .fit_to_exact_size(Vec2::new(64.0, 64.0))
                .corner_radius(4),
        );
        return;
    }

    let (rect, _) = ui.allocate_exact_size(Vec2::new(64.0, 64.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);
    ui.painter().rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, theme::BORDER_SUBTLE()),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "TV",
        egui::FontId::new(20.0, egui::FontFamily::Name("Impact".into())),
        theme::TEXT_MUTED(),
    );
}

fn render_stats(app: &TourviaApp, ui: &mut Ui, counts: &MatchCounts) {
    ui.columns(3, |columns| {
        stat_card(
            &mut columns[0],
            "Participants",
            app.participants.len().to_string(),
            "Registered teams",
            theme::TEXT_PRIMARY(),
        );
        stat_card(
            &mut columns[1],
            "Progress",
            format!("{}%", (counts.progress() * 100.0).round() as i32),
            if counts.total == 0 { "No bracket yet" } else { "Matches resolved" },
            theme::ACCENT_BRONZE(),
        );
        stat_card(
            &mut columns[2],
            "Live",
            counts.in_progress.to_string(),
            if counts.pending > 0 { "Matches waiting" } else { "Open matches" },
            if counts.in_progress > 0 {
                theme::WARNING()
            } else {
                theme::TEXT_SECONDARY()
            },
        );
    });
}

fn render_description(ui: &mut Ui, tournament: &Tournament) {
    egui::Frame::new()
        .fill(theme::BG_PANEL())
        .stroke(theme::card_stroke())
        .corner_radius(theme::card_rounding())
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.label(theme::section_header("Description"));
            ui.add_space(8.0);
            ui.label(
                RichText::new(value_or_default(
                    &tournament.description,
                    "No description provided.",
                ))
                .size(14.0)
                .color(theme::TEXT_SECONDARY()),
            );
        });
}

fn render_match_focus(
    app: &TourviaApp,
    ui: &mut Ui,
    focus_match: Option<&Match>,
    champion: Option<&(String, String)>,
) {
    egui::Frame::new()
        .fill(theme::BG_PANEL())
        .stroke(theme::card_stroke())
        .corner_radius(theme::card_rounding())
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.label(theme::section_header("Match Focus"));
            ui.add_space(10.0);

            if let Some(m) = focus_match {
                ui.horizontal(|ui| {
                    match_badge(ui, &m.status);
                    ui.label(
                        RichText::new(format!(
                            "{} - Match {}",
                            round_name(app, &m.round_id),
                            m.match_order + 1
                        ))
                        .size(12.0)
                        .color(theme::TEXT_MUTED()),
                    );
                });

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(player_name(&m.player1_name))
                            .size(17.0)
                            .color(theme::TEXT_PRIMARY())
                            .strong(),
                    );
                    ui.label(RichText::new("vs").size(12.0).color(theme::TEXT_MUTED()));
                    ui.label(
                        RichText::new(player_name(&m.player2_name))
                            .size(17.0)
                            .color(theme::TEXT_PRIMARY())
                            .strong(),
                    );
                });

                if m.status == MatchStatus::Completed || m.status == MatchStatus::InProgress {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("Score {} : {}", m.score1, m.score2))
                            .size(16.0)
                            .color(theme::ACCENT_BRONZE_LIGHT())
                            .strong(),
                    );
                }
            } else if let Some((name, _id)) = champion {
                ui.label(RichText::new("Tournament complete").size(13.0).color(theme::TEXT_SECONDARY()));
                ui.add_space(8.0);
                ui.label(theme::champion_text(name));
            } else {
                ui.label(
                    RichText::new("No playable match is available.")
                        .size(14.0)
                        .color(theme::TEXT_SECONDARY()),
                );
            }
        });
}

fn stat_card(ui: &mut Ui, label: &str, value: String, detail: &str, color: egui::Color32) {
    egui::Frame::new()
        .fill(theme::BG_PANEL())
        .stroke(theme::card_stroke())
        .corner_radius(theme::card_rounding())
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            ui.set_min_height(72.0);
            ui.label(
                RichText::new(label.to_uppercase())
                    .size(11.0)
                    .color(theme::TEXT_MUTED())
                    .strong(),
            );
            ui.add_space(7.0);
            ui.label(
                RichText::new(value)
                    .font(egui::FontId::new(28.0, egui::FontFamily::Name("Impact".into())))
                    .color(color)
                    .strong(),
            );
            ui.add_space(2.0);
            ui.label(RichText::new(detail).size(12.0).color(theme::TEXT_SECONDARY()));
        });
}

fn metadata_chip(ui: &mut Ui, label: &str, value: &str) {
    egui::Frame::new()
        .fill(ui.visuals().extreme_bg_color)
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE()))
        .corner_radius(theme::badge_rounding())
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(label.to_uppercase())
                        .size(10.0)
                        .color(theme::TEXT_MUTED())
                        .strong(),
                );
                ui.label(RichText::new(value).size(12.0).color(theme::TEXT_PRIMARY()));
            });
        });
}

fn focus_match(matches: &[Match]) -> Option<Match> {
    matches
        .iter()
        .find(|m| m.status == MatchStatus::InProgress)
        .or_else(|| {
            matches.iter().find(|m| {
                m.status == MatchStatus::Pending
                    && !m.player1_name.is_empty()
                    && !m.player2_name.is_empty()
            })
        })
        .or_else(|| matches.iter().find(|m| m.status == MatchStatus::Pending))
        .or_else(|| {
            matches
                .iter()
                .rev()
                .find(|m| m.status == MatchStatus::Completed)
        })
        .cloned()
}

fn status_badge(ui: &mut Ui, status: &TournamentStatus) {
    match status {
        TournamentStatus::Draft => {
            ui.add(Badge::neutral(status.as_str()));
        }
        TournamentStatus::InProgress => {
            ui.add(Badge::warning(status.as_str()));
        }
        TournamentStatus::Completed => {
            ui.add(Badge::success(status.as_str()));
        }
    };
}

fn match_badge(ui: &mut Ui, status: &MatchStatus) {
    match status {
        MatchStatus::Pending => {
            ui.add(Badge::neutral(status.as_str()));
        }
        MatchStatus::InProgress => {
            ui.add(Badge::warning(status.as_str()));
        }
        MatchStatus::Completed => {
            ui.add(Badge::success(status.as_str()));
        }
        MatchStatus::Bye => {
            ui.add(Badge::info(status.as_str()));
        }
    };
}

fn round_name(app: &TourviaApp, round_id: &str) -> String {
    app.rounds
        .iter()
        .find(|r| r.id == round_id)
        .map(|r| r.name.clone())
        .unwrap_or_else(|| "Match".to_string())
}

fn player_name(name: &str) -> &str {
    if name.trim().is_empty() {
        "TBD"
    } else {
        name
    }
}

fn value_or_default<'a>(value: &'a str, default: &'a str) -> &'a str {
    if value.trim().is_empty() {
        default
    } else {
        value
    }
}

fn created_date(created_at: &str) -> &str {
    created_at.split(' ').next().unwrap_or(created_at)
}
