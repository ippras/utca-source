use anyhow::{Error, Result};
use base64::prelude::*;
use egui::{
    CollapsingHeader, Context, Grid, Id, Label, RichText, ScrollArea, Sense, Ui, Widget, Window,
};
use egui_phosphor::regular::CLOUD_ARROW_DOWN;
use ehttp::{fetch, fetch_async, Headers, Request, Response};
use itertools::Itertools;
use poll_promise::Promise;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{env::var, fmt::Debug, future::Future, sync::mpsc::Sender};
use tracing::{error, info, trace};
use url::Url;

use crate::utils::spawn;

// https://api.github.com/repos/ippras/utca/gh-pages/configs/H242_Tamia_Peroxide.toml
// /repos/repos/ippras/git/trees/{tree_sha}
// const URL: &str = "https://api.github.com/repos/ippras/utca/contents/configs";
// const URL: &str = "https://api.github.com/repos/ippras/utca/contents/configs";
// /repos/{owner}/{repo}/git/trees/{tree_sha}
// https://api.github.com/repos/ippras/utca/git/trees/gh-pages?recursive=true
// https://api.github.com/repos/ippras/utca/git/trees/gh-pages/configs?recursive=true

const URL: &str = "https://api.github.com/repos/ippras/utca/git/trees/gh-pages?recursive=true";
// const GITHUB_TOKEN: &str = env!("GITHUB_TOKEN");

/// `github.com tree` renders a nested list of debugger values.
pub struct Github {
    pub url: String,
    pub open: bool,
    promise: Promise<Option<Tree>>,
}

impl Default for Github {
    fn default() -> Self {
        Self::new(URL)
    }
}

impl Github {
    pub fn new(url: impl ToString) -> Self {
        Self {
            url: url.to_string(),
            open: false,
            promise: Promise::from_ready(None),
        }
    }

    pub fn toggle(&mut self) {
        self.open ^= true;
        self.promise = if self.open {
            load_tree(URL)
        } else {
            Promise::from_ready(None)
        };
    }

    // if self.show_confirmation_dialog {
    //     egui::Window::new("Do you want to quit?")
    //         .collapsible(false)
    //         .resizable(false)
    //         .show(ctx, |ui| {
    //             ui.horizontal(|ui| {
    //                 if ui.button("No").clicked() {
    //                     self.show_confirmation_dialog = false;
    //                     self.allowed_to_close = false;
    //                 }
    //                 if ui.button("Yes").clicked() {
    //                     self.show_confirmation_dialog = false;
    //                     self.allowed_to_close = true;
    //                     ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    //                 }
    //             });
    //         });
    // }

