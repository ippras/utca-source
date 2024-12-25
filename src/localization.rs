use self::locales::{EN, RU};
use crate::app::icon;
use egui::{Response, Ui};
use egui_phosphor::regular::TRANSLATE;
use fluent::{FluentResource, concurrent::FluentBundle};
use fluent_content::Content;
use std::sync::{Arc, LazyLock, RwLock};
use tracing::{Level, enabled, error};
use unic_langid::LanguageIdentifier;

pub(crate) macro lowercase($key:expr) {
    LOCALIZATION.read().unwrap().0.content($key)
}

pub(crate) macro localize($key:literal) {
    match LOCALIZATION.read().unwrap().0.content($key) {
        Some(content) => {
            let mut chars = content.chars();
            chars
                .next()
                .map(char::to_uppercase)
                .into_iter()
                .flatten()
                .chain(chars)
                .collect()
        }
        None => $key.to_uppercase(),
    }
}

pub(crate) static LOCALIZATION: LazyLock<RwLock<Localization>> =
    LazyLock::new(|| RwLock::new(Localization::new(EN)));

/// Localization
#[derive(Clone)]
pub(crate) struct Localization(Arc<FluentBundle<FluentResource>>);

impl Localization {
    fn new(locale: LanguageIdentifier) -> Self {
        let sources = match locale {
            EN => sources::EN,
            RU => sources::RU,
            _ => unreachable!(),
        };
        let mut bundle = FluentBundle::new_concurrent(vec![locale.into()]);
        for &source in sources {
            let resource = match FluentResource::try_new(source.to_owned()) {
                Ok(resource) => resource,
                Err((resource, errors)) => {
                    if enabled!(Level::WARN) {
                        for error in errors {
                            error!(%error);
                        }
                    }
                    resource
                }
            };
            if let Err(errors) = bundle.add_resource(resource) {
                if enabled!(Level::WARN) {
                    for error in errors {
                        error!(%error);
                    }
                }
            }
        }
        Localization(Arc::new(bundle))
    }

    fn locale(&self) -> LanguageIdentifier {
        match self.0.locales[0] {
            EN => EN,
            RU => RU,
            _ => unreachable!(),
        }
    }
}

/// Localization extension methods for [`Ui`]
pub(crate) trait UiExt {
    fn locale_button(&mut self) -> Response;
}

impl UiExt for Ui {
    fn locale_button(&mut self) -> Response {
        self.menu_button(icon!(TRANSLATE, x32), |ui| {
            let mut locale = LOCALIZATION.read().unwrap().locale();
            let mut response = ui.selectable_value(&mut locale, EN, "🇺🇸");
            response |= ui.selectable_value(&mut locale, RU, "🇷🇺");
            if response.changed() {
                *LOCALIZATION.write().unwrap() = Localization::new(locale);
            }
            if response.clicked() {
                ui.close_menu();
            }
        })
        .response
    }
}

mod locales {
    use unic_langid::{LanguageIdentifier, langid};

    pub(super) const EN: LanguageIdentifier = langid!("en");
    pub(super) const RU: LanguageIdentifier = langid!("ru");
}

mod sources {
    macro source($path:literal) {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), $path))
    }

    pub(super) const EN: &[&str] = &[
        source!("/ftl/en/fatty_acids.ftl"),
        source!("/ftl/en/properties.ftl"),
        source!("/ftl/en/bars/top.ftl"),
    ];

    pub(super) const RU: &[&str] = &[
        source!("/ftl/ru/fatty_acids.ftl"),
        source!("/ftl/ru/properties.ftl"),
        source!("/ftl/ru/bars/top.ftl"),
    ];
}
