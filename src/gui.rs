use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;

use crate::builder::{Builder, BuilderUpdate, Stats};
use crate::BuildConfig;
use eframe::{egui, epaint::ColorImage, App, CreationContext, NativeOptions};
use image::DynamicImage;

static PREVIEW_TEXTURE_ID: &str = "preview-image";

pub fn run(build_config: BuildConfig) {
    let (builder_tx, builder_update_rx) = channel();

    thread::spawn(move || {
        let mut builder = Builder::new(builder_tx, build_config);
        builder.run();
    });

    let options = NativeOptions {
        follow_system_theme: true,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Sediment",
        options,
        Box::new(|cc| Box::new(SedimentApp::new(cc, builder_update_rx))),
    );
}

pub struct SedimentApp {
    rx: Receiver<BuilderUpdate>,
    preview_texture: Option<egui::TextureHandle>,
    preview_image: ColorImage,
    stats_line: String,
}

impl SedimentApp {
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

    pub fn update_status(&mut self, stats: Stats) {
        self.stats_line = format!(
            "Attempts: {}, Shapes: {}, Current Radius: {}, Elapsed: {:.2}s",
            stats.total_attempts,
            stats.total_successes,
            stats.radius,
            stats.elapsed.as_secs_f32(),
        );
    }

    pub fn fit_image_to_window(
        image_width: f32,
        image_height: f32,
        window_width: f32,
        window_height: f32,
    ) -> (f32, f32) {
        let aspect_ratio = image_width / image_height;
        let mut new_width = window_width;
        let mut new_height = new_width / aspect_ratio;

        if new_height > window_height {
            new_height = window_height;
            new_width = new_height * aspect_ratio;
        }

        (new_width, new_height)
    }
}

impl App for SedimentApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint(); // continuous repainting

        if ctx.input(|i| i.key_released(egui::Key::Escape)) {
            std::process::exit(0);
        }

        // handle any messages that may have come in from the builder
        if let Ok(input) = self.rx.try_recv() {
            match input {
                BuilderUpdate::Preview(new_preview) => self.handle_new_preview(new_preview),
                BuilderUpdate::Stats(s) => self.update_status(s),
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
                ui.ctx().load_texture(
                    PREVIEW_TEXTURE_ID,
                    self.preview_image.clone(),
                    Default::default(),
                )
            });

            // scale the image to fit the available space
            let ui_size = ui.available_size();
            let image_size = texture.size_vec2();

            let (new_width, new_height) =
                Self::fit_image_to_window(image_size.x, image_size.y, ui_size.x, ui_size.y);

            let new_dimensions = egui::Vec2::new(new_width, new_height);

            ui.image((texture.id(), new_dimensions));
        });
    }
}
