use egui::{Align2, Color32, Ui, Vec2b};
use egui_plot::{AxisHints, Bar, BarChart, Line, Plot, PlotPoints};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    hash::{Hash, Hasher},
    iter::zip,
};

/// Composition plot
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlotView<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: Settings,
}

impl<'a> PlotView<'a> {
    pub const fn new(data_frame: &'a DataFrame) -> Self {
        Self {
            data_frame,
            settings: Settings::new(),
        }
    }
}

impl PlotView<'_> {
    fn bar_chart(&self, column: &Column) -> PolarsResult<BarChart> {
        let key = column.struct_()?.field_by_name("Key")?;
        let value = column.struct_()?.field_by_name("Value")?;
        let mean = value.struct_()?.field_by_name("Mean")?;
        // let points: Option<Vec<[f64; 2]>> = zip(key.str()?, mean.f64()?).enumerate()
        //     .map(|(key, value)| Some([key?, value?]))
        //     .collect();
        let bars: Option<Vec<_>> = mean
            .f64()?
            .iter()
            .enumerate()
            // .map(|(index, value)| Some([index as _, value?]))
            .map(|(index, value)| Some(Bar::new(index as _, value?).name("name")))
            .collect();
        Ok(BarChart::new(bars.unwrap()))
        // let bar = Bar::new(x, y).name(name).base_offset(offset);
        // Ok(BarChart::new(PlotPoints::new(points.unwrap())).color(Color32::from_rgb(200, 100, 100)))
    }

    pub(crate) fn ui(&mut self, ui: &mut Ui) {
        let mut plot = Plot::new("plot")
            .allow_drag(self.settings.drag)
            .allow_scroll(self.settings.scroll)
            .custom_x_axes(vec![AxisHints::new_x().label("Time (s)")]);
        if self.settings.legend {
            plot = plot.legend(Default::default());
        }
        plot.show(ui, |plot_ui| {
            for column in self.data_frame.get_columns() {
                plot_ui.bar_chart(self.bar_chart(column).unwrap().name("T (°C)"));
            }
            // plot_ui.line(Self::f_line(data).name("F (kN)"));
            // plot_ui.line(Self::z_line(data).name("Z (mm)"));
        });

        // let Self { context } = self;
        // let p = context.settings.visualization.precision;
        // let percent = context.settings.visualization.percent;
        // match context.settings.visualization.source {
        //     Source::Composition => {
        //         context.compose(ui);
        //         let visualized = ui.memory_mut(|memory| {
        //             memory.caches.cache::<Visualized>().get((&*context).into())
        //         });
        //         ui.vertical_centered_justified(|ui| {
        //             let entry = context.state.entry();
        //             ui.heading(&entry.meta.name);
        //             let mut plot = Plot::new("plot")
        //                 .allow_drag(context.settings.visualization.drag)
        //                 .allow_scroll(context.settings.visualization.scroll)
        //                 .y_axis_formatter(move |y, _, _| {
        //                     let rounded = round_to_decimals(y.value, 5).to_string();
        //                     if percent {
        //                         format!("{rounded}%")
        //                     } else {
        //                         format!("{rounded}")
        //                     }
        //                 });
        //             if context.settings.visualization.legend {
        //                 plot = plot.legend(Default::default());
        //             }
        //             plot.show(ui, |ui| {
        //                 // let mut offsets = HashMap::new();
        //                 for (key, values) in visualized {
        //                     // Bars
        //                     let mut offset = 0.0;
        //                     let x = key.into_inner();
        //                     for (name, value) in values {
        //                         let mut y = value;
        //                         if percent {
        //                             y *= 100.0;
        //                         }
        //                         let bar = Bar::new(x, y).name(name).base_offset(offset);
        //                         let chart = BarChart::new(vec![bar])
        //                             .width(context.settings.visualization.width)
        //                             .name(x)
        //                             .color(color(x as _));
        //                         ui.bar_chart(chart);
        //                         offset += y;
        //                     }
        //                     // Text
        //                     if context.settings.visualization.text.show
        //                         && offset >= context.settings.visualization.text.min
        //                     {
        //                         let y = offset;
        //                         let text = Text::new(
        //                             PlotPoint::new(x, y),
        //                             RichText::new(format!("{y:.p$}"))
        //                                 .size(context.settings.visualization.text.size)
        //                                 .heading(),
        //                         )
        //                         .name(x)
        //                         .color(color(x as _))
        //                         .anchor(Align2::CENTER_BOTTOM);
        //                         ui.text(text);
        //                     }
        //                 }
        //             });
        //         });
        //     }
        //     Source::Comparison => {
        //         match context.settings.visualization.comparison {
        //             Comparison::One => {
        //                 context.compare(ui);
        //                 ui.vertical_centered_justified(|ui| {
        //                     let entry = context.state.entry();
        //                     ui.heading(&entry.meta.name);
        //                     let mut plot = Plot::new(ui.id())
        //                         .allow_drag(context.settings.visualization.drag)
        //                         .allow_scroll(context.settings.visualization.scroll);
        //                     if context.settings.visualization.legend {
        //                         plot = plot.legend(Default::default());
        //                     }
        //                     let base: HashMap<_, _> = entry
        //                         .data
        //                         .composed
        //                         .composition(context.settings.composition.method)
        //                         .leaves()
        //                         .map(|Leaf { data }| (data.tag, data.value))
        //                         .collect();
        //                     plot.show(ui, |ui| {
        //                         for (index, entry) in context
        //                             .state
        //                             .entries
        //                             .iter()
        //                             .enumerate()
        //                             .filter(|&(index, _)| index != context.state.index)
        //                         {
        //                             let mut bars = Vec::new();
        //                             let mut offsets = HashMap::new();
        //                             for Hierarchized(_, item) in entry
        //                                 .data
        //                                 .composed
        //                                 .composition(context.settings.composition.method)
        //                                 .hierarchy()
        //                             {
        //                                 match item {
        //                                     Item::Meta(meta) => {}
        //                                     Item::Data(data) => {
        //                                         let name = context.species(data.tag);
        //                                         let ecn = context.ecn(data.tag).sum();
        //                                         let x = ecn as f64;
        //                                         let mut y = base
        //                                             .get(&data.tag)
        //                                             .map_or(0.0, |value| value.0)
        //                                             - data.value.0;
        //                                         if context.settings.visualization.percent {
        //                                             y *= 100.0;
        //                                         }
        //                                         let key = if y < 0.0 {
        //                                             Offset::Negative(ecn)
        //                                         } else {
        //                                             Offset::Positive(ecn)
        //                                         };
        //                                         let offset = offsets.entry(key).or_default();
        //                                         let bar =
        //                                             Bar::new(x, y).name(name).base_offset(*offset);
        //                                         bars.push(bar);
        //                                         *offset += y;
        //                                     }
        //                                 }
        //                             }
        //                             let chart = BarChart::new(bars)
        //                                 .width(context.settings.visualization.width)
        //                                 .name(&entry.meta.name)
        //                                 .color(color(index));
        //                             ui.bar_chart(chart);
        //                             // // Text
        //                             // for (ecn, y) in offsets {
        //                             //     let x = ecn as f64;
        //                             //     let text = Text::new(
        //                             //         PlotPoint::new(x, y),
        //                             //         RichText::new(format!("{y:.p$}")).heading(),
        //                             //     )
        //                             //     .color(color(ecn))
        //                             //     .name(ecn)
        //                             //     .anchor(Align2::CENTER_BOTTOM);
        //                             //     ui.text(text);
        //                             // }
        //                         }
        //                     });
        //                 });
        //             }
        //             Comparison::Many => {
        //                 context.compare(ui);
        //                 // Plot::new("left-top")
        //                 //     .data_aspect(1.0)
        //                 //     .width(250.0)
        //                 //     .height(250.0)
        //                 //     .link_axis(link_group_id, self.link_x, self.link_y)
        //                 //     .link_cursor(link_group_id, self.link_cursor_x, self.link_cursor_y)
        //                 //     .show(ui, LinkedAxesDemo::configure_plot);
        //                 let height = ui.available_height() / context.settings.visualization.height;
        //                 let group_id = ui.id().with("link");
        //                 ui.vertical_centered_justified(|ui| {
        //                     for (index, entry) in context.state.entries.iter().enumerate() {
        //                         ui.heading(&entry.meta.name);
        //                         let mut plot = Plot::new(ui.id().with(index))
        //                             .height(height)
        //                             .allow_drag(context.settings.visualization.drag)
        //                             .allow_scroll(context.settings.visualization.scroll)
        //                             .link_axis(
        //                                 group_id,
        //                                 context.settings.visualization.links.axis.x,
        //                                 context.settings.visualization.links.axis.y,
        //                             )
        //                             .link_cursor(
        //                                 group_id,
        //                                 context.settings.visualization.links.cursor.x,
        //                                 context.settings.visualization.links.cursor.y,
        //                             )
        //                             .set_margin_fraction(Vec2::new(0.0, 0.15));
        //                         if context.settings.visualization.legend {
        //                             plot = plot.legend(Default::default());
        //                         }
        //                         let mut min = [0.0, 0.0];
        //                         let mut max = [0.0, 0.0];
        //                         plot.show(ui, |ui| {
        //                             let mut offsets = HashMap::new();
        //                             for Hierarchized(_, item) in entry
        //                                 .data
        //                                 .composed
        //                                 .composition(context.settings.composition.method)
        //                                 .hierarchy()
        //                             {
        //                                 match item {
        //                                     Item::Meta(meta) => {}
        //                                     Item::Data(data) => {
        //                                         let name = context.species(data.tag);
        //                                         let ecn = context.ecn(data.tag).sum();
        //                                         let x = ecn as f64;
        //                                         min[0] = x.min(min[0]);
        //                                         max[0] = x.max(max[0]);
        //                                         let mut y = data.value.0;
        //                                         if context.settings.visualization.percent {
        //                                             y *= 100.0;
        //                                         }
        //                                         let offset = offsets.entry(ecn).or_default();
        //                                         let bar =
        //                                             Bar::new(x, y).name(name).base_offset(*offset);
        //                                         let chart = BarChart::new(vec![bar])
        //                                             .width(context.settings.visualization.width)
        //                                             .name(ecn)
        //                                             .color(color(ecn));
        //                                         ui.bar_chart(chart);
        //                                         *offset += y;
        //                                     }
        //                                 }
        //                             }
        //                             // Text
        //                             for (ecn, y) in offsets {
        //                                 let x = ecn as f64;
        //                                 let text = Text::new(
        //                                     PlotPoint::new(x, y),
        //                                     RichText::new(format!("{y:.p$}"))
        //                                         .size(context.settings.visualization.text.size)
        //                                         .heading(),
        //                                 )
        //                                 .color(color(ecn))
        //                                 .name(ecn)
        //                                 .anchor(Align2::CENTER_BOTTOM);
        //                                 ui.text(text);
        //                             }
        //                             // ui.set_plot_bounds(PlotBounds::from_min_max(
        //                             //     [33.0, 0.0],
        //                             //     [51.0, 0.0],
        //                             // ));
        //                             // ui.set_auto_bounds(Vec2b::new(false, true));
        //                         });
        //                     }
        //                 });
        //             }
        //         }
        //     }
        // }
    }
}

