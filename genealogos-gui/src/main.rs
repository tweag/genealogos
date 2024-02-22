#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::mpsc::{Receiver, Sender};

use eframe::egui;
use reqwest::Client;
use serde_json::Value;

enum RequestState {
    Nothing,
    Loading,
    Sbom(String),
}

struct GenealogosApp {
    // State
    flake_ref: String,
    attribute_path: String,
    request_state: RequestState,

    // Sender/Receiver for async notifications.
    request_tx: Sender<RequestState>,
    request_rx: Receiver<RequestState>,
}
impl GenealogosApp {
    fn show_sbom(&self, ui: &mut egui::Ui, sbom: &String) -> eframe::egui::InnerResponse<()> {
        let layout = egui::Layout::top_down(egui::Align::Min);
        ui.allocate_ui_with_layout(ui.available_size(), layout, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                let sbom_label = ui.label("SBOM:");
                ui.code(sbom).labelled_by(sbom_label.id);
            });
        })
    }
}

impl Default for GenealogosApp {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        Self {
            flake_ref: "nixpkgs".to_owned(),
            attribute_path: "hello".to_owned(),
            request_state: RequestState::Nothing,

            request_tx: tx,
            request_rx: rx,
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
        if let Ok(state) = self.request_rx.try_recv() {
            self.request_state = state;
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
                    self.request_tx.clone(),
                    ctx.clone(),
                );
            }
            match &self.request_state {
                RequestState::Nothing => {
                    ui.label("No sbom yet");
                }
                RequestState::Loading => {
                    ui.spinner();
                }
                RequestState::Sbom(sbom) => {
                    self.show_sbom(ui, sbom);
                }
            }
        });
    }
}

fn send_req(
    flake_ref: String,
    attribute_path: String,
    tx: Sender<RequestState>,
    context: egui::Context,
) {
    tokio::spawn(async move {
        let _ = tx.send(RequestState::Loading);

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

        match sbom {
            Some(sbom) => {
                let _ = tx.send(RequestState::Sbom(sbom));
            }
            None => {
                let _ = tx.send(RequestState::Nothing);
            }
        }

        context.request_repaint();
    });
}
