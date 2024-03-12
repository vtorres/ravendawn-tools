use crate::ravendawn::constants::{
    PLAYER_CURRENT_LEVEL_EXP_OFFSET, PLAYER_NEXT_LEVEL_EXP_OFFSET, PLAYER_OFFSET,
};
#[allow(dead_code)]
#[allow(non_snake_case)]
use crate::{external, WINDOW_HEIGHT, WINDOW_WIDTH};
use eframe::NativeOptions;
use eframe::{egui, egui::containers::ScrollArea};
use std::collections::HashMap;
use sysinfo::System;
use winapi::um::winnt::{HANDLE, PROCESS_ALL_ACCESS};

#[derive(Debug, std::cmp::PartialEq)]
enum SelectGui {
    MemoryHacks,
}

struct SeparatedThreadState {
    ctx: Option<egui::Context>,
}

impl SeparatedThreadState {
    pub fn new() -> Self {
        Self { ctx: None }
    }
}

// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct CustomWindow {
    // Selected GUI
    #[cfg_attr(feature = "persistence", serde(skip))]
    selected_gui: SelectGui,

    // Keep updating without focus
    #[cfg_attr(feature = "persistence", serde(skip))]
    state: std::sync::Arc<std::sync::Mutex<SeparatedThreadState>>,

    // Processes
    #[cfg_attr(feature = "persistence", serde(skip))]
    pid: u32,
    #[cfg_attr(feature = "persistence", serde(skip))]
    process_filter: String,
    #[cfg_attr(feature = "persistence", serde(skip))]
    system: System,
    #[cfg_attr(feature = "persistence", serde(skip))]
    processes: HashMap<u32, String>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    process_name: String,
    #[cfg_attr(feature = "persistence", serde(skip))]
    handle: HANDLE,
    #[cfg_attr(feature = "persistence", serde(skip))]
    process_base: usize,

    // Exp Tracker
    #[cfg_attr(feature = "persistence", serde(skip))]
    first_exp_recorded: f64,
    #[cfg_attr(feature = "persistence", serde(skip))]
    first_exp_recorded_time: std::time::Instant,
    #[cfg_attr(feature = "persistence", serde(skip))]
    exp_made_last_fifteen_minutes: u32,

    // Window state
    window_position: egui::Pos2,
}

impl Default for CustomWindow {
    fn default() -> Self {
        // Thread sync state
        let state = std::sync::Arc::new(std::sync::Mutex::new(SeparatedThreadState::new()));

        let mut data: CustomWindow = Self {
            // Selected GUI
            selected_gui: SelectGui::MemoryHacks,

            // Thread state
            state: state,

            // Processes
            pid: 0,
            process_filter: String::from(""),
            system: System::new_all(),
            processes: HashMap::new(),
            process_name: "".into(),
            handle: 0 as HANDLE,
            process_base: 0,

            // Exp Tracker
            first_exp_recorded: 0.0,
            first_exp_recorded_time: std::time::Instant::now(),
            exp_made_last_fifteen_minutes: 0,

            // Window state
            window_position: egui::Pos2 { x: 0f32, y: 0f32 },
        };

        data.system.refresh_all();

        for (pid, process) in data.system.processes() {
            data.processes
                .insert(pid.as_u32(), process.name().to_string());
        }

        data
    }
}

pub fn draw_window() {
    let app_icon: eframe::egui::IconData =
        eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png")).unwrap();

    let options: NativeOptions = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([*WINDOW_WIDTH, *WINDOW_HEIGHT])
            .with_resizable(true)
            .with_fullscreen(false)
            .with_maximize_button(false)
            .with_icon(app_icon)
            .with_decorations(true)
            .with_always_on_top()
            .with_transparent(true),
        ..Default::default()
    };

    let _ = eframe::run_native(
        obfstr::obfstr!(r"Ravendawn Exp Tracker"),
        options,
        Box::new(|ctx: &eframe::CreationContext| {
            // Setting Style
            let mut style = egui::Style::default();
            style.visuals.dark_mode = true;
            style.visuals.window_rounding = egui::Rounding::ZERO;
            style.visuals.window_shadow = eframe::epaint::Shadow {
                extrusion: 0.0,
                color: egui::Color32::BLACK,
            };
            ctx.egui_ctx.set_style(style);

            let mut default_window = CustomWindow::default();

            #[cfg(feature = "persistence")]
            if let Some(storage) = ctx.storage {
                if let Some(state) = eframe::get_value(storage, eframe::APP_KEY) {
                    default_window = state;
                }
            }

             // Cloning state of the default CustomWindow
             default_window.state.lock().unwrap().ctx = Some(ctx.egui_ctx.clone());
             let state_clone = default_window.state.clone();
             std::thread::spawn(move || {
                 different_thread_process(state_clone);
             });

            // This gives us image support:
            egui_extras::install_image_loaders(&ctx.egui_ctx);

            Box::new(default_window)
        }),
    );
}

