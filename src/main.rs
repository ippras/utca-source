//! `trunk serve --address=0.0.0.0`
//! - nix:  
//! `RUST_LOG=none,utca=trace cargo run`
//! - win:  
//! `$env:RUST_LOG="none,utca=trace"` `cargo run`
//!
//! `rustup target add wasm32-unknown-unknown`
//! `trunk build --release --public-url utca`
//!
//! [Determination of the Positional-Species Composition of Plant Reserve Triacylglycerols by Partial Chemical Deacylation](https://sci-hub.ru/10.1023/A:1016732708350)

// #![feature(anonymous_lifetime_in_impl_trait)]
// #![feature(associated_type_defaults)]
// #![feature(decl_macro)]
// #![feature(float_next_up_down)]
// #![feature(hash_extract_if)]
// #![feature(impl_trait_in_assoc_type)]
// #![feature(lazy_get)]
// #![feature(slice_split_once)]
// #![feature(step_trait)]

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use utca::App;

// fn main() -> Result<(), eframe::Error> {
//     let mut list: Vec<String> = Vec::new();
//     let mut index = None;
//     eframe::run_simple_native("combo test", Default::default(), move |ctx, _| {
//         use eframe::egui;
//         egui::CentralPanel::default().show(ctx, |ui| {
//             let mut show_combo = |ui: &mut egui::Ui| {
//                 ui.set_min_height(100.0);
//                 ui.horizontal(|ui| {
//                     let mut remove_at = None;
//                     egui::ComboBox::from_id_source("select")
//                         .width(125.0)
//                         .selected_text(match index {
//                             // for some reason this is necessary
//                             Some(i) => &list[i] as &str,
//                             None => "none",
//                         })
//                         .show_ui(ui, |ui| {
//                             if ui.selectable_label(index.is_none(), "none").clicked() {
//                                 index = None;
//                             }
//                             for (i, pak) in list.iter().enumerate() {
//                                 ui.horizontal(|ui| {
//                                     if ui.selectable_label(index == Some(i), pak).clicked() {
//                                         index = Some(i);
//                                     }
//                                     if ui.button("x").clicked() {
//                                         remove_at = Some(i);
//                                         if index == remove_at {
//                                             index = None
//                                         }
//                                         if index.is_some_and(|index| index > i) {
//                                             index.as_mut().map(|i| *i -= 1);
//                                         }
//                                     }
//                                 });
//                             }
//                         });
//                     if let Some(i) = remove_at {
//                         list.remove(i);
//                     }
//                     if ui.button("+").clicked() {
//                         list.push("test".to_string())
//                     }
//                 })
//             };
//             show_combo(ui);
//             ui.menu_button("test", show_combo);
//         });
//     })
// }

// When compiling natively
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> eframe::Result<()> {
    use eframe::run_native;

    unsafe { std::env::set_var("POLARS_FMT_MAX_COLS", "256") };
    // unsafe { std::env::set_var("POLARS_FMT_MAX_ROWS", "32") };
    unsafe { std::env::set_var("POLARS_FMT_TABLE_CELL_LIST_LEN", "256") };
    unsafe { std::env::set_var("POLARS_FMT_STR_LEN", "256") };

    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();
    run_native(
        "UTCA",
        Default::default(),
        Box::new(|context| Ok(Box::new(App::new(context)))),
    )
}

// When compiling to web using trunk
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::{
        WebLogger, WebRunner,
        wasm_bindgen::JsCast as _,
        web_sys::{HtmlCanvasElement, window},
    };
    use wasm_bindgen_futures::spawn_local;

    // Redirect `log` message to `console.log` and friends
    WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = Default::default();
    spawn_local(async {
        let document = window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(App::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
