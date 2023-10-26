use std::{
    sync::{Arc, Mutex},
    thread,
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    ip_address: String,
    initial_altitude: f32,
    direction: f32,
    apoapsis_altitude: f32,
    circularize_in_apoapsis: bool,
    hover_altitude: f32,
    #[serde(skip)]
    abort_button_text: String,
    #[serde(skip)]
    abort: Arc<Mutex<bool>>,
    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            ip_address: "192.168.0.56".to_owned(),
            initial_altitude: 200.0,
            direction: 90.0,
            apoapsis_altitude: 80000.0,
            circularize_in_apoapsis: true,
            value: 2.7,
            abort: Arc::new(Mutex::new(false)),
            abort_button_text: "Abort".to_owned(),
            hover_altitude: 100.0,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            _frame.close();
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Connection address:");

            ui.horizontal(|ui| {
                ui.label("IP address to connect to KRPC:");
                ui.text_edit_singleline(&mut self.ip_address);
            });

            ui.separator();

            ui.heading("Lift off parameters:");

            ui.horizontal(|ui| {
                ui.label("Initial altitude:");
                ui.add(egui::Slider::new(&mut self.initial_altitude, 0.0..=1000.0).suffix("m"));
            });
            ui.horizontal(|ui| {
                ui.label("Direction:");
                ui.add(egui::Slider::new(&mut self.direction, 0.0..=360.0).suffix("°"));
            });
            ui.horizontal(|ui| {
                ui.label("Apoapsis altitude:");
                ui.add(egui::Slider::new(&mut self.apoapsis_altitude, 0.0..=200000.0).suffix("m"));
            });
            ui.horizontal(|ui| {
                ui.label("Circularize in apoapsis:");
                ui.checkbox(&mut self.circularize_in_apoapsis, "Circularize");
                if ui.button("Launch").clicked() {
                    println!("Launch!");
                    println!("lauch params: init altitude: {}", self.initial_altitude);
                    println!("direction: {}", self.direction);
                    println!("apoapsis altitude: {}", self.apoapsis_altitude);
                    println!("Circularize?: {}", self.circularize_in_apoapsis);
                    let ip_address = self.ip_address.clone();
                    let initial_altitude = self.initial_altitude as f64;
                    let direction = self.direction as f32;
                    let apoapsis_altitude = self.apoapsis_altitude as f64;
                    let circularize_in_apoapsis = self.circularize_in_apoapsis;
                    let abort = self.abort.clone();
                    thread::spawn(move || {
                        let conn = ksp_lib::connection::Connection::builder()
                            .ip_addr(ip_address)
                            .build()
                            .unwrap();
                        let lift_off = ksp_lib::lift_off::LiftOff::builder()
                            .connect(conn.clone())
                            .t_minus(5)
                            .stopper(abort.clone())
                            .build();
                        lift_off.start();
                        let grav_curve =
                            ksp_lib::gravitational_curve::GravitationalCurve::builder()
                                .connect(conn.clone())
                                .grav_curve_initial_altitude(initial_altitude)
                                .direction(direction)
                                .final_apoastro(apoapsis_altitude)
                                .stopper(abort.clone())
                                .build();
                        grav_curve.start();
                        if circularize_in_apoapsis {
                            let maneuver = ksp_lib::maneuver::Maneuver::builder()
                                .connect(conn)
                                .circularize_in(ksp_lib::maneuver::Apsis::Apoapsis)
                                .stopper(abort.clone())
                                .build();
                            maneuver.execute();
                        }
                    });
                }
                if ui.button("Abort").clicked() {
                    println!("Abort!");
                }
            });

            ui.separator();
            ui.heading("Maneuver menu:");
            ui.horizontal(|ui| {
                if ui.button("Circularize in apoapsis").clicked() {
                    println!("Circularize in apoapsis!");
                }
                if ui.button("Circularize in periapsis").clicked() {
                    println!("Circularize in periapsis!");
                }
                if ui.button("Execute next maneuver").clicked() {
                    println!("Execute next maneuver!");
                    let ip_address = self.ip_address.clone();
                    let abort = self.abort.clone();
                    thread::spawn(move || {
                        let maneuver = ksp_lib::maneuver::Maneuver::builder()
                            .connect(
                                ksp_lib::connection::Connection::builder()
                                    .ip_addr(ip_address)
                                    .build()
                                    .unwrap(),
                            )
                            .stopper(abort.clone())
                            .get_maneuver_by_index(0)
                            .build();
                        maneuver.execute();
                    });
                }
            });

            ui.separator();
            ui.heading("Landing and hovering menu:");
            ui.horizontal(|ui| {
                ui.label("Hover altitude:");
                ui.add(egui::DragValue::new(&mut self.hover_altitude).speed(1));
                if ui.button("Hover").clicked() {
                    println!("Hover");
                }
                if ui.button("Land").clicked() {
                    println!("Land!");
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
                if ui.button(self.abort_button_text.clone()).clicked() {
                    let mut abort = self.abort.lock().unwrap();
                    if *abort == false {
                        *abort = true;
                        self.abort_button_text = "Restore".to_owned();
                    } else {
                        *abort = false;
                        self.abort_button_text = "Abort".to_owned();
                    }
                }
            });
        });
    }
}
