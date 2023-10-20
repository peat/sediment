use std::sync::mpsc::Receiver;

use crate::builder::BuilderUpdate;
use eframe::{egui, epaint::ColorImage, App, CreationContext, NativeOptions};
use image::DynamicImage;

pub fn run(rx: Receiver<BuilderUpdate>) {
    let options = NativeOptions {
        follow_system_theme: true,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Sediment",
        options,
        Box::new(|cc| Box::new(MainWindow::new(cc, rx))),
    );
}

pub struct MainWindow {
    rx: Receiver<BuilderUpdate>,
    preview_texture: Option<egui::TextureHandle>,
    preview_image: ColorImage,
    stats_line: String,
}

impl MainWindow {
    pub fn new(_creation_context: &CreationContext<'_>, rx: Receiver<BuilderUpdate>) -> Self {
        Self {
            rx,
            preview_image: egui::ColorImage::example(),
            preview_texture: None,
            stats_line: String::new(),
        }
    }

    pub fn window_title(&self) -> String {
        "Sediment".to_owned()
    }

    pub fn handle_new_preview(&mut self, new_preview: DynamicImage) {
        let rgba8 = new_preview.to_rgba8();
        let pixels = rgba8.as_flat_samples();
        let egui_img = ColorImage::from_rgba_unmultiplied(
            [new_preview.width() as _, new_preview.height() as _],
            pixels.as_slice(),
        );
        self.preview_image = egui_img;
        self.preview_texture = None; // clear the texture so that update() knows to rebuild it.
    }
}

impl App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint(); // continuous repainting

        // handle any messages that may have come in from the builder
        if let Ok(input) = self.rx.try_recv() {
            match input {
                BuilderUpdate::Preview(new_preview) => self.handle_new_preview(new_preview),
                BuilderUpdate::Stats(s) => self.stats_line = format!("{:?}", s),
            }
        }

        // top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading(&self.window_title());
        });

        // bottom panel
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.y = 10.0;
                ui.label(&self.stats_line);
            });
        });

        // main content
        egui::CentralPanel::default().show(ctx, |ui| {
            let texture: &egui::TextureHandle = self.preview_texture.get_or_insert_with(|| {
                // Load the texture only once.
                ui.ctx().load_texture(
                    "current-image",
                    self.preview_image.clone(),
                    egui::TextureOptions::LINEAR,
                )
            });

            ui.image(texture);
        });
    }
}
