//! Visualize the PGM segmentation algorithm for sanity bug catching

use egui::plot::*;
use kdam::{tqdm, BarExt};
use limousine_core::{
    learned::pgm::gapped_pgm::{GappedKey, GappedPGM, GappedValue},
    Entry,
};
use rand::{distributions::Uniform, Rng};

struct AppState<V: GappedValue, const INT_EPS: usize, const LEAF_EPS: usize, const LEAF_BUFSIZE: usize> {
    model: GappedPGM<V, INT_EPS, LEAF_EPS, LEAF_BUFSIZE>,
}

impl<V: GappedValue, const INT_EPS: usize, const LEAF_EPS: usize, const LEAF_BUFSIZE: usize>
    AppState<V, INT_EPS, LEAF_EPS, LEAF_BUFSIZE>
{
    /// Create an empty app state
    fn new(ctx: &eframe::CreationContext<'_>, model: GappedPGM<V, INT_EPS, LEAF_EPS, LEAF_BUFSIZE>) -> Self {
        Self { model }
    }
}

impl<V: GappedValue, const INT_EPS: usize, const LEAF_EPS: usize, const BUFSIZE: usize> eframe::App
    for AppState<V, INT_EPS, LEAF_EPS, BUFSIZE>
{
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
                    let mut cur_ptr = self.model.root_ptr;
                    let mut height = self.model.height;
                    while let Some(ptr) = cur_ptr {
                        if height > 0 {
                            let node = self.model.get_internal_node(ptr).unwrap();
                            let ix = node.ga.next_occupied_ix(0).unwrap();
                            cur_ptr = Some(node.ga.vals[ix]);
                            height -= 1;
                        } else {
                            let node = self.model.get_leaf_node(ptr);
                            cur_ptr = None;
                        }
                    }
                    // let points = PlotPoints::new(
                    //     self.keys
                    //         .iter()
                    //         .copied()
                    //         .enumerate()
                    //         .map(|(rank, key)| [key as f64, rank as f64])
                    //         .collect(),
                    // );
                    // plot_ui.points(Points::new(points).radius(5.0).name("key-ranks"));

                    // // Plot the models
                    // assert!(self.models.len() == self.model_ranks.len());
                    // let mut model_lines: Vec<Line> = vec![];
                    // for ix in 0..(self.models.len()) {
                    //     let model = self.models[ix];
                    //     let model_rank = self.model_ranks[ix];
                    //     let first_key = model.key;
                    //     let end_key = self.keys[(self.keys.len() - 1).min(model_rank + model.size)];
                    //     let end_rank = model_rank as f64 + ((end_key.saturating_sub(first_key)) as f64 * model.slope);
                    //     let line = Line::new(PlotPoints::new(vec![
                    //         [first_key as f64, model_rank as f64],
                    //         [end_key as f64, end_rank],
                    //     ]))
                    //     .width(8.0);
                    //     model_lines.push(line);
                    // }
                    // for line in model_lines {
                    //     plot_ui.line(line);
                    // }

                    // // Plot the current segment once it has enough entries to be a little stable (not huge slopes)
                    // if self.cur_segment.num_entries > 3 {
                    //     let first_key = self.cur_segment.first_k.unwrap() as f64;
                    //     let cur_key = self.keys[self.adding_ix] as f64;
                    //     let first_rank = (self.adding_ix - self.cur_segment.num_entries) as f64;
                    //     let max_rank = first_rank + (cur_key - first_key) * self.cur_segment.max_slope;
                    //     let min_rank = first_rank + (cur_key - first_key) * self.cur_segment.min_slope;
                    //     let segment_max_line =
                    //         Line::new(PlotPoints::new(vec![[first_key, first_rank], [cur_key, max_rank]])).width(2.0);
                    //     plot_ui.line(segment_max_line);
                    //     let segment_min_line =
                    //         Line::new(PlotPoints::new(vec![[first_key, first_rank], [cur_key, min_rank]])).width(2.0);
                    //     plot_ui.line(segment_min_line);
                    // }

                    // // Plot the point we're querying and the bounds
                    // if self.query_point.is_some() && self.get_query_bounds().is_some() {
                    //     let approx = self.get_query_bounds().unwrap();
                    //     let points = PlotPoints::new(vec![
                    //         [self.query_point.unwrap().key as f64, approx.lo as f64],
                    //         [
                    //             self.query_point.unwrap().key as f64,
                    //             self.query_point.unwrap().value as f64,
                    //         ],
                    //         [self.query_point.unwrap().key as f64, approx.hi as f64],
                    //     ]);
                    //     plot_ui.points(Points::new(points).radius(9.0).name("query"));
                    // }
                });
        });
    }
}

pub fn viz_model<V: GappedValue, const INT_EPS: usize, const LEAF_EPS: usize, const LEAF_BUFSIZE: usize>(
    model: GappedPGM<V, INT_EPS, LEAF_EPS, LEAF_BUFSIZE>,
) {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Piecewise Geometric Model Segmentation Algorithm",
        native_options,
        Box::new(|cc| Box::new(AppState::new(cc, model))),
    )
    .ok();
}

fn generate_random_entries(size: usize) -> Vec<Entry<i32, i32>> {
    let range = Uniform::from((GappedKey::MIN)..(GappedKey::MAX));
    let mut random_values: Vec<i32> = rand::thread_rng().sample_iter(&range).take(size).collect();
    random_values.sort();
    random_values.dedup();
    let entries: Vec<Entry<GappedKey, i32>> = random_values
        .into_iter()
        .enumerate()
        .map(|(ix, key)| Entry::new(key, ix as i32))
        .collect();
    entries
}

fn main() {
    let entries = generate_random_entries(100);
    let gapped_pgm: GappedPGM<i32, 4, 4, 4> = GappedPGM::build_from_slice(&entries);
    viz_model(gapped_pgm);
    // let mut pb = tqdm!(total = entries.len());
    // for entry in entries {
    //     let val = gapped_pgm.search(entry.key);
    //     assert!(*val.unwrap() == entry.value);
    //     pb.update(1).ok();
    // }
}
