use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Egui Button App",
        eframe::NativeOptions {
            drag_and_drop_support: false,
            initial_window_size: Some(egui::vec2(800.0, 600.0)),
            ..Default::default()
        },
        Box::new(|_| Box::new(App))
    )
}

struct App;

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("hi mom");

                ui.group(|ui| {
                    ui.spacing_mut().item_spacing.y = 20.0;
                    if ui.button("Button 1").clicked() {
                        println!("Button 1 was pressed!");
                    }
                    if ui.button("Button 2").clicked() {
                        println!("Button 2 was pressed!");
                    }
                });
            });
        });
    }
}
