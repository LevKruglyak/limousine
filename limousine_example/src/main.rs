#![allow(unused)]

use average::*;
use egui::DragValue;
use itertools::Itertools;
use itertools::Unique;
use limousine_core::classical::*;
use limousine_core::component::*;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use rand_distr::Uniform;
use std::time::Instant;

type Key = i32;
type Value = [u8; 32];

type Component0 = BTreeBaseComponent<Key, Value, 16>;
type Component1 = BTreeInternalComponent<Key, Component0, 16>;
type Component2 = BTreeInternalComponent<Key, Component1, 16>;
type Component3 = BTreeTopComponent<Key, Component2>;

pub struct TestIndex {
    component3: Component3,
    component2: Component2,
    component1: Component1,
    component0: Component0,
}

impl TestIndex {
    fn search(&mut self, key: &Key) -> Option<&Value> {
        let search3 = self.component3.search(&key);
        let search2 = self.component2.search(search3, &key);
        let search1 = self.component1.search(search2, &key);
        let search0 = self.component0.get(search1, &key);

        search0
    }

    fn insert(&mut self, key: Key, value: Value) -> Option<Value> {
        // Search stage
        let search3 = self.component3.search(&key);
        let search2 = self.component2.search(search3, &key);
        let search1 = self.component1.search(search2, &key);
        let search0 = self.component0.get(search1, &key);

        // If value already exists, return
        let result = if let Some(value) = search0 {
            Some(value)
        } else {
            None
        };

        // Insert stage
        let propogate0 = self.component0.insert(search1, key, value)?;
        let propogate1 = self.component1.insert(search2, propogate0)?;
        let propogate2 = self.component2.insert(search3, propogate1)?;
        self.component3.insert(propogate2);

        None
    }

    fn empty() -> Self {
        let component0 = Component0::empty();
        let component1 = Component1::build(&component0);
        let component2 = Component2::build(&component1);
        let component3 = Component3::build(&component2);

        Self {
            component3,
            component2,
            component1,
            component0,
        }
    }

    fn build(iter: impl Iterator<Item = (Key, Value)>) -> Self {
        let component0 = Component0::build(iter);
        let component1 = Component1::build(&component0);
        let component2 = Component2::build(&component1);
        let component3 = Component3::build(&component2);

        Self {
            component3,
            component2,
            component1,
            component0,
        }
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Box::new(HybridBenchmark::new(cc))),
    );
}

struct HybridBenchmark {
    index: TestIndex,
    num_trials: usize,
    num_inserts: usize,
    size: Vec<usize>,
}

impl HybridBenchmark {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            index: TestIndex::empty(),
            num_trials: 1000,
            num_inserts: 1000,
            size: Vec::new(),
        }
    }
}

impl eframe::App for HybridBenchmark {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Empty").clicked() {
                self.index = TestIndex::empty();

                self.size.clear();
            }

            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut self.num_trials));
                if ui.button("Bulk Random").clicked() {
                    self.index = TestIndex::build(
                        StdRng::from_entropy()
                            .sample_iter(Uniform::new(0, 1_000_000_000))
                            .take(self.num_trials)
                            .sorted()
                            .unique()
                            .map(|x| (x, Default::default())),
                    );

                    self.size.clear();
                }
            });

            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut self.num_trials));
                if ui.button("Insert Random").clicked() {
                    for (key, value) in StdRng::from_entropy()
                        .sample_iter(Uniform::new(0, 1_000_000_000))
                        .take(self.num_inserts)
                        .sorted()
                        .unique()
                        .map(|x| (x, Default::default()))
                    {
                        self.index.insert(key, value);

                        let total_size = self.index.component2.memory_size()
                            + self.index.component1.memory_size()
                            + self.index.component0.memory_size();

                        self.size.push(total_size);
                    }
                }
            });

            ui.label("Index Component Lengths:");
            ui.label(format!("top layer: {}", self.index.component3.len()));
            ui.label(format!("internal layer: {}", self.index.component2.len()));
            ui.label(format!("internal layer: {}", self.index.component1.len()));
            ui.label(format!("base layer: {}", self.index.component0.len()));

            let size: egui::plot::PlotPoints = self
                .size
                .iter()
                .enumerate()
                .map(|(i, x)| [i as f64, *x as f64])
                .collect();

            let line = egui::plot::Line::new(size);

            egui::plot::Plot::new("my_plot")
                .view_aspect(2.0)
                .show(ui, |plot_ui| {
                    plot_ui.line(line);
                });
        });
    }
}
