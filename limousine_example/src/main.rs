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
    size0: Vec<usize>,
    size1: Vec<usize>,
    size2: Vec<usize>,
    size3: Vec<usize>,
}

impl HybridBenchmark {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            index: TestIndex::empty(),
            num_trials: 1000,
            num_inserts: 1000,
            size0: Vec::new(),
            size1: Vec::new(),
            size2: Vec::new(),
            size3: Vec::new(),
        }
    }
}

impl eframe::App for HybridBenchmark {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Empty").clicked() {
                self.index = TestIndex::empty();

                self.size0.clear();
                self.size1.clear();
                self.size2.clear();
                self.size3.clear();
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

                    self.size0.clear();
                    self.size1.clear();
                    self.size2.clear();
                    self.size3.clear();
                }
            });

            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut self.num_trials));
                if ui.button("Insert Random").clicked() {
                    for (key, value) in StdRng::from_entropy()
                        .sample_iter(Uniform::new(0, 1_000_000_000))
                        .take(self.num_trials)
                        .sorted()
                        .unique()
                        .map(|x| (x, Default::default()))
                    {
                        self.index.insert(key, value);
                        self.size0.push(self.index.component0.size());
                        self.size1.push(self.index.component1.size());
                        self.size2.push(self.index.component2.size());
                        self.size3.push(self.index.component3.size());
                    }
                }
            });

            ui.label("Index description:");
            ui.label(format!("top layer: {}", self.index.component3.size()));
            ui.label(format!("internal layer: {}", self.index.component2.size()));
            ui.label(format!("internal layer: {}", self.index.component1.size()));
            ui.label(format!("base layer: {}", self.index.component0.size()));

            let size0: egui::plot::PlotPoints = self
                .size0
                .iter()
                .enumerate()
                .map(|(i, x)| [*x as f64, i as f64])
                .collect();

            let size1: egui::plot::PlotPoints = self
                .size1
                .iter()
                .enumerate()
                .map(|(i, x)| [*x as f64, i as f64])
                .collect();

            let size2: egui::plot::PlotPoints = self
                .size2
                .iter()
                .enumerate()
                .map(|(i, x)| [*x as f64, i as f64])
                .collect();

            let size3: egui::plot::PlotPoints = self
                .size2
                .iter()
                .enumerate()
                .map(|(i, x)| [*x as f64, i as f64])
                .collect();

            let line0 = egui::plot::Line::new(size0);
            let line1 = egui::plot::Line::new(size1);
            let line2 = egui::plot::Line::new(size2);
            let line3 = egui::plot::Line::new(size3);

            egui::plot::Plot::new("my_plot")
                .view_aspect(2.0)
                .show(ui, |plot_ui| {
                    plot_ui.line(line0);
                    plot_ui.line(line1);
                    plot_ui.line(line2);
                    plot_ui.line(line3);
                });
        });
    }
}
