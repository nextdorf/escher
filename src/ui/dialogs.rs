use egui_winit::{
  egui::{
    Context,
    CentralPanel
  },
  winit::{
    self,
    window,
    event_loop::{
      ControlFlow,
      EventLoopWindowTarget
    }
  }
};

use super::{simple::{SimpleWindow, WindowDrawRes}, UIState, EscherEvent, UI, UIType};

pub struct LicenseDialog {
  pub(super) inner: SimpleWindow,
}

impl LicenseDialog {
  pub fn redraw(&mut self, ctx: &Context, window: &window::Window, state: &UIState, control_flow: &mut ControlFlow) -> WindowDrawRes {
    let inner = &mut unsafe {(self as *mut Self).as_mut()}.unwrap().inner;
    inner.redraw(ctx, window, state, control_flow, |ctx, state| self.ui(ctx, state))
  }

  pub fn resize(&mut self, width: Option<u32>, height: Option<u32>, scale: Option<f32>) {
    self.inner.resize(width, height, scale)
  }


  fn ui(&mut self, ctx: &Context, _state: &UIState) {
    CentralPanel::default().show(ctx, |ui| {
      ui.label(include_str!("../../LICENSE"));
    });
  }

  pub fn new(window_target: &EventLoopWindowTarget<EscherEvent>, scale_factor: f32) -> UI {
    let (mut res, inner) = SimpleWindow::new(
      window::WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(true)
        .with_title("License")
        .with_inner_size(winit::dpi::PhysicalSize {
          width: 512,
          height: 512,
        }),
      window_target,
      scale_factor
    );

    res.ui_impl = Some(UIType::License(Box::new(
      Self { inner }
    )));
    res
  }


}
