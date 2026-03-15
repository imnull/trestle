//! Trestle Client - 桌面 GUI 客户端

mod app;
mod pages;
mod api;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Trestle"),
        ..Default::default()
    };

    eframe::run_native(
        "Trestle",
        options,
        Box::new(|cc| Ok(Box::new(app::TrestleApp::new(cc)))),
    )
}
