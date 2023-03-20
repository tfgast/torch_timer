#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).

    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(180.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "TorchTimer",
        options,
        Box::new(|cc| Box::new(torch_timer::MyApp::new(cc))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(torch_timer::MyApp::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
