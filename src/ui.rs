use std::time;

use epaint::TextureHandle;
use egui_winit::{egui, winit::{event, event_loop::{ControlFlow, EventLoopWindowTarget, self}, window::Window, dpi::PhysicalSize}, State};

use crate::wgpustate::WgpuState;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum MyEvent {
  RequestRedraw,
  Rescale(f32),
  Exit(u8),
}

pub mod constants {
  pub const ZOOM_100: f32 = 1.125;
  pub const ZOOM_PLUS: f32 = 1.125;
}


pub struct UI {
  pub img_hnd: Vec<TextureHandle>,
  test_var: usize,
  event_loop_proxy: event_loop::EventLoopProxy<MyEvent>,

  pub lctrl_modifier: bool,
  pub rctrl_modifier: bool,
  pub license_status: bool,
  fps: f64,
  last_frame_time: time::Instant,
}


impl UI {
  pub fn new(event_loop_proxy: event_loop::EventLoopProxy<MyEvent>, ctx: &egui::Context) -> Self {
    let img_hnd = vec![
      ctx.load_texture("uv_texture",
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
        egui::TextureFilter::Linear),
      ctx.load_texture("sample_texture",
        egui::ColorImage::example(),
        egui::TextureFilter::Linear),
    ];

    Self {
      img_hnd,
      test_var: 0,
      event_loop_proxy,
      lctrl_modifier: false,
      rctrl_modifier: false,
      license_status: false,
      fps: 60.,
      last_frame_time: time::Instant::now()
    }
  }

  pub fn dispatch_event(&mut self, event: MyEvent) -> bool {
    self.event_loop_proxy
      .send_event(event)
      .expect("Eventproxy expired");
    true
  }

  pub fn ui(&mut self, ctx: &egui::Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| self.ui_menu_bar(ctx, ui));
    
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

