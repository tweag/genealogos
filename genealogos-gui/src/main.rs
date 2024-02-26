#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::mpsc::{Receiver, Sender};

use eframe::egui;
use egui_graphs::{DefaultEdgeShape, DefaultNodeShape, Graph, GraphView, SettingsStyle};
use genealogos::cyclonedx::CycloneDX;
use petgraph::{
    graph,
    stable_graph::{NodeIndex, StableGraph},
};
use reqwest::Client;
use serde_json::Value;

enum RequestState {
    Nothing,
    Loading,
    Done(String, Graph<(), ()>),
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
                    ui.label("No SBOM generated yet");
                }
                RequestState::Loading => {
                    ui.spinner();
                }
                RequestState::Done(sbom, graph) => {
                    let mut cloned_graph = graph.clone();
                    let style_settings = &SettingsStyle::new().with_labels_always(true);
                    ui.add(
                        &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(
                            &mut cloned_graph,
                        )
                        .with_styles(style_settings),
                    );
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
                let graph = generate_graph(&sbom);
                let _ = tx.send(RequestState::Done(sbom, graph));
            }
            None => {
                let _ = tx.send(RequestState::Nothing);
            }
        }

        context.request_repaint();
    });
}

fn generate_graph(sbom: &str) -> Graph<(), ()> {
    println!("{}", sbom);
    let cyclonedx: CycloneDX = serde_json::from_str(sbom).unwrap();

    match cyclonedx {
        CycloneDX::V1_4(_) => todo!(),
        CycloneDX::V1_5(cyclonedx) => generate_graph_v1_5(cyclonedx),
    }
}

fn generate_graph_v1_5(cyclonedx: serde_cyclonedx::cyclonedx::v_1_5::CycloneDx) -> Graph<(), ()> {
    // // Create a new graph
    // let mut graph = StableGraph::new();

    // // Maps the components' bom-ref to the node index
    // // This will be used when creating the dependency edges
    // let mut ref_to_idx: std::collections::HashMap<String, graph::NodeIndex> =
    //     std::collections::HashMap::new();

    // // Map the idx to the entire component
    // // This will be used when setting the label of the nodes and providing more information
    // let mut idx_to_component: std::collections::HashMap<
    //     graph::NodeIndex,
    //     serde_cyclonedx::cyclonedx::v_1_5::Component,
    // > = std::collections::HashMap::new();

    // // Create all the nodes
    // if let Some(components) = cyclonedx.components {
    //     for component in components {
    //         let idx = graph.add_node(());
    //         // Store in the maps
    //         ref_to_idx.insert(component.bom_ref.clone().unwrap(), idx);
    //         idx_to_component.insert(idx, component);
    //     }
    // }

    // eprintln!("ref_to_idx: {:?}", ref_to_idx);

    // // Create all the edges
    // if let Some(dependencies) = cyclonedx.dependencies {
    //     for dependency in dependencies {
    //         let from = match &dependency.ref_ {
    //             Value::String(ref_) => {
    //                 if let Some(idx) = ref_to_idx.get(ref_) {
    //                     *idx
    //                 } else {
    //                     continue;
    //                 }
    //             }
    //             _ => continue,
    //         };

    //         if let Some(depends_on) = dependency.depends_on {
    //             for dep in depends_on {
    //                 match dep {
    //                     Value::String(dep) => {
    //                         let to = ref_to_idx.get(&dep).unwrap();

    //                         eprintln!("from: {:?}, to: {:?}", from, to);
    //                         graph.add_edge(from, *to, ());
    //                     }
    //                     _ => continue,
    //                 }
    //             }
    //         }
    //     }
    // }

    // let mut graph = Graph::from(&graph);

    // // Set the label of the nodes
    // for (idx, node) in graph.clone().nodes_iter() {
    //     graph
    //         .node_mut(idx)
    //         .unwrap()
    //         .set_label(idx_to_component.get(&idx).unwrap().name.clone());
    // }

    // graph

    let mut g = StableGraph::new();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    Graph::from(&g)
}