impl CustomWindow {
    // Header
    //
    pub fn header(&mut self, ctx: &egui::Context) {
        // Shows the header only after selecting a process
        if self.pid != 0 {
            egui::TopBottomPanel::top("top").show(ctx, |ui: &mut egui::Ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.selected_gui,
                        SelectGui::MemoryHacks,
                        "Exp Tracker",
                    );
                });
            });
        }
    }

    // Body
    //
    pub fn body(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            if self.selected_gui == SelectGui::MemoryHacks {
                if self.pid != 0 {
                    self.exp_tracker_screen(ctx);
                } else {
                    self.processes_screen(ui);
                }
            }
        });
    }

    // Footer
    //
    pub fn footer(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui: &mut egui::Ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                ui.small(obfstr::obfstr!(r"v1.0.0"));
            })
        });
    }

    /// Show the content for Process selection
    ///
    pub fn processes_screen(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui: &mut egui::Ui| {
            if ui.button(obfstr::obfstr!(r"Refresh")).clicked() {
                self.system.refresh_all();
                self.process_filter = String::from(obfstr::obfstr!(r"Process name"));
                self.processes = HashMap::new();
                for (pid, process) in self.system.processes() {
                    self.processes
                        .insert(pid.as_u32(), process.name().to_string());
                }
            }

            if ui.button(obfstr::obfstr!(r"Filter")).clicked() {
                self.system.refresh_all();

                self.processes = HashMap::new();
                for (pid, process) in self.system.processes() {
                    if process
                        .name()
                        .to_lowercase()
                        .contains(&self.process_filter.to_lowercase())
                    {
                        self.processes
                            .insert(pid.as_u32(), process.name().to_string());
                    }
                }
            }

            ui.text_edit_singleline(&mut self.process_filter);
        });

        ui.add_space(4.0);

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show_viewport(ui, |ui: &mut eframe::egui::Ui, _| {
                let font_id = egui::TextStyle::Body.resolve(ui.style());
                let row_height = ui.fonts(|f| f.row_height(&font_id)) + ui.spacing().item_spacing.y;

                ui.set_height((self.processes.len() as f32) * (row_height * 1.5));

                for (pid, process) in &self.processes {
                    ui.horizontal(|ui| {
                        ui.label(pid.to_string());

                        if ui.link(process).clicked() {
                            self.pid = *pid;
                            self.process_name = process.to_string();
                        }
                    });
                }
            });
    }

    /// Show the content for EXP Tracker
    ///
    pub fn exp_tracker_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            ui.add_space(25.0);

            if self.handle == std::ptr::null_mut() {
                self.handle = unsafe {
                    external::process::open_process(PROCESS_ALL_ACCESS, self.pid).unwrap()
                };
            }

            if self.first_exp_recorded == 0.0 {
                let current_exp = external::readmem::resolve_multi_level_pointer(
                    self.handle,
                    self.process_base + PLAYER_OFFSET,
                    vec![PLAYER_CURRENT_LEVEL_EXP_OFFSET],
                );

                let player_current_level_xp_value =
                    external::readmem::read_f64(self.handle, current_exp);

                if player_current_level_xp_value > 0.0 {
                    self.first_exp_recorded = player_current_level_xp_value;
                    self.first_exp_recorded_time = std::time::Instant::now();
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            }

            self.process_base =
                external::process::get_module_base(self.pid, &self.process_name).unwrap();

            let current_exp_offsetp = external::readmem::resolve_multi_level_pointer(
                self.handle,
                self.process_base + PLAYER_OFFSET,
                vec![PLAYER_CURRENT_LEVEL_EXP_OFFSET],
            );
            let next_exp_offset = external::readmem::resolve_multi_level_pointer(
                self.handle,
                self.process_base + PLAYER_OFFSET,
                vec![PLAYER_NEXT_LEVEL_EXP_OFFSET],
            );

            
            let current_xp: f64 = external::readmem::read_f64(self.handle, current_exp_offsetp);
            let xp_to_next_level = external::readmem::read_f64(self.handle, next_exp_offset);
            let current_xp_time = std::time::Instant::now();

            if current_xp == 0.0 || current_xp == -1.0
            {
                ui.vertical_centered(|ui| {
                    ui.label("Informations");
                    ui.add_space(5.0);

                    ui.label("Please, log in to your character.");
                });
            } else {
                // let percentage_from_level = current_xp as f32 / xp_to_next_level as f32 * 100.0;
                // let percentage_to_next_level = 100.0 - percentage_from_level;
                let missing_xp = xp_to_next_level - current_xp;
                let elapsed = current_xp_time - self.first_exp_recorded_time;
                let xp_made = current_xp - self.first_exp_recorded;

                fn convert_total_to_hour_and_seconds(total_time_in_seconds: u32) -> String {
                    let hours = total_time_in_seconds / 3600;
                    let minutes = (total_time_in_seconds % 3600) / 60;

                    return format!("{hours} hours and {minutes} minutes");
                }

                ui.vertical_centered(|ui| {
                    let missing_label =
                        format!("{} experience remaining for next level", missing_xp);

                    ui.add_space(5.0);
                    ui.label(missing_label);

                    ui.add_space(5.0);

                    if xp_made < 1.0 {
                        ui.label("You haven't gained exp yet.");
                    } else {
                        if elapsed.as_secs() < 60 {
                            ui.label("Collecting information! ( it takes about one minute )");
                        } else {
                            let xp_in_one_hour = xp_made as u32 * 3600 / elapsed.as_secs() as u32;
                            let xp_in_one_hour_label = format!("{} xp per hour", xp_in_one_hour);

                            ui.label(xp_in_one_hour_label);
                            let total_time = (xp_to_next_level - current_xp) as u32
                                * elapsed.as_secs() as u32
                                / xp_made as u32;

                            let total_time_formatted =
                                convert_total_to_hour_and_seconds(total_time);
                            let time_to_next_level = format!(
                                "You will reach the next level in {}",
                                total_time_formatted
                            );

                            ui.label(time_to_next_level);

                            ui.add_space(5.0);

                            if ui.button(obfstr::obfstr!(r"Reset Tracker")).clicked() {
                                let current_exp = external::readmem::resolve_multi_level_pointer(
                                    self.handle,
                                    self.process_base + PLAYER_OFFSET,
                                    vec![PLAYER_CURRENT_LEVEL_EXP_OFFSET],
                                );
                                let player_current_level_xp_value =
                                    external::readmem::read_f64(self.handle, current_exp);
                                self.first_exp_recorded = player_current_level_xp_value;
                                self.first_exp_recorded_time = std::time::Instant::now();
                                std::thread::sleep(std::time::Duration::from_millis(1000));
                            }

                            ui.add_space(5.0);
                        }
                    }

                    ui.add_space(10.0);
                });
            }
        });
    }
}

fn different_thread_process(state_clone: std::sync::Arc<std::sync::Mutex<SeparatedThreadState>>) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let ctx = &state_clone.lock().unwrap().ctx;

        match ctx {
            Some(x) => x.request_repaint(),
            None => panic!("error in Option<>"),
        }
    }
}

impl eframe::App for CustomWindow {
    // Make sure we don't paint anything behind the rounded corners
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    // Called by the framework to save state before shutdown.
    /// Note that you must enable the persistence feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.header(ctx);
        self.body(ctx);
        self.footer(ctx);
    }
}
