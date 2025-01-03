use crate::{localization::localize, utils::spawn};
use anyhow::{Error, Result};
use base64::prelude::*;
use egui::{
    CollapsingHeader, Context, Grid, Id, Label, RichText, ScrollArea, Sense, TextEdit, Ui, Widget,
    Window,
};
use egui_phosphor::regular::CLOUD_ARROW_DOWN;
use ehttp::{fetch, fetch_async, Headers, Request, Response};
use itertools::{Either, Itertools};
use poll_promise::Promise;
use radix_trie::{iter::Children, SubTrie, Trie, TrieCommon};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    env::var,
    f32::INFINITY,
    fmt::{Debug, Display},
    future::Future,
    path::{Components, Path},
    sync::mpsc::Sender,
};
use tracing::{error, info, trace};
use url::Url;

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
    pub open: bool,
    pub token: String,
    promise: Promise<Option<Tree>>,
}

impl Default for Github {
    fn default() -> Self {
        Self::new()
    }
}

impl Github {
    pub fn new() -> Self {
        let token = var("GITHUB_TOKEN").unwrap_or_default();
        Self {
            open: false,
            token,
            promise: Promise::from_ready(None),
        }
    }

    pub fn toggle(&mut self) {
        self.open ^= true;
        self.promise = if self.open {
            load_tree(&self.token, URL)
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
                ui.collapsing(localize!("settings"), |ui| {
                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                        ui.label(localize!("github token"));
                        // ui.text_edit_singleline(&mut self.token)
                        ui.add(TextEdit::singleline(&mut self.token).desired_width(INFINITY));
                        ctx.request_repaint();
                    });
                });
                ui.separator();
                ScrollArea::vertical().show(ui, |ui| {
                    if let Some(Some(tree)) = self.promise.ready() {
                        let mut trie = Trie::new();
                        for node in &tree.tree {
                            // if node.r#type == "blob" {
                            if let Some(path) = node.path.strip_prefix("configs/") {
                                trie.insert(path, &*node.url);
                            }
                        }
                        ui_children(ui, trie.children());
                    } else {
                        ui.spinner();
                    }
                });
            });
    }
}

fn ui_children(ui: &mut Ui, children: Children<'_, &str, &str>) {
    for trie in children.sorted_by_cached_key(|trie| trie.is_leaf()) {
        if let Some(&path) = trie.key() {
            let name = path.rsplit_once('/').map_or(path, |(_, suffix)| suffix);
            if trie.is_leaf() {
                if let Some(&url) = trie.value() {
                    ui.horizontal(|ui| {
                        if ui.button(CLOUD_ARROW_DOWN).on_hover_text(url).clicked() {
                            load_blob(ui.ctx(), name, url);
                        }
                        ui.label(name);
                    });
                }
            } else {
                ui.collapsing(RichText::new(name).heading(), |ui| {
                    ui_children(ui, trie.children());
                });
            }
        } else {
            ui_children(ui, trie.children());
        }
    }
}

fn load_tree(github_token: impl ToString, url: impl ToString) -> Promise<Option<Tree>> {
    let url = url.to_string();
    let github_token = github_token.to_string();
    spawn(async {
        match try_load_tree(github_token, url).await {
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

async fn try_load_tree(github_token: impl Display, url: impl ToString) -> Result<Tree> {
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
