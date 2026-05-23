use egui::{Align2, Color32, Frame, Id, Margin, RichText, Stroke, Ui, Vec2};
use std::time::{Duration, Instant};

use crate::ui::theme;

#[derive(Clone, Copy, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Info,
}

struct Toast {
    id: Id,
    message: String,
    toast_type: ToastType,
    created_at: Instant,
    duration: Duration,
}

pub struct NotificationQueue {
    toasts: Vec<Toast>,
}

impl NotificationQueue {
    pub fn new() -> Self {
        Self { toasts: Vec::new() }
    }

    pub fn add(&mut self, message: impl Into<String>, toast_type: ToastType, duration: Duration) {
        let msg_str = message.into();
        self.toasts.push(Toast {
            id: Id::new(msg_str.clone() + &rand::random::<u64>().to_string()),
            message: msg_str,
            toast_type,
            created_at: Instant::now(),
            duration,
        });
    }

    pub fn success(&mut self, message: impl Into<String>) {
        self.add(message, ToastType::Success, Duration::from_secs(3));
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.add(message, ToastType::Error, Duration::from_secs(5));
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.add(message, ToastType::Info, Duration::from_secs(3));
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        self.toasts.retain(|t| now.duration_since(t.created_at) < t.duration);

        if self.toasts.is_empty() {
            return;
        }

        let mut offset_y = -20.0;
        let screen_rect = ctx.screen_rect();

        for toast in self.toasts.iter().rev() {
            let color = match toast.toast_type {
                ToastType::Success => theme::SUCCESS(),
                ToastType::Error => theme::ERROR(),
                ToastType::Info => theme::INFO(),
            };

            let bg_color = match toast.toast_type {
                ToastType::Success => Color32::from_rgb(20, 40, 20),
                ToastType::Error => Color32::from_rgb(40, 20, 20),
                ToastType::Info => Color32::from_rgb(20, 30, 40),
            };

            let remaining = toast.duration.as_secs_f32() - now.duration_since(toast.created_at).as_secs_f32();
            let mut alpha = 1.0;
            if remaining < 0.5 {
                alpha = remaining / 0.5;
            } else if now.duration_since(toast.created_at).as_secs_f32() < 0.2 {
                alpha = now.duration_since(toast.created_at).as_secs_f32() / 0.2;
            }

            let frame = Frame::new()
                .fill(bg_color.linear_multiply(alpha))
                .stroke(Stroke::new(1.0, color.linear_multiply(alpha)))
                .corner_radius(4)
                .inner_margin(Margin::same(12));

            let res = egui::Window::new("Toast")
                .id(toast.id)
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .frame(frame)
                .fixed_pos(egui::pos2(screen_rect.width() - 20.0, screen_rect.height() + offset_y))
                .pivot(egui::Align2::RIGHT_BOTTOM)
                .show(ctx, |ui| {
                    ui.set_min_width(240.0);
                    ui.horizontal(|ui| {
                        let icon = match toast.toast_type {
                            ToastType::Success => "✅",
                            ToastType::Error => "❌",
                            ToastType::Info => "ℹ",
                        };
                        ui.label(RichText::new(icon).color(color.linear_multiply(alpha)));
                        ui.label(RichText::new(&toast.message).color(Color32::WHITE.linear_multiply(alpha)));
                    });
                });
                
            if let Some(r) = res {
                offset_y -= r.response.rect.height() + 10.0;
            }
        }
        
        ctx.request_repaint(); // Keep repainting to animate fading
    }
}