    pub fn window(&mut self, ctx: &Context) {
        Window::new(format!("{CLOUD_ARROW_DOWN} Load config"))
            .open(&mut self.open)
            .show(ctx, |ui| {
                ui.visuals_mut().collapsing_header_frame = true;
                ScrollArea::vertical().show(ui, |ui| {
                    if let Some(Some(tree)) = self.promise.ready() {
                        let chunks = tree
                            .tree
                            .iter()
                            .filter(|node| node.r#type == "blob")
                            .filter_map(|node| {
                                Some((node.path.strip_prefix("configs/")?, &node.url))
                            })
                            .chunk_by(|(path, _)| path.split_once('/').map(|(prefix, _)| prefix));
                        for (key, group) in &chunks {
                            // let heading = key.unwrap_or("Root");
                            if let Some(key) = key {
                                ui.collapsing(RichText::new(key).heading(), |ui| {
                                    for (path, url) in group {
                                        let path =
                                            path.strip_prefix(&format!("{key}/")).unwrap_or(path);
                                        ui.horizontal(|ui| {
                                            if ui
                                                .button(CLOUD_ARROW_DOWN)
                                                .on_hover_text(url)
                                                .clicked()
                                            {
                                                load_blob(ctx, path, url);
                                            }
                                            ui.label(path);
                                        });
                                    }
                                });
                            }
                        }
                    } else {
                        ui.spinner();
                    }
                });
            });
    }
}

struct PathUrl {
    
}

// impl Widget for &mut GithubTree {
//     fn ui(self, ui: &mut Ui) -> Response {
//         if let Some(entries) = self.promise.ready() {
//             for entry in entries {
//                 match entry.r#type {
//                     Type::Dir => {
//                         ui.collapsing(&entry.name, |ui| {
//                             // load(self.url)
//                         });
//                     }
//                     Type::File => {
//                         ui.horizontal(|ui| {
//                             ui.label(&entry.name);
//                             if ui.button("📥").clicked() {
//                                 if let Some(url) = &entry.download_url {
//                                     let promise: Promise<String> = load(url);
//                                 }
//                             }
//                         });
//                     }
//                 }
//             }
//         }
//         ui.spinner()
//         // Grid::new(ui.next_auto_id())
//         //     .num_columns(3)
//         //     .striped(true)
//         //     .show(ui, |ui| {
//         //         for value in self.values {
//         //             if value.children().count() > 0 {
//         //                 CollapsingHeader::new(value.name().expect("name should be present"))
//         //                     .id_source(ui.next_auto_id())
//         //                     .show(ui, |ui| {
//         //                         ui.add(VariableList::new(value.children()));
//         //                     });
//         //             } else {
//         //                 ui.label(value.name().unwrap_or_default());
//         //                 ui.label(value.display_type_name().unwrap_or_default());
//         //                 ui.label(value.value().unwrap_or_default());
//         //             }
//         //             ui.end_row();
//         //         }
//         //     })
//         //     .response
//     }
// }

// fn load(url: impl ToString) -> Promise<Vec<Entry>> {
//     let request = Request {
//         headers: Headers::new(&[
//             ("Accept", "application/vnd.github+json"),
//             ("Authorization", &format!("Bearer {GITHUB_TOKEN}")),
//             ("X-GitHub-Api-Version", "2022-11-28"),
//         ]),
//         ..Request::get(format!("{}?recursive=true", url.to_string()))
//     };
//     let (sender, promise) = Promise::new();
//     fetch(request, move |response| match response {
//         Ok(response) => match response.json::<Vec<Entry>>() {
//             Ok(mut entries) => {
//                 println!("entries: {entries:#?}");
//                 entries.sort_by_key(|entry| entry.r#type);
//                 sender.send(entries);
//             }
//             Err(error) => {
//                 error!(%error);
//                 info!("Status code: {}", response.status);
//                 sender.send(Default::default());
//             }
//         },
//         Err(error) => {
//             error!(%error);
//             sender.send(Default::default());
//         }
//     });
//     promise
// }
fn load_tree(url: impl ToString) -> Promise<Option<Tree>> {
    let url = url.to_string();
    spawn(async {
        match try_load_tree(url).await {
            Ok(tree) => Some(tree),
            Err(error) => {
                error!(%error);
                None
            }
        }
    })
    // let (sender, promise) = Promise::new();
    // fetch(request, move |response| {
    //     if let Err(error) = try_load_tree(sender, response) {
    //         error!(%error);
    //     }
    //     // match response {
    //     // Ok(response) => match response.json::<Tree>() {
    //     //     Ok(tree) => sender.send(tree),
    //     //     Err(error) => {
    //     //         error!(%error);
    //     //         info!("Status code: {}", response.status);
    //     //         sender.send(Default::default());
    //     //     }
    //     // },
    //     // Err(error) => {
    //     //     error!(%error);
    //     //     sender.send(Default::default());
    //     // }
    // });
    // promise
}

fn load_blob(ctx: &Context, name: impl ToString, url: impl ToString) {
    let ctx = ctx.clone();
    let name = name.to_string();
    let url = url.to_string();
    let _ = spawn(async move {
        match try_load_blob(url).await {
            Ok(blob) => ctx.data_mut(|data| {
                if let Some(sender) = data.get_temp::<Sender<(String, String)>>(Id::new("Data")) {
                    sender.send((name, blob)).ok();
                }
            }),
            Err(error) => error!(%error),
        }
    });
}

async fn try_load_tree(url: impl ToString) -> Result<Tree> {
    let github_token = var("GITHUB_TOKEN").expect("GITHUB_TOKEN not found");
    let request = Request {
        headers: Headers::new(&[
            ("Accept", "application/vnd.github+json"),
            ("Authorization", &format!("Bearer {github_token}")),
            ("X-GitHub-Api-Version", "2022-11-28"),
        ]),
        ..Request::get(url)
    };
    let response = fetch_async(request).await.map_err(Error::msg)?;
    let tree = response.json::<Tree>()?;
    Ok(tree)
}

async fn try_load_blob(url: impl ToString) -> Result<String> {
    let request = Request::get(url);
    let response = fetch_async(request).await.map_err(Error::msg)?;
    let blob = response.json::<Blob>()?;
    trace!(?blob);
    let mut content = String::new();
    for line in blob.content.split_terminator('\n') {
        content.push_str(&String::from_utf8(BASE64_STANDARD.decode(line)?)?);
    }
    Ok(content)
}

/// Tree
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct Tree {
    sha: String,
    url: String,
    truncated: bool,
    tree: Vec<Node>,
}

/// Node
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Node {
    path: String,
    mode: String,
    r#type: String,
    sha: String,
    size: Option<u64>,
    url: String,
}

/// Blob
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Blob {
    content: String,
    encoding: String,
    url: String,
    sha: String,
    size: u64,
    node_id: String,
}

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct Entry {
//     pub name: String,
//     pub size: usize,
//     pub download_url: Option<Url>,
//     pub r#type: Type,
// }

// #[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
// #[serde(rename_all = "lowercase")]
// pub enum Type {
//     Dir,
//     File,
// }
