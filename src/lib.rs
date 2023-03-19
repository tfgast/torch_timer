use instant::Instant;
use std::time::Duration;

use eframe::egui::{self, style::Spacing, Style};

fn circle_icon(ui: &mut egui::Ui, openness: f32, response: &egui::Response) {
    let stroke = ui.style().interact(response).fg_stroke;
    let radius = egui::lerp(2.0..=3.0, openness);
    ui.painter()
        .circle_filled(response.rect.center(), radius, stroke.color);
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
#[serde(from = "Duration", into = "Duration")]
enum TimerState {
    RunUntil(Instant),
    Paused(Duration),
}

impl From<Duration> for TimerState {
    fn from(value: Duration) -> Self {
        TimerState::Paused(value)
    }
}

impl From<TimerState> for Duration {
    fn from(val: TimerState) -> Self {
        match val {
            TimerState::RunUntil(end) => end.duration_since(Instant::now()),
            TimerState::Paused(duration) => duration,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct Timer {
    name: String,
    state: TimerState,
    displayed_time: u64,
    local_pause: bool,
    id: u32,
}

const BASE_TIME: u64 = 60;

impl Timer {
    fn new(name: String, duration: u64, id: u32) -> Self {
        let duration = Duration::from_secs(duration * BASE_TIME);
        Self {
            name,
            state: TimerState::Paused(duration),
            displayed_time: 10,
            local_pause: false,
            id,
        }
    }

    fn remove_time(&mut self, removal_time: u64) {
        let d = Duration::from_secs(removal_time * BASE_TIME);
        match &mut self.state {
            TimerState::RunUntil(end) => *end -= d,
            TimerState::Paused(duration) => *duration = duration.saturating_sub(d),
        }
    }

    fn add_time(&mut self, added_time: u64) {
        let d = Duration::from_secs(added_time * BASE_TIME);
        match &mut self.state {
            TimerState::RunUntil(end) => *end += d,
            TimerState::Paused(duration) => *duration = duration.saturating_add(d),
        }
    }

    fn set_time(&mut self, time: u64) {
        let d = Duration::from_secs(time * BASE_TIME);
        match &mut self.state {
            TimerState::RunUntil(end) => *end = Instant::now() + d,
            TimerState::Paused(duration) => *duration = d,
        }
    }

    fn time_remaining(&mut self, now: Instant) -> Duration {
        match self.state {
            TimerState::RunUntil(end) => end.duration_since(now),
            TimerState::Paused(duration) => duration,
        }
    }

    fn is_paused(&self) -> bool {
        matches!(self.state, TimerState::Paused(_))
    }

    fn start(&mut self, now: Instant) {
        if let TimerState::Paused(duration) = self.state {
            self.state = TimerState::RunUntil(now + duration);
        }
    }

    fn pause(&mut self, time_left: Duration) {
        self.state = TimerState::Paused(time_left);
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new(String::new(), 60, 0)
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MyApp {
    timers: Vec<Timer>,
    start_duration: u64,
    displayed_time: u64,
    new_name: String,
    next_timer_id: u32,
    #[serde(skip)]
    running: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            timers: Default::default(),
            start_duration: 60,
            displayed_time: 10,
            new_name: "torch".to_owned(),
            next_timer_id: 0,
            running: false,
        }
    }
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_style(Style {
            spacing: Spacing {
                text_edit_width: 70.0,
                ..Default::default()
            },
            ..Default::default()
        });

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for MyApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let now = Instant::now();
            self.timers.retain_mut(|timer| {
                let time_left = timer.time_remaining(now);
                let mut ret = true;
                if !time_left.is_zero() {
                    ui.horizontal(|ui| {
                        if ui.button("×").clicked() {
                            ret = false;
                        }
                        ui.vertical(|ui| {
                            let id = ui.make_persistent_id(timer.id);
                            let mut state =
                                egui::collapsing_header::CollapsingState::load_with_default_open(
                                    ui.ctx(),
                                    id,
                                    false,
                                );

                            let header_res = ui.horizontal(|ui| {
                                let time = time_left.as_secs();
                                let minutes = time / 60;
                                let seconds = time % 60;
                                ui.text_edit_singleline(&mut timer.name);
                                let text_time = format!("{minutes:0>2}:{seconds:0>2}");
                                if ui.selectable_label(!timer.local_pause, text_time).clicked() {
                                    if timer.local_pause {
                                        if self.running {
                                            timer.start(now);
                                        }
                                        timer.local_pause = false;
                                    } else {
                                        timer.pause(time_left);
                                        timer.local_pause = true;
                                    }
                                }
                                state.show_toggle_button(ui, circle_icon);
                            });

                            state.show_body_indented(&header_res.response, ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.add(egui::DragValue::new(&mut timer.displayed_time));
                                    if ui.button("⏮").clicked() {
                                        timer.add_time(timer.displayed_time);
                                    }
                                    if ui.button("⏭").clicked() {
                                        timer.remove_time(timer.displayed_time);
                                    }
                                    if ui.button("=").clicked() {
                                        timer.set_time(timer.displayed_time);
                                    }
                                });
                            });
                        });
                    });
                } else {
                    ui.horizontal(|ui| {
                        if ui.button("×").clicked() {
                            ret = false;
                        }
                        ui.colored_label(egui::Color32::RED, "Done");
                    });
                }
                ret
            });
            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    let mut timer = Timer::new(
                        self.new_name.clone(),
                        self.start_duration,
                        self.next_timer_id,
                    );
                    self.next_timer_id = self.next_timer_id.wrapping_add(1);
                    if self.running {
                        timer.start(now);
                    }
                    self.timers.push(timer);
                }
                ui.text_edit_singleline(&mut self.new_name);
                ui.add(egui::DragValue::new(&mut self.start_duration));
            });
            ui.separator();
            if !self.timers.is_empty() {
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.displayed_time));
                    if ui.button("⏮").clicked() {
                        for timer in &mut self.timers {
                            if !timer.local_pause {
                                timer.add_time(self.displayed_time);
                            }
                        }
                    }
                    if ui.button("⏭").clicked() {
                        for timer in &mut self.timers {
                            if !timer.local_pause {
                                timer.remove_time(self.displayed_time);
                            }
                        }
                    }
                    if !self.running {
                        if ui.button("⏵").clicked() {
                            for timer in &mut self.timers {
                                if timer.is_paused() && !timer.local_pause {
                                    timer.start(now);
                                }
                            }
                            self.running = true;
                        }
                    } else if ui.button("⏸").clicked() {
                        for timer in &mut self.timers {
                            if !timer.is_paused() {
                                let time_left = timer.time_remaining(now);
                                timer.pause(time_left);
                            }
                        }
                        self.running = false;
                    }
                });
            }
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        });
    }
}
