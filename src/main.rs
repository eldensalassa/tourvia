#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod database;
mod domain;
mod services;
mod ui;
mod utils;

use app::TourviaApp;
use database::Database;

fn load_icon() -> Option<egui::IconData> {
    if let Ok(image) = image::load_from_memory(include_bytes!("assets/logo.png")) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        Some(egui::IconData {
            rgba,
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> eframe::Result<()> {
    // Initialize logger
    env_logger::init();

    // Determine safe data directory
    let db_path = if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "Tourvia") {
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir).unwrap_or_default();
        data_dir.join("tourvia.db")
    } else {
        std::path::PathBuf::from("tourvia.db")
    };

    // Open database
    let db = Database::open(db_path.to_str().unwrap()).expect("Failed to open database");

    // Configure window
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Tourvia")
        .with_inner_size([1000.0, 650.0])
        .with_min_inner_size([900.0, 600.0]);

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "Tourvia",
        options,
        Box::new(|cc| {
            // Install image loaders for egui to load textures
            egui_extras::install_image_loaders(&cc.egui_ctx);
            
            // Bundle the cross-platform open-source Anton font (Impact alternative)
            let mut fonts = egui::FontDefinitions::default();
            let font_data = include_bytes!("assets/Anton-Regular.ttf");
            fonts.font_data.insert("Impact".to_owned(), std::sync::Arc::new(egui::FontData::from_static(font_data)));
            fonts.families.insert(egui::FontFamily::Name("Impact".into()), vec!["Impact".to_owned()]);
            cc.egui_ctx.set_fonts(fonts);
            
            let mut app = TourviaApp::new(db);
            app.refresh_tournaments();
            app.load_rosters();
            Ok(Box::new(app))
        }),
    )
}
