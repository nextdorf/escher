use std::thread::{self, JoinHandle};
use std::sync::mpsc;
use eframe::{egui};

pub struct LicenseDialog { handle: JoinHandle<()> }

impl eframe::App for LicenseDialog {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.label(include_str!("../../LICENSE"));
    });
  }
}

impl LicenseDialog {
  pub fn run_concurrently() {
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
      let handle = rx.recv().unwrap();
      let dialog = LicenseDialog { handle };
      dialog.run_concurrently_inner();
    });
    tx.send(handle).unwrap();
  }

  fn run_concurrently_inner(self) {
    let options = eframe::NativeOptions { 
      renderer: eframe::Renderer::Wgpu,
      ..eframe::NativeOptions::default()
    };
    eframe::run_native(
      "License",
      options,
      Box::new(|_cc| Box::new(self)),
    );
    println!("Done.")
  }
}
