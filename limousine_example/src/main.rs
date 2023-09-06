#![allow(unused)]

use average::*;
use egui::DragValue;
use itertools::Itertools;
use itertools::Unique;
use limousine_core::classical::*;
use limousine_core::component::*;
use limousine_core::learned::*;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use rand_distr::Uniform;
use std::time::Instant;

use limousine_engine::prelude::*;

create_hybrid_index! {
    name: MyHybridIndex,
    layout: [
        std_btree(),
        btree(fanout = 8),
        btree(fanout = 8),
        btree(fanout = 8),
        btree(fanout = 16),
        btree(fanout = 18),
    ]
}

fn main() {
    let index = MyHybridIndex::build((0..1_000).map(|x| (x, x * x)));

    println!("{:?}", index.search(&0));
}

//     fn insert(&mut self, key: Key, value: Value) -> Option<Value> {
//         // Search stage
//         let search3 = self.component3.search(&self.component2, &key);
//         let search2 = self.component2.search(&self.component1, search3, &key);
//         let search1 = self.component1.search(&self.component0, search2, &key);
//         let search0 = self.component0.get(search1, &key);
//
//         // If value already exists, return
//         let result = if let Some(value) = search0 {
//             Some(value)
//         } else {
//             None
//         };
//
//         // Insert stage
//         let propogate0 = self.component0.insert(search1, key, value)?;
//         let propogate1 = self
//             .component1
//             .insert(&self.component0, search2, propogate0)?;
//         let propogate2 = self
//             .component2
//             .insert(&self.component1, search3, propogate1)?;
//         self.component3.insert(&self.component2, propogate2);
//
//         None
//     }
// fn main() {
//     let native_options = eframe::NativeOptions::default();
//     eframe::run_native(
//         "My egui App",
//         native_options,
//         Box::new(|cc| Box::new(HybridBenchmark::new(cc))),
//     );
// }
//
// struct HybridBenchmark {
//     index: TestIndex,
//     num_trials: usize,
//     num_inserts: usize,
//     size: Vec<usize>,
// }
//
// impl HybridBenchmark {
//     fn new(cc: &eframe::CreationContext<'_>) -> Self {
//         Self {
//             index: TestIndex::empty(),
//             num_trials: 1000,
//             num_inserts: 1000,
//             size: Vec::new(),
//         }
//     }
// }
//
// impl eframe::App for HybridBenchmark {
//     fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             if ui.button("Empty").clicked() {
//                 self.index = TestIndex::empty();
//
//                 self.size.clear();
//             }
//
//             ui.horizontal(|ui| {
//                 ui.add(DragValue::new(&mut self.num_trials));
//                 if ui.button("Bulk Random").clicked() {
//                     self.index = TestIndex::build(
//                         StdRng::from_entropy()
//                             .sample_iter(Uniform::new(Key::MIN, Key::MAX))
//                             .take(self.num_trials)
//                             .sorted()
//                             .unique()
//                             .map(|x| (x, Default::default())),
//                     );
//
//                     self.size.clear();
//                 }
//             });
//
//             ui.horizontal(|ui| {
//                 ui.add(DragValue::new(&mut self.num_inserts));
//                 if ui.button("Insert Random").clicked() {
//                     for (key, value) in StdRng::from_entropy()
//                         .sample_iter(Uniform::new(Key::MIN, Key::MAX))
//                         .take(self.num_inserts)
//                         .sorted()
//                         .unique()
//                         .map(|x| (x, Default::default()))
//                     {
//                         self.index.insert(key, value);
//
//                         let total_size = self.index.component2.memory_size()
//                             + self.index.component1.memory_size()
//                             + self.index.component0.memory_size();
//
//                         self.size.push(total_size);
//                     }
//                 }
//             });
//
//             ui.label("Index Component Lengths:");
//             ui.label(format!("top layer: {}", self.index.component3.len()));
//             ui.label(format!("internal layer: {}", self.index.component2.len()));
//             ui.label(format!("internal layer: {}", self.index.component1.len()));
//             ui.label(format!("base layer: {}", self.index.component0.len()));
//
//             let mut lines = Vec::new();
//
//             ui.collapsing("Models", |ui| {
//                 for node in self.index.component1.full_range() {
//                     let node = self.index.component1.deref(node.1);
//                     ui.label(format!("model: {:?}", node));
//                 }
//             });
//
//             let mut iter = self.index.component1.full_range();
//             // iter.next();
//             let mut previous: Option<&<Component1 as NodeLayer<Key>>::Node> = None;
//             for node in iter {
//                 let node = self.index.component1.deref(node.1);
//
//                 if let Some(next_node) = previous {
//                     let model: egui::plot::PlotPoints = [
//                         [node.intercept as f64, node.key as f64],
//                         [next_node.intercept as f64, next_node.key as f64],
//                     ]
//                     .iter()
//                     .copied()
//                     .collect();
//
//                     lines.push(egui::plot::Line::new(model).width(2.0));
//                 }
//
//                 previous = Some(node);
//             }
//
//             egui::plot::Plot::new("my_plot")
//                 .view_aspect(2.0)
//                 .auto_bounds_x()
//                 .auto_bounds_y()
//                 .allow_drag(false)
//                 .allow_zoom(false)
//                 .show(ui, |plot_ui| {
//                     for line in lines {
//                         plot_ui.line(line);
//                     }
//                 });
//         });
//     }
// }
