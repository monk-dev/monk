use eframe::{egui, epi};
use tracing::info;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::bail;
use crossbeam_channel::{Receiver, Sender};
use monk::types::Item;
use uuid::Uuid;

use crate::message::{Request, RequestMsg, Response, ResponseMsg};

pub struct MonkApp {
    pub items: HashMap<Uuid, Item>,
    cancel: Arc<AtomicBool>,
    rx: Receiver<Response>,
    tx: Sender<Request>,
}

impl MonkApp {
    pub fn init(
        cancel: Arc<AtomicBool>,
        tx: Sender<Request>,
        rx: Receiver<Response>,
    ) -> anyhow::Result<Self> {
        let items = HashMap::new();

        Ok(Self {
            cancel,
            items,
            rx,
            tx,
        })
    }

    pub fn load_items(&mut self) -> anyhow::Result<()> {
        info!("loading items");
        self.tx.send(RequestMsg::ListItem(Default::default()))?;

        match self.rx.recv()?? {
            ResponseMsg::List(items) => {
                self.items
                    .extend(items.into_iter().map(|item| (item.id.clone(), item)));
            }
            _ => {
                bail!("client and server are desynced")
            }
        };

        Ok(())
    }

    pub fn draw_item(&self, id: Uuid, ui: &mut egui::Ui) -> egui::Response {
        let item = &self.items[&id];

        ui.group(|ui| {
            ui.columns(3, |columns| {
                columns[0].monospace(id.to_string());
                columns[1].label(format!("{:?}", item.name));
                columns[2].label(format!("{:?}", item.comment))
            })
        })
        .response
    }
}

impl epi::App for MonkApp {
    fn name(&self) -> &str {
        "eframe template"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        // info!("updating");

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        self.cancel.store(true, Ordering::Relaxed);
                        frame.quit();
                    }
                });
            });

            for key in self.items.keys() {
                self.draw_item(key.clone(), ui);
            }
        });
    }
}
// impl Default for MonkApp {
//     fn default() -> Self {
//         Self {
//             monk: Monk::from_config,
//         }
//     }
// }
