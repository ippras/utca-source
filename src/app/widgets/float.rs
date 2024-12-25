use egui::{Response, RichText, Ui, Widget};
use polars::prelude::AnyValue;

/// Float value
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FloatValue {
    pub(crate) value: Option<f64>,
    pub(crate) disable: bool,
    pub(crate) hover: bool,
    pub(crate) percent: bool,
    pub(crate) precision: Option<usize>,
}

impl FloatValue {
    pub(crate) fn new(value: Option<f64>) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }

    pub(crate) fn disable(self, disable: bool) -> Self {
        Self { disable, ..self }
    }

    pub(crate) fn hover(self) -> Self {
        Self {
            hover: true,
            ..self
        }
    }

    pub(crate) fn percent(self, percent: bool) -> Self {
        Self { percent, ..self }
    }

    pub(crate) fn precision(self, precision: Option<usize>) -> Self {
        Self { precision, ..self }
    }
}

impl Widget for FloatValue {
    fn ui(self, ui: &mut Ui) -> Response {
        if self.disable {
            ui.disable();
        }
        let text = match self.value {
            None => {
                // let text = RichText::new(AnyValue::Float64(0.0).to_string());
                let text = RichText::new(AnyValue::Null.to_string());
                text
            }
            Some(mut value) => {
                if self.percent {
                    value *= 100.0;
                }
                match self.precision {
                    Some(precision) => RichText::new(format!("{value:.precision$}")),
                    None => RichText::new(AnyValue::from(value).to_string()),
                }
            }
        };
        let mut response = ui.label(text);
        if self.hover {
            let mut value = self.value.unwrap_or_default();
            if self.percent {
                value *= 100.0;
            }
            let text = RichText::new(AnyValue::Float64(value).to_string());
            response = response
                .on_hover_text(text.clone())
                .on_disabled_hover_text(text);
        }
        response
    }
}