/// Visualization plot settings
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) drag: Vec2b,
    pub(crate) scroll: Vec2b,
    pub(crate) legend: bool,
}

impl Hash for Settings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.drag.x.hash(state);
        self.drag.y.hash(state);
        self.scroll.x.hash(state);
        self.scroll.y.hash(state);
    }
}

impl Settings {
    pub const fn new() -> Self {
        Self {
            drag: Vec2b { x: false, y: false },
            scroll: Vec2b { x: false, y: false },
            legend: true,
        }
    }
}

//     pub(crate) fn ui(&mut self, ui: &mut Ui) {
//         // let Self { context } = self;
//         // let p = context.settings.visualization.precision;
//         // let percent = context.settings.visualization.percent;
//         // match context.settings.visualization.source {
//         //     Source::Composition => {
//         //         context.compose(ui);
//         //         let visualized = ui.memory_mut(|memory| {
//         //             memory.caches.cache::<Visualized>().get((&*context).into())
//         //         });
//         //         ui.vertical_centered_justified(|ui| {
//         //             let entry = context.state.entry();
//         //             ui.heading(&entry.meta.name);
//         //             let mut plot = Plot::new("plot")
//         //                 .allow_drag(context.settings.visualization.drag)
//         //                 .allow_scroll(context.settings.visualization.scroll)
//         //                 .y_axis_formatter(move |y, _, _| {
//         //                     let rounded = round_to_decimals(y.value, 5).to_string();
//         //                     if percent {
//         //                         format!("{rounded}%")
//         //                     } else {
//         //                         format!("{rounded}")
//         //                     }
//         //                 });
//         //             if context.settings.visualization.legend {
//         //                 plot = plot.legend(Default::default());
//         //             }
//         //             plot.show(ui, |ui| {
//         //                 // let mut offsets = HashMap::new();
//         //                 for (key, values) in visualized {
//         //                     // Bars
//         //                     let mut offset = 0.0;
//         //                     let x = key.into_inner();
//         //                     for (name, value) in values {
//         //                         let mut y = value;
//         //                         if percent {
//         //                             y *= 100.0;
//         //                         }
//         //                         let bar = Bar::new(x, y).name(name).base_offset(offset);
//         //                         let chart = BarChart::new(vec![bar])
//         //                             .width(context.settings.visualization.width)
//         //                             .name(x)
//         //                             .color(color(x as _));
//         //                         ui.bar_chart(chart);
//         //                         offset += y;
//         //                     }
//         //                     // Text
//         //                     if context.settings.visualization.text.show
//         //                         && offset >= context.settings.visualization.text.min
//         //                     {
//         //                         let y = offset;
//         //                         let text = Text::new(
//         //                             PlotPoint::new(x, y),
//         //                             RichText::new(format!("{y:.p$}"))
//         //                                 .size(context.settings.visualization.text.size)
//         //                                 .heading(),
//         //                         )
//         //                         .name(x)
//         //                         .color(color(x as _))
//         //                         .anchor(Align2::CENTER_BOTTOM);
//         //                         ui.text(text);
//         //                     }
//         //                 }
//         //             });
//         //         });
//         //     }
//         //     Source::Comparison => {
//         //         match context.settings.visualization.comparison {
//         //             Comparison::One => {
//         //                 context.compare(ui);
//         //                 ui.vertical_centered_justified(|ui| {
//         //                     let entry = context.state.entry();
//         //                     ui.heading(&entry.meta.name);
//         //                     let mut plot = Plot::new(ui.id())
//         //                         .allow_drag(context.settings.visualization.drag)
//         //                         .allow_scroll(context.settings.visualization.scroll);
//         //                     if context.settings.visualization.legend {
//         //                         plot = plot.legend(Default::default());
//         //                     }
//         //                     let base: HashMap<_, _> = entry
//         //                         .data
//         //                         .composed
//         //                         .composition(context.settings.composition.method)
//         //                         .leaves()
//         //                         .map(|Leaf { data }| (data.tag, data.value))
//         //                         .collect();
//         //                     plot.show(ui, |ui| {
//         //                         for (index, entry) in context
//         //                             .state
//         //                             .entries
//         //                             .iter()
//         //                             .enumerate()
//         //                             .filter(|&(index, _)| index != context.state.index)
//         //                         {
//         //                             let mut bars = Vec::new();
//         //                             let mut offsets = HashMap::new();
//         //                             for Hierarchized(_, item) in entry
//         //                                 .data
//         //                                 .composed
//         //                                 .composition(context.settings.composition.method)
//         //                                 .hierarchy()
//         //                             {
//         //                                 match item {
//         //                                     Item::Meta(meta) => {}
//         //                                     Item::Data(data) => {
//         //                                         let name = context.species(data.tag);
//         //                                         let ecn = context.ecn(data.tag).sum();
//         //                                         let x = ecn as f64;
//         //                                         let mut y = base
//         //                                             .get(&data.tag)
//         //                                             .map_or(0.0, |value| value.0)
//         //                                             - data.value.0;
//         //                                         if context.settings.visualization.percent {
//         //                                             y *= 100.0;
//         //                                         }
//         //                                         let key = if y < 0.0 {
//         //                                             Offset::Negative(ecn)
//         //                                         } else {
//         //                                             Offset::Positive(ecn)
//         //                                         };
//         //                                         let offset = offsets.entry(key).or_default();
//         //                                         let bar =
//         //                                             Bar::new(x, y).name(name).base_offset(*offset);
//         //                                         bars.push(bar);
//         //                                         *offset += y;
//         //                                     }
//         //                                 }
//         //                             }
//         //                             let chart = BarChart::new(bars)
//         //                                 .width(context.settings.visualization.width)
//         //                                 .name(&entry.meta.name)
//         //                                 .color(color(index));
//         //                             ui.bar_chart(chart);
//         //                             // // Text
//         //                             // for (ecn, y) in offsets {
//         //                             //     let x = ecn as f64;
//         //                             //     let text = Text::new(
//         //                             //         PlotPoint::new(x, y),
//         //                             //         RichText::new(format!("{y:.p$}")).heading(),
//         //                             //     )
//         //                             //     .color(color(ecn))
//         //                             //     .name(ecn)
//         //                             //     .anchor(Align2::CENTER_BOTTOM);
//         //                             //     ui.text(text);
//         //                             // }
//         //                         }
//         //                     });
//         //                 });
//         //             }
//         //             Comparison::Many => {
//         //                 context.compare(ui);
//         //                 // Plot::new("left-top")
//         //                 //     .data_aspect(1.0)
//         //                 //     .width(250.0)
//         //                 //     .height(250.0)
//         //                 //     .link_axis(link_group_id, self.link_x, self.link_y)
//         //                 //     .link_cursor(link_group_id, self.link_cursor_x, self.link_cursor_y)
//         //                 //     .show(ui, LinkedAxesDemo::configure_plot);
//         //                 let height = ui.available_height() / context.settings.visualization.height;
//         //                 let group_id = ui.id().with("link");
//         //                 ui.vertical_centered_justified(|ui| {
//         //                     for (index, entry) in context.state.entries.iter().enumerate() {
//         //                         ui.heading(&entry.meta.name);
//         //                         let mut plot = Plot::new(ui.id().with(index))
//         //                             .height(height)
//         //                             .allow_drag(context.settings.visualization.drag)
//         //                             .allow_scroll(context.settings.visualization.scroll)
//         //                             .link_axis(
//         //                                 group_id,
//         //                                 context.settings.visualization.links.axis.x,
//         //                                 context.settings.visualization.links.axis.y,
//         //                             )
//         //                             .link_cursor(
//         //                                 group_id,
//         //                                 context.settings.visualization.links.cursor.x,
//         //                                 context.settings.visualization.links.cursor.y,
//         //                             )
//         //                             .set_margin_fraction(Vec2::new(0.0, 0.15));
//         //                         if context.settings.visualization.legend {
//         //                             plot = plot.legend(Default::default());
//         //                         }
//         //                         let mut min = [0.0, 0.0];
//         //                         let mut max = [0.0, 0.0];
//         //                         plot.show(ui, |ui| {
//         //                             let mut offsets = HashMap::new();
//         //                             for Hierarchized(_, item) in entry
//         //                                 .data
//         //                                 .composed
//         //                                 .composition(context.settings.composition.method)
//         //                                 .hierarchy()
//         //                             {
//         //                                 match item {
//         //                                     Item::Meta(meta) => {}
//         //                                     Item::Data(data) => {
//         //                                         let name = context.species(data.tag);
//         //                                         let ecn = context.ecn(data.tag).sum();
//         //                                         let x = ecn as f64;
//         //                                         min[0] = x.min(min[0]);
//         //                                         max[0] = x.max(max[0]);
//         //                                         let mut y = data.value.0;
//         //                                         if context.settings.visualization.percent {
//         //                                             y *= 100.0;
//         //                                         }
//         //                                         let offset = offsets.entry(ecn).or_default();
//         //                                         let bar =
//         //                                             Bar::new(x, y).name(name).base_offset(*offset);
//         //                                         let chart = BarChart::new(vec![bar])
//         //                                             .width(context.settings.visualization.width)
//         //                                             .name(ecn)
//         //                                             .color(color(ecn));
//         //                                         ui.bar_chart(chart);
//         //                                         *offset += y;
//         //                                     }
//         //                                 }
//         //                             }
//         //                             // Text
//         //                             for (ecn, y) in offsets {
//         //                                 let x = ecn as f64;
//         //                                 let text = Text::new(
//         //                                     PlotPoint::new(x, y),
//         //                                     RichText::new(format!("{y:.p$}"))
//         //                                         .size(context.settings.visualization.text.size)
//         //                                         .heading(),
//         //                                 )
//         //                                 .color(color(ecn))
//         //                                 .name(ecn)
//         //                                 .anchor(Align2::CENTER_BOTTOM);
//         //                                 ui.text(text);
//         //                             }
//         //                             // ui.set_plot_bounds(PlotBounds::from_min_max(
//         //                             //     [33.0, 0.0],
//         //                             //     [51.0, 0.0],
//         //                             // ));
//         //                             // ui.set_auto_bounds(Vec2b::new(false, true));
//         //                         });
//         //                     }
//         //                 });
//         //             }
//         //         }
//         //     }
//         // }
//     }
