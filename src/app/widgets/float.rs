use egui::{DragValue, InnerResponse, RichText, Ui, Widget, vec2};
use polars::prelude::*;

/// Float widget
pub(crate) struct FloatWidget<'a> {
    value: Box<dyn Fn() -> PolarsResult<Option<f64>> + 'a>,
    settings: Settings,
}

impl<'a> FloatWidget<'a> {
    pub(crate) fn new(value: impl Fn() -> PolarsResult<Option<f64>> + 'a) -> Self {
        Self {
            value: Box::new(value),
            settings: Settings::default(),
        }
    }

    pub(crate) fn disable(mut self, disable: bool) -> Self {
        self.settings.disable = disable;
        self
    }

    pub(crate) fn editable(mut self, editable: bool) -> Self {
        self.settings.editable = editable;
        self
    }

    pub(crate) fn hover(mut self) -> Self {
        self.settings.hover = true;
        self
    }

    pub(crate) fn percent(mut self, percent: bool) -> Self {
        self.settings.percent = percent;
        self
    }

    pub(crate) fn precision(mut self, precision: Option<usize>) -> Self {
        self.settings.precision = precision;
        self
    }

    pub(crate) fn try_show(self, ui: &mut Ui) -> PolarsResult<InnerResponse<Option<f64>>> {
        let format = |value: f64| match self.settings.precision {
            Some(precision) => format!("{value:.precision$}"),
            None => AnyValue::from(value).to_string(),
        };
        let mut inner = None;
        if self.settings.disable {
            ui.disable();
        }
        let response = if let Some(mut value) = (self.value)()? {
            // Percent
            if self.settings.percent {
                value *= 100.0;
            }
            // Editable
            let mut response = if self.settings.editable {
                // Writable
                let response = ui.add_sized(
                    vec2(ui.available_width(), ui.style().spacing.interact_size.y),
                    DragValue::new(&mut value)
                        .range(0.0..=f64::MAX)
                        .custom_formatter(|value, _| format(value)),
                );
                if response.changed() {
                    inner = Some(value)
                }
                response
            } else {
                // Readable
                ui.label(format(value))
            };
            if self.settings.hover {
                response =
                    response.on_hover_text(RichText::new(AnyValue::Float64(value).to_string()));
            }
            response
        } else {
            // Null
            ui.label(AnyValue::Null.to_string())
        };
        Ok(InnerResponse::new(inner, response))
    }

    pub(crate) fn show(self, ui: &mut Ui) -> InnerResponse<Option<f64>> {
        self.try_show(ui).expect("Float widget")
    }
}

impl Widget for FloatWidget<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        self.show(ui).response
    }
}

/// Settings
#[derive(Clone, Copy, Debug, Default)]
struct Settings {
    editable: bool,
    disable: bool,
    percent: bool,
    hover: bool,
    precision: Option<usize>,
}
