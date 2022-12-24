#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui_winit::winit::event_loop::EventLoopBuilder;

use escher::ui;
use escher_hierarchy::Hierarchy;


fn main() {
  env_logger::init();
  let event_loop = EventLoopBuilder::<ui::EscherEvent>::with_user_event().build();

  let mut ui_hierarchy = ui::UIHierarchy::new_escher_ui(
    &event_loop,
    ui::constants::ZOOM_100,
  );

  event_loop.run(move |event, window_target, control_flow|
    ui_hierarchy.run(None, ui::FullUIInput { event, window_target, control_flow })
      .and(Ok(())).unwrap()
  );
}



