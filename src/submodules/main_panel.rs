use eframe::egui::{self, CentralPanel};

use crate::Flasher;

impl Flasher {
    pub fn panel_pre_send(&mut self, ctx: &egui::Context) {
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
                            for (model_number, device_name, product_id) in self.data.devices.clone()
                            {
                                ui.selectable_value(
                                    &mut self.data.device,
                                    (model_number, device_name.clone(), product_id),
                                    device_name,
                                );
                            }
                        });
                });

                if ui.button("Print device list to console.").clicked() {
                    println!("{:?}", self.data.devices);
                    println!("{:?}", self.data.device)
                }
                if ui.button("Print connected devices to console.").clicked() {
                    Flasher::transmit_payload(self);
                }
                if ui.button("Install NickelMenu.").clicked() {
                    Flasher::install_nickelmenu(self);
                }
            });
            if ui.button("send message to thread.").clicked() {
                self.data.tx.as_mut().unwrap().send(true).unwrap();
            }
        });
    }
    pub fn panel_post_send(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| ui.label("Watching for device."));
    }
}
