use epaint::TextureHandle;
use egui_winit::{
  egui,
  winit::{
    self,
    event_loop::{
      EventLoopProxy,
      EventLoopWindowTarget,
    },
    window,
  }
};
use crate::wgpustate::WgpuState;

use super::{EscherEvent, UIState, UIType, UI};

use std::time;


pub struct MainWindow {
  pub img_hnd: Vec<TextureHandle>,
  test_var: usize,
  pub license_status: bool,
  render_state: WgpuState,
  pub(super) egui_winit_state: egui_winit::State,
}


pub enum MainWindowDrawRes {
  InvaldRenderFrame,
  NoRedrawScheduled,
  RedrawNextFrame,
  RedrawScheduled(time::Duration)
}

impl MainWindow {
  pub fn redraw(&mut self, ctx: &egui::Context, window: &window::Window, state: &UIState) -> MainWindowDrawRes {

    self.render_state.update_window_size_bind_group(false);
    let current_frame = match self.render_state.get_current_frame() {
      Ok(frame) => frame,
      Err(_) => return MainWindowDrawRes::InvaldRenderFrame
    };

    let raw_input = self.egui_winit_state.take_egui_input(window);
    let full_output = ctx.run(raw_input, |ctx| self.ui(ctx, state));
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
      MainWindowDrawRes::RedrawNextFrame
    } else if time_until_repaint == time::Duration::MAX {
      MainWindowDrawRes::NoRedrawScheduled
    } else {
      MainWindowDrawRes::RedrawScheduled(time_until_repaint)
    }
  }

  pub fn resize(&mut self, width: Option<u32>, height: Option<u32>, scale: Option<f32>) {
    self.render_state.resize(width, height, scale, &mut self.egui_winit_state)
  }

  fn ui(&mut self, ctx: &egui::Context, state: &UIState) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui|
      self.ui_menu_bar(ui, &state.event_loop_proxy)
    );
    
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.label("Top text");
      ui.separator();
      ui.label("Middle text");
      ui.separator();
      if ui.button(self.test_var.to_string()).clicked() {
        self.test_var += 1;
      }
      ui.separator();
      ui.horizontal(|ui| {
        for tex in self.img_hnd.iter() {
          ui.image(tex.id(), tex.size_vec2());
        }
      });
      ui.separator();
      ui.label("Bottom text");
    });

    self.show_dialogs(ctx);
  }

  pub fn new(window_target: &EventLoopWindowTarget<EscherEvent>, scale_factor: f32) -> UI {
    let mut res = UI::new(
      window::WindowBuilder::new()
        .with_decorations(false)
        .with_resizable(true)
        .with_transparent(true)
        .with_title("escher")
        .with_inner_size(winit::dpi::PhysicalSize {
          width: 45*16,
          height: 45*9,
        }),
      None,
      window_target,
      scale_factor
    );

    let mut egui_winit_state = egui_winit::State::new(window_target);
    egui_winit_state.set_pixels_per_point(scale_factor);
    let render_state = WgpuState::new(&res.window, scale_factor).unwrap();

    let img_hnd = vec![
      res.ctx.load_texture("uv_texture",
        (|| {
          let size = [256, 256];
          let mut rgba = Vec::with_capacity(size[0]*size[1]*4);
          for j in 0..size[1] {
            for i in 0..size[0] {
              let r = ((i as f32) / ((size[0]-1) as f32) * 255.).round() as _;
              let g = ((j as f32) / ((size[1]-1) as f32) * 255.).round() as _;
              rgba.push(r);
              rgba.push(g);
              rgba.push(0);
              rgba.push(255);
            }
          }
          
          egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_slice())
        })(),
        egui::TextureOptions::default()),
        res.ctx.load_texture("sample_texture",
        egui::ColorImage::example(),
        egui::TextureOptions::default()),
    ];

    res.ui_impl = Some(UIType::Main(Box::new(
      Self {
        img_hnd,
        test_var: 0,
        license_status: false,
        render_state,
        egui_winit_state,
      }
    )));
    res
  }

  pub fn ui_menu_bar(&mut self, ui: &mut egui::Ui, event_proxy: &EventLoopProxy<EscherEvent>) {
    egui::menu::bar(ui, |ui| {
      ui.menu_button("File", |ui| {
        ui.separator();
        if ui.button("Exit").clicked() {
          event_proxy.send_event(EscherEvent::Exit(0)).unwrap();
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


  // const FPS_EMA_COEFF: f64 = 1./15.;
  // fn calc_fps(&self) -> (f64, f64, time::Instant){
  //   //1/fps = EMA of dtime
  //   let time0 = time::Instant::now();
  //   let dtime = (time0 - self.last_frame_time).as_secs_f64();
  //   (self.fps / (1. + Self::FPS_EMA_COEFF*(self.fps * dtime - 1.)), 1./dtime, time0)
  // }

  // fn update_fps(&mut self) {
  //   let (fps_measured, fps_projected);
  //   (fps_measured, fps_projected, self.last_frame_time) = self.calc_fps();
  //   const SIMILARITY_RANGE: f64 = 2.;
  //   let fps_ratio = fps_measured / fps_projected;
  //   let fps_ratio_in_range = 1./SIMILARITY_RANGE <= fps_ratio && fps_ratio <= SIMILARITY_RANGE;
  //   self.fps = if fps_ratio_in_range {fps_measured} else {fps_projected};
  // }

  // pub fn get_fps(&self) -> f64 {
  //   let (fps_measured, fps_projected, _) = self.calc_fps();
  //   let fps_measured = fps_measured * (1.-Self::FPS_EMA_COEFF); //Why do I need this??
  //   const SIMILARITY_RANGE: f64 = 1.035;
  //   let fps_ratio = fps_measured / fps_projected;
  //   let fps_ratio_in_range = 1./SIMILARITY_RANGE <= fps_ratio && fps_ratio <= SIMILARITY_RANGE;
  //   if fps_ratio_in_range {fps_measured} else {fps_measured.min(fps_projected)}
  // }

  // pub fn handle_events(&mut self, event: Event<EscherEvent>, _window_target: &EventLoopWindowTarget<EscherEvent>,
  //   control_flow: &mut ControlFlow, window: &window::Window, win_state: &mut State, ctx: &egui::Context, render_state: &mut WgpuState)
  // {
  //   if let Event::WindowEvent { event: window_event, .. } = event {
  //     util::update_event_modifier(&mut self.modifier, window_event);
  //   }
  //   let windows = self.owning_ui.collect_all_children();
  //   let q = windows.get(&self.get_owning_ui().get_native_window_id()).unwrap();
  //   let qq = *q;
    
  //   q.ctrl_modifier();
  // }
}

