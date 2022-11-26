use std::collections::HashMap;

use eframe::egui_wgpu::RenderState;
use eframe::{egui, CreationContext};
use eframe::wgpu::{Adapter, Device, Queue};


pub struct EscherApp {
  render_state: RenderState,
  license_status: bool,
}

impl eframe::App for EscherApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| self.ui_menu_bar(ctx, ui));
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.label("Top Text");
      ui.separator();
      ui.label("Bottom Text");
    });

    self.show_dialogs(ctx);
  }

}
impl EscherApp {
  pub fn from_creation_context(cc: &CreationContext) -> Option<Self> {
    let render_state = cc.wgpu_render_state.as_ref()?.clone();
    Some(Self { render_state, license_status: false })
  }

  pub fn ui_menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
    egui::menu::bar(ui, |ui| {
      ui.menu_button("File", |ui| {
        ui.separator();
        if ui.button("Exit").clicked() {
          // ui.
        }
      });

      ui.menu_button("Edit", |_| {});

      ui.menu_button("Help", |ui| {
        if ui.button("License").clicked() {
          self.license_status = true;
        }
      });
    });
  }

  pub fn show_dialogs(&mut self, ctx: &egui::Context) {
    if self.license_status {
      egui::Window::new("License")
        .open(&mut self.license_status)
        .show(ctx, |ui| {
          ui.label(include_str!("../../LICENSE"));
        });
    }
  }
}
  



