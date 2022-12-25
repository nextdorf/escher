#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui_winit::winit::event_loop::EventLoopBuilder;

use escher::{ui::{self, UIType}, assets::DummyAsset};
use escher_hierarchy::Hierarchy;


fn main() {
  env_logger::init();
  let event_loop = EventLoopBuilder::<ui::EscherEvent>::with_user_event().build();

  let mut ui_hierarchy = ui::UIHierarchy::new_escher_ui(
    &event_loop,
    ui::constants::ZOOM_100,
  );

  let main_id = ui_hierarchy.get_toplevel_id();
  let main_ui = ui_hierarchy.access_entity(&main_id).unwrap();
  if let Some(UIType::Main(main_window)) = &mut main_ui.ui_impl {
    main_window.asset_manager.add(DummyAsset::load_default(&main_ui.ctx)).unwrap()
  }

  event_loop.run(move |event, window_target, control_flow|
    ui_hierarchy.run(None, ui::FullUIInput { event, window_target, control_flow })
      .and(Ok(())).unwrap()
  );
}



