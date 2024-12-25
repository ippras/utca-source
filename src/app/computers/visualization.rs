use crate::app::panes::visualization::control::Settings;
use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use poll_promise::Promise;
use std::hash::{Hash, Hasher};

/// Visualization computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Visualization computer
#[derive(Default)]
pub(crate) struct Computer;

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        DataFrame::empty()
    }
}

/// Visualization key
#[derive(Clone, Copy, Debug)]
pub(crate) struct Key<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // for fatty_acid in self.fatty_acids["FA"].phys_iter() {
        //     fatty_acid.hash(state);
        // }
        // for tag in self.fatty_acids["TAG"].phys_iter() {
        //     tag.hash(state);
        // }
        // for dag1223 in self.fatty_acids["DAG1223"].phys_iter() {
        //     dag1223.hash(state);
        // }
        // for mag2 in self.fatty_acids["MAG2"].phys_iter() {
        //     mag2.hash(state);
        // }
        self.settings.hash(state);
    }
}

/// Value
type Value = DataFrame;
