use crate::Entry;

use super::{pgm_model::LinearModel, pgm_segmentation::SimplePGMSegmentator};
use egui::plot;
use rand::{distributions::Uniform, random, Rng};
use std::borrow::Borrow;

type Key = i32;
type Value = i32;
const EPSILON: usize = 4;

struct AppState {
    num_entries: usize,
    keys: Vec<Key>,
    models: Vec<(LinearModel<Key, EPSILON>, usize, usize)>,
    cur_segment: SimplePGMSegmentator<Key, Value, EPSILON>,
    // For controlling how we step through
    adding_ix: usize,
    batch_add_size: usize,
}

impl AppState {
    fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        Self {
            num_entries: 200,
            keys: vec![],
            models: vec![],
            cur_segment: SimplePGMSegmentator::new(),
            adding_ix: 0,
            batch_add_size: 5,
        }
    }

    fn add_batched_elements(&mut self) {
        let ceil = (self.keys.len() - 1).min(self.adding_ix + self.batch_add_size);
        while self.adding_ix < ceil {
            let entry: Entry<Key, Value> = Entry::new(self.keys[self.adding_ix], 0);
            match self.cur_segment.try_add_entry(entry) {
                Ok(_) => {
                    // Nothing to do, move on to next
                    self.adding_ix += 1;
                }
                Err(_) => {
                    // Export model and clear
                    let start_ix = self.adding_ix - self.cur_segment.num_entries;
                    self.models.push((
                        self.cur_segment.to_linear_model(),
                        start_ix,
                        self.cur_segment.num_entries,
                    ));
                    self.cur_segment = SimplePGMSegmentator::new();
                }
            }
        }
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

                    let model_lines: Vec<Line> = self
                        .models
                        .iter()
                        .map(|(model, start_ix, size)| {
                            let first_key: Key = *model.borrow();
                            let first_key = first_key as f64;
                            let first_rank = start_ix.clone() as f64;
                            let end_key = self.keys[start_ix + size] as f64;
                            let end_rank = first_rank + (end_key - first_key) * model.slope;
                            return Line::new(PlotPoints::new(vec![
                                [first_key, first_rank],
                                [self.keys[start_ix + size.clone()] as f64, end_rank],
                            ]))
                            .width(8.0);
                        })
                        .collect();
                    for line in model_lines {
                        plot_ui.line(line);
                    }

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
                });

            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.num_entries));
                if ui.button("Number of Entries").clicked() {
                    self.keys.clear();
                    let range = Uniform::from((Key::MIN)..(Key::MAX));
                    let mut random_values: Vec<i32> =
                        rand::thread_rng().sample_iter(&range).take(self.num_entries).collect();
                    random_values.sort();
                    random_values.dedup();
                    self.keys = random_values;
                    self.adding_ix = 0;
                    self.cur_segment = SimplePGMSegmentator::new();
                }
            });

            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.batch_add_size));
                if ui.button("Add Entries").clicked() {
                    self.add_batched_elements();
                }
            });

            if ui.button("Reset").clicked() {
                self.keys.clear();
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