  pub fn ui_menu_bar(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
    egui::menu::bar(ui, |ui| {
      ui.menu_button("File", |ui| {
        ui.separator();
        if ui.button("Exit").clicked() {
          self.dispatch_event(MyEvent::Exit(0));
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
          ui.label(include_str!("../LICENSE"));
        });
    }
  }

  pub fn ctrl_modifier(&self) -> bool {
    self.lctrl_modifier || self.rctrl_modifier
  }

  const FPS_EMA_COEFF: f64 = 1./15.;
  fn calc_fps(&self) -> (f64, f64, time::Instant){
    //1/fps = EMA of dtime
    let time0 = time::Instant::now();
    let dtime = (time0 - self.last_frame_time).as_secs_f64();
    (self.fps / (1. + Self::FPS_EMA_COEFF*(self.fps * dtime - 1.)), 1./dtime, time0)
  }

  fn update_fps(&mut self) {
    let (fps_measured, fps_projected);
    (fps_measured, fps_projected, self.last_frame_time) = self.calc_fps();
    const SIMILARITY_RANGE: f64 = 2.;
    let fps_ratio = fps_measured / fps_projected;
    let fps_ratio_in_range = 1./SIMILARITY_RANGE <= fps_ratio && fps_ratio <= SIMILARITY_RANGE;
    self.fps = if fps_ratio_in_range {fps_measured} else {fps_projected};
  }

  pub fn get_fps(&self) -> f64 {
    let (fps_measured, fps_projected, _) = self.calc_fps();
    let fps_measured = fps_measured * (1.-Self::FPS_EMA_COEFF); //Why do I need this??
    const SIMILARITY_RANGE: f64 = 1.035;
    let fps_ratio = fps_measured / fps_projected;
    let fps_ratio_in_range = 1./SIMILARITY_RANGE <= fps_ratio && fps_ratio <= SIMILARITY_RANGE;
    if fps_ratio_in_range {fps_measured} else {fps_measured.min(fps_projected)}
  }
}


pub fn wait_at_most_until(control_flow: &mut ControlFlow, t: time::Instant) {
  if let Some(new_control_flow) = match *control_flow {
    ControlFlow::Wait | ControlFlow::Poll => Some(ControlFlow::WaitUntil(t)),
    ControlFlow::WaitUntil(t2) if t2 > t => Some(ControlFlow::WaitUntil(t)),
    _ => None
  }{
    *control_flow = new_control_flow;
  }
}

static mut START_TIME_STORE: Option<time::Instant> = None;
pub fn start_time() -> &'static time::Instant { unsafe {START_TIME_STORE.as_ref().unwrap()} }

pub fn handle_events(event: event::Event<MyEvent>, _window_target: &EventLoopWindowTarget<MyEvent>, control_flow: &mut ControlFlow,
  window: &Window, win_state: &mut State, ctx: &egui::Context, render_state: &mut WgpuState, ui_state: &mut UI)
{
  // let event_str = format!("{:?}", event);

  match event {
    event::Event::WindowEvent { window_id, event } if window_id==window.id() => {
      if !win_state.on_event(&ctx, &event) {
        match event {
          event::WindowEvent::Resized(PhysicalSize { width, height}) =>
            render_state.resize(Some(width), Some(height), None, win_state),
          event::WindowEvent::CloseRequested | event::WindowEvent::Destroyed => 
            *control_flow = ControlFlow::Exit,
          event::WindowEvent::KeyboardInput { input, .. } => {
            if let event::KeyboardInput { virtual_keycode: Some(event::VirtualKeyCode::LControl), state,.. } = input {
              ui_state.lctrl_modifier = state == event::ElementState::Pressed;
            }
            else if let event::KeyboardInput { virtual_keycode: Some(event::VirtualKeyCode::RControl), state,.. } = input {
              ui_state.rctrl_modifier = state == event::ElementState::Pressed;
            }
            else if let event::KeyboardInput { virtual_keycode: Some(keycode), state: event::ElementState::Pressed,.. } = input {
              match keycode {
                event::VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                event::VirtualKeyCode::F10 => println!("FPS: {}", ui_state.get_fps()),
                _ => {}
              }
            }
            if ui_state.ctrl_modifier() {
              if let event::KeyboardInput { virtual_keycode: Some(keycode), state: event::ElementState::Pressed,.. } = input {
                match keycode {
                  event::VirtualKeyCode::Plus | event::VirtualKeyCode::NumpadAdd => {
                    let scale_factor = win_state.pixels_per_point() * constants::ZOOM_PLUS;
                    render_state.resize(None, None, Some(scale_factor), win_state);
                  },
                  event::VirtualKeyCode::Minus | event::VirtualKeyCode::NumpadSubtract => {
                    let scale_factor = win_state.pixels_per_point() / constants::ZOOM_PLUS;
                    render_state.resize(None, None, Some(scale_factor), win_state);
                  },
                  event::VirtualKeyCode::Key0 | event::VirtualKeyCode::Numpad0 =>
                    render_state.resize(None, None, Some(constants::ZOOM_100), win_state),
                  _ => {}
                }
              }
            }
          },
          event::WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => {
            render_state.resize(Some(new_inner_size.width), Some(new_inner_size.height), Some(scale_factor as _), win_state);
          },
          _ => {}
        }
      }
      window.request_redraw();
    },
    event::Event::DeviceEvent { .. } => {
      //TODO
      window.request_redraw();
    },
    event::Event::RedrawRequested(window_id) if window_id != window.id() => { },
    event::Event::RedrawRequested(..) => {
      render_state.update_window_size_bind_group(false);
  
      let _did_render = render_state.redraw(|| {
        let raw_input = win_state.take_egui_input(&window);
        let full_output = ctx.run(raw_input, |ctx| ui_state.ui(ctx));
        let time_until_repaint = full_output.repaint_after;
        if time_until_repaint.is_zero() {
          *control_flow = ControlFlow::Poll;
        } else if time_until_repaint == time::Duration::MAX {
          *control_flow = ControlFlow::Wait;
        } else {
          // wait_at_most_until(control_flow, time::Instant::now() + time_until_repaint);
          *control_flow = ControlFlow::WaitUntil(time::Instant::now() + time_until_repaint);
        }

        win_state.handle_platform_output(&window, &ctx, full_output.platform_output);
        let paint_jobs = ctx.tessellate(full_output.shapes);
        let texture_delta = full_output.textures_delta;
        
        (texture_delta, paint_jobs)
      }).and(Some(true))
      .unwrap_or_else(|| {
        eprintln!("Incomplete rendering");
        false
      });

      ui_state.update_fps();
    },
    event::Event::UserEvent(MyEvent::Exit(err_code)) =>
      *control_flow = if err_code==0 {ControlFlow::Exit} else {ControlFlow::ExitWithCode(err_code as _)},
    event::Event::MainEventsCleared => {
    },
    event::Event::UserEvent(MyEvent::RequestRedraw) => {
      *control_flow = ControlFlow::Poll;
      window.request_redraw();
    },
    event::Event::NewEvents(start_cause) => match start_cause {
      event::StartCause::Init  => unsafe {START_TIME_STORE.get_or_insert(time::Instant::now());},
      event::StartCause::ResumeTimeReached { .. } => window.request_redraw(),
      event::StartCause::Poll => window.request_redraw(),
      event::StartCause::WaitCancelled { .. } => {}
    },
    _ => {}
  }

  // if *control_flow != ControlFlow::Wait {
  //   eprintln!("[{:?}] {:?}: {:?}", ((time::Instant::now() - *start_time()).as_secs_f32()*60.) as usize, *control_flow, event_str)
  // }
}

