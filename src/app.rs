use crate::{
    systemd::{self, UnitData},
    widgets,
};

use poll_promise::Promise;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,
    #[serde(skip)]
    units: Vec<UnitData>,

    #[serde(skip)]
    user_units: Vec<UnitData>,

    #[serde(skip)]
    tab: String,

    #[serde(skip)]
    properties: widgets::PropertiesWindow,

    #[serde(skip)]
    system_units_promise: Promise<zbus::Result<Vec<UnitData>>>
    
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            units: Vec::default(),
            user_units: Vec::default(),
            tab: "system".to_string(),
            properties: widgets::PropertiesWindow::default(),
            system_units_promise: Promise::spawn_async(systemd::list_system_units())
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        units: Vec<UnitData>,
        user_units: Vec<UnitData>,
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let mut app: Self;
        if let Some(storage) = cc.storage {
            app = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        } else {
            app = Default::default();
        }

        app.units = units;
        app.user_units = user_units;

        app
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            label,
            value,
            units,
            user_units,
            tab,
            ..
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            egui::warn_if_debug_build(ui);

            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));

            let system_tab = "system".to_string();
            let user_tab = "user".to_string();

            let sys_services_radio = ui.radio_value(tab, system_tab, "System Services");
            let user_serices_radio = ui.radio_value(tab, user_tab, "User Services");

            if sys_services_radio.clicked() || user_serices_radio.clicked() {
                self.properties.close();
            }

            let mut unit_index: Option<usize> = None;

            match tab.as_str() {
                "system" => {
                    ui.heading(format!("services: {}", units.len()));
                    widgets::units_table(units, ui, &mut unit_index);

                    if let Some(index) = unit_index {
                        self.properties.open(index);
                    }

                    self.properties.draw(ui.ctx(), &|i| units.get(i));
                }

                "user" => {
                    ui.heading(format!("user services: {}", user_units.len()));
                    widgets::units_table(user_units, ui, &mut unit_index);

                    if let Some(index) = unit_index {
                        self.properties.open(index);
                    }

                    self.properties.draw(ui.ctx(), &|i| user_units.get(i));
                }
                _ => todo!(),
            }
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }

}
