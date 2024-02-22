#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::mpsc::{Receiver, Sender};

use eframe::egui;
use reqwest::Client;
use serde_json::Value;

struct GenealogosApp {
    // State
    flake_ref: String,
    attribute_path: String,
    sbom: Option<String>,

    // Sender/Receiver for async notifications.
    tx: Sender<Option<String>>,
    rx: Receiver<Option<String>>,
}

impl Default for GenealogosApp {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        Self {
            flake_ref: "nixpkgs".to_owned(),
            attribute_path: "hello".to_owned(),
            sbom: None,

            tx,
            rx,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), impl std::error::Error> {
    env_logger::init();
    eframe::run_native(
        "Genealogos",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::<GenealogosApp>::default()),
    )
}

impl eframe::App for GenealogosApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(Some(sbom)) = self.rx.try_recv() {
            self.sbom = Some(sbom);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Genelogos");
            ui.horizontal(|ui| {
                let flake_ref_label = ui.label("Flake Ref: ");
                ui.text_edit_singleline(&mut self.flake_ref)
                    .labelled_by(flake_ref_label.id);
                1
            });
            ui.horizontal(|ui| {
                let attribute_path_label = ui.label("Attribute Path: ");
                ui.text_edit_singleline(&mut self.attribute_path)
                    .labelled_by(attribute_path_label.id);
            });
            if ui.button("Generate SBOM").clicked() {
                send_req(
                    self.flake_ref.clone(),
                    self.attribute_path.clone(),
                    self.tx.clone(),
                    ctx.clone(),
                );
            }
            let layout = egui::Layout::top_down(egui::Align::Min);
            ui.allocate_ui_with_layout(ui.available_size(), layout, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    let sbom_label = ui.label("SBOM:");
                    ui.code(self.sbom.as_deref().unwrap_or("No sbomb yet"))
                        .labelled_by(sbom_label.id);
                });
            })
        });
    }
}

fn send_req(
    flake_ref: String,
    attribute_path: String,
    tx: Sender<Option<String>>,
    context: egui::Context,
) {
    tokio::spawn(async move {
        let json: Value = Client::default()
            .get(format!(
                "http://127.0.0.1:8000/api/analyze?flake_ref={}&attribute_path={}",
                flake_ref, attribute_path
            ))
            .send()
            .await
            .expect("Unable to send request")
            .json()
            .await
            .expect("Unable to parse response");

        let sbom = json
            .get("sbom")
            .map(|sbom| serde_json::to_string_pretty(sbom).unwrap());

        // After parsing the response, notify the GUI thread of the increment value.
        let _ = tx.send(sbom);
        context.request_repaint();
    });
}
