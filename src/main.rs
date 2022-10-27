use eframe::{
    egui::{self, CentralPanel},
    App, CreationContext, NativeOptions,
};

fn main() {
    let mut options = NativeOptions::default();
    options.follow_system_theme = true;
    eframe::run_native(
        "Sediment",
        options,
        Box::new(|cc| Box::new(MainWindow::new(cc))),
    );
}

struct MainWindow {
    running: bool,
    ready: bool,
    src_file_name: Option<String>,
}

impl MainWindow {
    pub fn new(_creation_context: &CreationContext<'_>) -> Self {
        Self {
            running: false,
            ready: false,
            src_file_name: None,
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
        self.src_file_name = Some("example.jpg".to_owned());
    }

    pub fn window_title(&self) -> String {
        match &self.src_file_name {
            Some(sfn) => format!("Sediment: {}", sfn),
            None => format!("Sediment"),
        }
    }
}

impl App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |_ui| {
            // top menu
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
        });
    }
}
