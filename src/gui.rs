use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::builder::{Builder, BuilderCommand, BuilderUpdate, Stats};
use crate::{BuildConfig, Canvas};
use eframe::{egui, epaint::ColorImage, App, CreationContext, NativeOptions};
use image::DynamicImage;

static PREVIEW_TEXTURE_ID: &str = "preview-image";
static REFERENCE_TEXTURE_ID: &str = "reference-image";

pub fn run(build_config: BuildConfig) {
    let (builder_update_tx, builder_update_rx) = channel();
    let (builder_command_tx, builder_command_rx) = channel();

    let builder_config = build_config.clone();
    let gui_config = build_config.clone();

    thread::spawn(move || {
        let mut builder = Builder::new(builder_update_tx, builder_config);
        builder.interactive(builder_command_rx);
    });

    let options = NativeOptions {
        follow_system_theme: true,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Sediment",
        options,
        Box::new(|cc| {
            Box::new(SedimentApp::new(
                cc,
                gui_config,
                builder_update_rx,
                builder_command_tx,
            ))
        }),
    );
}

pub struct SedimentApp {
    builder_update_rx: Receiver<BuilderUpdate>,
    builder_command_tx: Sender<BuilderCommand>,
    reference_texture: Option<egui::TextureHandle>,
    preview_texture: Option<egui::TextureHandle>,
    preview_image: ColorImage,
    stats_line: String,
    config: BuildConfig,
}

impl SedimentApp {
    pub fn new(
        _creation_context: &CreationContext<'_>,
        config: BuildConfig,
        builder_update_rx: Receiver<BuilderUpdate>,
        builder_command_tx: Sender<BuilderCommand>,
    ) -> Self {
        Self {
            builder_update_rx,
            builder_command_tx,
            reference_texture: None,
            preview_image: egui::ColorImage::example(),
            preview_texture: None,
            stats_line: String::new(),
            config,
        }
    }

    fn dynamic_image_to_color_image(dynamic_image: DynamicImage) -> ColorImage {
        let rgba8 = dynamic_image.to_rgba8();
        let pixels = rgba8.as_flat_samples();
        ColorImage::from_rgba_unmultiplied(
            [dynamic_image.width() as _, dynamic_image.height() as _],
            pixels.as_slice(),
        )
    }

    fn update_preview(&mut self, new_preview: DynamicImage) {
        self.preview_image = Self::dynamic_image_to_color_image(new_preview);
        self.preview_texture = None; // clear the texture so that update() knows to rebuild it.
    }

    fn update_status(&mut self, stats: Stats) {
        self.stats_line = format!(
            "Attempts: {}, Shapes: {}, Current Radius: {}, Elapsed: {:.2}s",
            stats.total_attempts,
            stats.total_successes,
            stats.radius,
            stats.elapsed.as_secs_f32(),
        );
    }

    fn maximize_dimensions(original: egui::Vec2, available: egui::Vec2) -> egui::Vec2 {
        let aspect_ratio = original.x / original.y;
        let mut new_width = available.x;
        let mut new_height = new_width / aspect_ratio;

        if new_height > available.y {
            new_height = available.y;
            new_width = new_height * aspect_ratio;
        }

        egui::Vec2::new(new_width, new_height)
    }

    fn scale_texture_to_fit(
        ui: &egui::Ui,
        texture: &egui::TextureHandle,
    ) -> (egui::TextureId, egui::Vec2) {
        let new_dimensions = Self::maximize_dimensions(texture.size_vec2(), ui.available_size());

        (texture.id(), new_dimensions)
    }

    fn preview_texture_handle(&mut self, ui: &mut egui::Ui) -> &egui::TextureHandle {
        self.preview_texture.get_or_insert_with(|| {
            ui.ctx().load_texture(
                PREVIEW_TEXTURE_ID,
                self.preview_image.clone(),
                Default::default(),
            )
        })
    }

    fn reference_texture_handle(&mut self, ui: &mut egui::Ui) -> &egui::TextureHandle {
        self.reference_texture.get_or_insert_with(|| {
            let reference_image = Canvas::open(&self.config.input).unwrap();
            let reference_texture = Self::dynamic_image_to_color_image(reference_image.img);
            ui.ctx()
                .load_texture(REFERENCE_TEXTURE_ID, reference_texture, Default::default())
        })
    }
}

impl App for SedimentApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint(); // continuous repainting

        if ctx.input(|i| i.key_released(egui::Key::Escape)) {
            self.builder_command_tx.send(BuilderCommand::Quit).unwrap();
            std::process::exit(0);
        }

        if ctx.input(|i| i.key_released(egui::Key::R)) {
            self.builder_command_tx.send(BuilderCommand::Start).unwrap();
        }

        // handle any messages that may have come in from the builder
        if let Ok(input) = self.builder_update_rx.try_recv() {
            match input {
                BuilderUpdate::Preview(new_preview) => self.update_preview(new_preview),
                BuilderUpdate::Stats(s) => self.update_status(s),
            }
        }

        // window is split into four panels:
        // - left panel is the reference image
        // - middle panel is the generated image
        // - bottom panel is the status line

        let image_margin_px = 10.0;
        let image_margin = egui::Margin::same(image_margin_px);
        let panel_frame = egui::Frame::none().outer_margin(image_margin);

        // calculate panel widths (50% of the window)
        let window_dimensions = frame.info().window_info.size;
        let panel_width = (window_dimensions.x / 2.0) - (image_margin_px * 2.0);

        // bottom panel status line
        egui::TopBottomPanel::bottom("stats_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.y = 10.0;
                ui.label(&self.stats_line);
            });
        });

        egui::SidePanel::left("reference_panel")
            .exact_width(panel_width)
            .resizable(false)
            .show_separator_line(true)
            .frame(panel_frame)
            .show(ctx, |ui| {
                let texture = self.reference_texture_handle(ui);
                ui.image(Self::scale_texture_to_fit(ui, texture));
            });

        // generated image preview
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                let texture = self.preview_texture_handle(ui);
                ui.image(Self::scale_texture_to_fit(ui, texture));
            });
    }
}
