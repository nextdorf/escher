use egui_winit::{
  egui,
  winit::{
    event_loop::{
      EventLoopWindowTarget, ControlFlow,
    },
    window,
  }
};
use crate::wgpustate::WgpuState;

use super::{EscherEvent, UIState, UI};

use std::time;


pub struct SimpleWindow {
  render_state: WgpuState,
  pub(super) egui_winit_state: egui_winit::State,
}


pub enum WindowDrawRes {
  InvaldRenderFrame,
  NoRedrawScheduled(bool),
  RedrawNextFrame(bool),
  RedrawScheduled(time::Duration)
}

impl SimpleWindow {
  pub fn redraw(&mut self, ctx: &egui::Context, window: &window::Window, state: &UIState, control_flow: &mut ControlFlow, run_ui: impl FnOnce(&egui::Context, &UIState)) -> WindowDrawRes {

    self.render_state.update_window_size_bind_group(false);
    let current_frame = match self.render_state.get_current_frame() {
      Ok(frame) => frame,
      Err(_) => return WindowDrawRes::InvaldRenderFrame
    };

    let raw_input = self.egui_winit_state.take_egui_input(window);
    let full_output = ctx.run(raw_input, |ctx| run_ui(ctx, state));
    let time_until_repaint = full_output.repaint_after;

    //TODO: construct Result and tell it when to repaint using time_until_repaint

    self.egui_winit_state.handle_platform_output(window, ctx, full_output.platform_output);
    let paint_jobs = ctx.tessellate(full_output.shapes);
    let texture_delta = full_output.textures_delta;

    let _did_render = match self.render_state.redraw(current_frame, texture_delta, paint_jobs) {
      Some(()) => true,
      None => {eprintln!("Incomplete rendering!"); false}
    };

    if time_until_repaint.is_zero() {
      if *control_flow == ControlFlow::Poll {
        WindowDrawRes::RedrawNextFrame(false)
      } else {
        *control_flow = ControlFlow::Poll;
        WindowDrawRes::RedrawNextFrame(true)
      }
    } else if time_until_repaint == time::Duration::MAX {
      if *control_flow == ControlFlow::Wait {
        WindowDrawRes::NoRedrawScheduled(false)
      } else {
        *control_flow = ControlFlow::Wait;
        WindowDrawRes::NoRedrawScheduled(true)
      }
    } else {
      WindowDrawRes::RedrawScheduled(time_until_repaint)
    }
  }

  pub fn resize(&mut self, width: Option<u32>, height: Option<u32>, scale: Option<f32>) {
    self.render_state.resize(width, height, scale, &mut self.egui_winit_state)
  }

  pub fn new(window_builder: window::WindowBuilder, window_target: &EventLoopWindowTarget<EscherEvent>, scale_factor: f32) -> (UI, Self) {
    let empty_ui = UI::new(
      window_builder,
      None,
      window_target,
      scale_factor
    );

    let mut egui_winit_state = egui_winit::State::new(window_target);
    egui_winit_state.set_pixels_per_point(scale_factor);
    let render_state = WgpuState::new(&empty_ui.window, scale_factor).unwrap();

    (empty_ui, Self { render_state, egui_winit_state })
  }
}

