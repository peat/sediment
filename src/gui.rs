use std::sync::mpsc::Receiver;

use eframe::{egui, epaint::ColorImage, App, CreationContext, NativeOptions};
use image::DynamicImage;

pub fn run(rx: Receiver<MainWindowInput>) {
    let options = NativeOptions {
        follow_system_theme: true,
        ..Default::default()
    };

    eframe::run_native(
        "Sediment",
        options,
        Box::new(|cc| Box::new(MainWindow::new(cc, rx))),
    );
}

// TODO: send stats!
pub enum MainWindowInput {
    Preview(image::DynamicImage),
}

pub struct MainWindow {
    running: bool,
    ready: bool,
    rx: Receiver<MainWindowInput>,
    preview_texture: Option<egui::TextureHandle>,
    preview_image: ColorImage,
}

impl MainWindow {
    pub fn new(_creation_context: &CreationContext<'_>, rx: Receiver<MainWindowInput>) -> Self {
        Self {
            running: false,
            ready: false,
            rx,
            preview_image: egui::ColorImage::example(),
            preview_texture: None,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn run_pause_button_label(&self) -> &str {
        match self.running {
            true => "Pause",
            false => "Run",
        }
    }

    pub fn run_pause_button_clicked(&mut self) {
        self.running = !self.running;
    }

    pub fn open_button_clicked(&mut self) {
        self.ready = true;
        self.running = false;
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
        ctx.request_repaint_after(std::time::Duration::from_millis(250));

        // handle any messages that may have come in from the grinder
        if let Ok(input) = self.rx.try_recv() {
            match input {
                MainWindowInput::Preview(new_preview) => self.handle_new_preview(new_preview),
            }
        }

        // top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading(&self.window_title());
        });

        // bottom panel
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;

                // title and open button
                if ui.button("Open").clicked() {
                    self.open_button_clicked();
                };
                // toggle whether the other buttons are enabled
                ui.set_enabled(self.is_ready());
                if ui.button("Reset").clicked() {
                    println!("Reset: Clicked!");
                };
                if ui.button(self.run_pause_button_label()).clicked() {
                    self.run_pause_button_clicked();
                };
                if ui.button("Save").clicked() {
                    println!("Save: Clicked!");
                };
                ui.label("show stats here");
            });
        });

        // main content
        egui::CentralPanel::default().show(ctx, |ui| {
            let texture: &egui::TextureHandle = self.preview_texture.get_or_insert_with(|| {
                // Load the texture only once.
                ui.ctx().load_texture(
                    "current-image",
                    self.preview_image.clone(),
                    egui::TextureFilter::Linear,
                )
            });

            ui.image(texture, ui.available_size());
        });
    }
}
