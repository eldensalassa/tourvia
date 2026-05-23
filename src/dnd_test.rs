fn main() {
    let _ = egui::Id::new("test");
}

fn test(ui: &mut egui::Ui) {
    let id = egui::Id::new("drag");
    let response = ui.dnd_drag_source(id, |ui| {
        ui.label("drag me");
    });
}
