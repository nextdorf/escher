#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui_winit::{egui, winit::{self, event_loop::EventLoop, window::Window}};

use escher::{ui, util, wgpustate::WgpuState};

fn setup_egui_winit(event_loop: &EventLoop<ui::MyEvent>) -> (Window, egui_winit::State, egui::Context){
  let window = winit::window::WindowBuilder::new()
    .with_decorations(false)
    .with_resizable(true)
    .with_transparent(true)
    .with_title("not eframe")
    .with_inner_size(winit::dpi::PhysicalSize {
      width: 512,
      height: 512,
    })
    .build(event_loop)
    .unwrap();

  let win_state = egui_winit::State::new(event_loop);
  let egui_ctx = egui::Context::default();

  let mut style = (*egui_ctx.style()).clone();
  style.visuals = util::VisualsColorMap::with_rgba_to_srgba(Some(style.visuals))
    .map_state()
    .unwrap();
  egui_ctx.set_style(style);

  // let fonts = egui::FontDefinitions::default();
  // egui_ctx.set_fonts(fonts);

  (window, win_state, egui_ctx)
}

fn main() {
  env_logger::init();
  let event_loop = winit::event_loop::EventLoopBuilder::<ui::MyEvent>::with_user_event().build();

  let (window, mut win_state, egui_ctx) = setup_egui_winit(&event_loop);
  let mut ui_state = ui::UI::new(event_loop.create_proxy(), &egui_ctx);
  let mut render_state = WgpuState::new(&window, ui::constants::ZOOM_100).unwrap();
  win_state.set_pixels_per_point(ui::constants::ZOOM_100);


  event_loop.run(move |event, window_target, control_flow|
    ui::handle_events(event, window_target, control_flow, &window, &mut win_state, &egui_ctx, &mut render_state, &mut ui_state)
  );


}



