mod app;
mod database;
mod domain;
mod services;
mod ui;
mod utils;

use app::TourviaApp;
use database::Database;

fn main() -> eframe::Result<()> {
    // Initialize logger
    env_logger::init();

    // Open database
    let db = Database::open("tourvia.db").expect("Failed to open database");

    // Configure window
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Tourvia — Tournament Visualization & Administration")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "Tourvia",
        options,
        Box::new(|cc| {
            // Install image loaders for egui to load textures
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let mut app = TourviaApp::new(db);
            app.refresh_tournaments();
            app.load_rosters();
            Ok(Box::new(app))
        }),
    )
}
