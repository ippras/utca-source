use egui::{DragValue, InnerResponse, RichText, Ui, Widget, vec2};
use polars::prelude::*;

/// Float widget
pub(crate) struct FloatWidget<'a> {
    pub(crate) value: Box<dyn Fn() -> PolarsResult<Option<f64>> + 'a>,
    pub(crate) editable: bool,
    pub(crate) disable: bool,
    pub(crate) percent: bool,
    pub(crate) hover: bool,
    pub(crate) precision: Option<usize>,
}

impl<'a> FloatWidget<'a> {
    pub(crate) fn new(value: impl Fn() -> PolarsResult<Option<f64>> + 'a) -> Self {
        Self {
            value: Box::new(value),
            editable: false,
            disable: false,
            percent: true,
            hover: false,
            precision: None,
        }
    }

    pub(crate) fn disable(self, disable: bool) -> Self {
        Self { disable, ..self }
    }

    pub(crate) fn editable(self, editable: bool) -> Self {
        Self { editable, ..self }
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

    pub(crate) fn try_ui(self, ui: &mut Ui) -> PolarsResult<InnerResponse<Option<f64>>> {
        let format = |value: f64| match self.precision {
            Some(precision) => format!("{value:.precision$}"),
            None => AnyValue::from(value).to_string(),
        };
        let mut inner = None;
        let response = if let Some(mut value) = (self.value)()? {
            // Editable
            let mut response = if self.editable {
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
            if self.hover {
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

    pub(crate) fn ui(self, ui: &mut Ui) -> InnerResponse<Option<f64>> {
        self.try_ui(ui).expect("Float widget")
    }
}

impl Widget for FloatWidget<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        self.ui(ui).response
    }
}
