use crate::{
    learned::generic::{ApproxPos, Model, Segmentation},
    Entry,
};

use super::{pgm_model::LinearModel, pgm_segmentation::SimplePGMSegmentator};
use egui::plot;
use rand::{distributions::Uniform, random, Rng};
use std::borrow::Borrow;

type Key = i32;
type Value = i32;
const EPSILON: usize = 4;
type OurModel = LinearModel<Key, EPSILON>;

struct AppState {
    num_entries: usize,
    keys: Vec<Key>,
    models: Vec<OurModel>,
    cur_segment: SimplePGMSegmentator<Key, Value, EPSILON>,
    // For controlling how we step through
    adding_ix: usize,
    batch_add_size: usize,
    // For rendering the models at the proper y value, with proper length
    model_ranks: Vec<usize>,
    // For querying a point and showing predictions
    query_point: Option<Entry<Key, Value>>,
}

impl AppState {
    /// Create an empty app state
    fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        Self {
            num_entries: 200,
            keys: vec![],
            models: vec![],
            cur_segment: SimplePGMSegmentator::new(),
            adding_ix: 0,
            batch_add_size: 5,
            model_ranks: vec![],
            query_point: None,
        }
    }

    /// Resete everything (data + state)
    fn reset(&mut self) {
        self.keys.clear();
        let range = Uniform::from((Key::MIN)..(Key::MAX));
        let mut random_values: Vec<i32> = rand::thread_rng().sample_iter(&range).take(self.num_entries).collect();
        random_values.sort();
        random_values.dedup();
        self.keys = random_values;
        self.reset_state();
    }

    /// Reset the app state
    /// Useful when you want to do basically another round of training on a fresh layer
    fn reset_state(&mut self) {
        self.models.clear();
        self.model_ranks.clear();
        self.cur_segment = SimplePGMSegmentator::new();
        self.adding_ix = 0;
    }

    /// Add the next `self.batch_add_size` keys to layer
    /// If there are no more keys, and the current model has elements, it will wrap them up into
    /// a model and add it. Otherwise it does nothing
    fn add_batched_elements(&mut self) {
        if self.keys.len() <= 0 {
            return;
        }
        let ceil = (self.keys.len() - 1).min(self.adding_ix + self.batch_add_size);
        if self.adding_ix >= ceil {
            // We've ran through all the elements
            if self.cur_segment.num_entries > 0 {
                // Push the last segment if it has elements
                self.models.push(self.cur_segment.to_linear_model());
                self.model_ranks.push(self.num_entries - self.cur_segment.num_entries);
                self.cur_segment = SimplePGMSegmentator::new();
            }
            return;
        }
        // Otherwise add as many elements as we can
        while self.adding_ix < ceil {
            let entry: Entry<Key, Value> = Entry::new(self.keys[self.adding_ix], 0);
            match self.cur_segment.try_add_entry(entry) {
                Ok(_) => {
                    // Nothing to do, move on to next
                    self.adding_ix += 1;
                }
                Err(_) => {
                    // Export model and clear
                    self.models.push(self.cur_segment.to_linear_model());
                    self.model_ranks.push(self.adding_ix - self.cur_segment.num_entries);
                    self.cur_segment = SimplePGMSegmentator::new();
                }
            }
        }
    }

    /// Resets the state, and then fully trains on the data provided to create an entire layer
    fn full_train(&mut self) {
        self.reset_state();
        let entries: Vec<Entry<Key, Value>> = self
            .keys
            .iter()
            .enumerate()
            .map(|(ix, key)| Entry::new(key.clone(), ix as i32))
            .collect();
        let trained_result = OurModel::make_segmentation(entries.into_iter());
        for ix in 0..(trained_result.len()) {
            let (model, value) = trained_result[ix];
            self.models.push(model);
            self.model_ranks.push(value as usize);
        }
        self.adding_ix = self.num_entries;
    }

    /// Selects a random entry, and then shows the upper and lower bound for predicted position
    fn do_query(&mut self) {
        if self.keys.len() <= 0 {
            return;
        }
        let ix: usize = (rand::thread_rng().gen::<usize>()) % self.keys.len();
        self.query_point = Some(Entry::new(self.keys[ix] as Key, ix.try_into().unwrap()));
    }

    /// Returns the predicted upper and lower positions
    fn get_query_bounds(&self) -> Option<ApproxPos> {
        if self.query_point.is_none() || self.models.len() <= 0 {
            return None;
        }
        let mut model_ix = 0;
        while model_ix < self.models.len().saturating_sub(1) {
            if self.models[model_ix + 1].key >= self.query_point.unwrap().key {
                break;
            }
            model_ix += 1;
        }
        let base_rank = self.model_ranks[model_ix];
        let range = self.models[model_ix].approximate(&self.query_point.unwrap().key);
        return Some(ApproxPos {
            lo: base_rank + range.lo,
            hi: base_rank + range.hi,
        });
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::plot::Plot::new("pgm_plot")
                .view_aspect(2.0)
                .auto_bounds_x()
                .auto_bounds_y()
                .show_x(true)
                .show_y(false)
                .legend(egui::plot::Legend::default())
                .allow_drag(false)
                .allow_zoom(false)
                .show(ui, |plot_ui| {
                    use egui::plot::*;

                    let points = PlotPoints::new(
                        self.keys
                            .iter()
                            .copied()
                            .enumerate()
                            .map(|(rank, key)| [key as f64, rank as f64])
                            .collect(),
                    );
                    plot_ui.points(Points::new(points).radius(5.0).name("key-ranks"));

                    // Plot the models
                    assert!(self.models.len() == self.model_ranks.len());
                    let mut model_lines: Vec<Line> = vec![];
                    for ix in 0..(self.models.len()) {
                        let model = self.models[ix];
                        let model_rank = self.model_ranks[ix];
                        let first_key = model.key;
                        let end_key = self.keys[(self.keys.len() - 1).min(model_rank + model.size)];
                        let end_rank = model_rank as f64 + ((end_key.saturating_sub(first_key)) as f64 * model.slope);
                        let line = Line::new(PlotPoints::new(vec![
                            [first_key as f64, model_rank as f64],
                            [end_key as f64, end_rank],
                        ]))
                        .width(8.0);
                        model_lines.push(line);
                    }
                    for line in model_lines {
                        plot_ui.line(line);
                    }

                    // Plot the current segment once it has enough entries to be a little stable (not huge slopes)
                    if self.cur_segment.num_entries > 3 {
                        let first_key = self.cur_segment.first_k.unwrap() as f64;
                        let cur_key = self.keys[self.adding_ix] as f64;
                        let first_rank = (self.adding_ix - self.cur_segment.num_entries) as f64;
                        let max_rank = first_rank + (cur_key - first_key) * self.cur_segment.max_slope;
                        let min_rank = first_rank + (cur_key - first_key) * self.cur_segment.min_slope;
                        let segment_max_line =
                            Line::new(PlotPoints::new(vec![[first_key, first_rank], [cur_key, max_rank]])).width(2.0);
                        plot_ui.line(segment_max_line);
                        let segment_min_line =
                            Line::new(PlotPoints::new(vec![[first_key, first_rank], [cur_key, min_rank]])).width(2.0);
                        plot_ui.line(segment_min_line);
                    }

                    // Plot the point we're querying and the bounds
                    if self.query_point.is_some() && self.get_query_bounds().is_some() {
                        let approx = self.get_query_bounds().unwrap();
                        let points = PlotPoints::new(vec![
                            [self.query_point.unwrap().key as f64, approx.lo as f64],
                            [
                                self.query_point.unwrap().key as f64,
                                self.query_point.unwrap().value as f64,
                            ],
                            [self.query_point.unwrap().key as f64, approx.hi as f64],
                        ]);
                        plot_ui.points(Points::new(points).radius(9.0).name("query"));
                    }
                });

            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.num_entries));
                if ui.button("Number of Entries").clicked() {
                    self.reset();
                }
            });

            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.batch_add_size));
                if ui.button("Add Entries").clicked() {
                    self.add_batched_elements();
                }
            });

            if ui.button("Train Layer").clicked() {
                self.full_train();
            }

            if ui.button("Reroll + Retrain").clicked() {
                self.reset();
                self.full_train();
            }

            if ui.button("Reset State").clicked() {
                self.reset_state();
            }

            if ui.button("Do Query").clicked() {
                self.do_query();
            }
        });
    }
}

pub fn play_pgm() -> Result<(), Box<dyn std::error::Error>> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Piecewise Geometric Model Segmentation Algorithm",
        native_options,
        Box::new(|cc| Box::new(AppState::new(cc))),
    )?;

    Ok(())
}
