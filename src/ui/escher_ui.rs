use egui;

pub struct EscherUI {
  pub ctx: egui::Context,
}

impl EscherUI {
  pub fn new() -> Self { 
    Self { ctx: egui::Context::default() }
  }
  pub fn init(&mut self) {}
  pub fn run(&mut self) {
    loop {
      let raw_in = egui::RawInput::default();
      let full_out = self.ctx.run(raw_in, |ui| {
        egui::CentralPanel::default().show(&self.ctx, |ui| {
          ui.label("Hello, World");
        });
      });
    }
  }
}


