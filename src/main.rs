#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod ui;
use std::path;

use eframe::egui;
use escher_video::ffi::{VideoStream, PartialVideoStream, AVPixelFormat, SWS_Scaling, VideoStreamErr, Seek};

fn main() {
  let options = eframe::NativeOptions { 
    renderer: eframe::Renderer::Wgpu,
    ..eframe::NativeOptions::default()
  };
  eframe::run_native(
    "escher",
    options,
    Box::new(|cc| Box::new(ui::EscherApp::from_creation_context(cc)
      .expect("Wgpu could not supply the render backend for escher"))),
  );
}




