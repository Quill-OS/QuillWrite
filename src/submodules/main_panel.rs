use eframe::egui::{self, CentralPanel};

use crate::Flasher;

impl Flasher {
    pub fn render_main_panel(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.visuals_mut().widgets.active.bg_stroke = eframe::epaint::Stroke {
                    width: 0.,
                    color: egui::Color32::RED,
                };
                ui.visuals_mut().widgets.inactive.bg_stroke = eframe::epaint::Stroke {
                    width: 0.,
                    color: egui::Color32::RED,
                };

                ui.vertical(|ui| {
                    ui.label("Select your E-Reader");
                    egui::ComboBox::from_label(" ")
                        .selected_text(&self.data.device.1)
                        .show_ui(ui, |ui| {
                            for (model_number, device_name) in self.data.devices.clone() {
                                ui.selectable_value(
                                    &mut self.data.device,
                                    (model_number, device_name.clone()),
                                    device_name,
                                );
                            }
                        });
                });

                if ui.button("Print device list to console.").clicked() {
                    println!("{:?}", self.data.devices)
                }
            });
        });
    }
}
