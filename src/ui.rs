pub mod event;
mod error;

use std::{time, collections::HashMap};

use epaint::TextureHandle;
use egui_winit::{egui, winit::{event_loop::{self, EventLoopClosed}, window}};

// use crate::wgpustate::WgpuState;


#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum EscherEvent {
  RequestRedrawPath{ path: Vec<window::WindowId>, redraw_children: bool },
  Rescale(f32),
  Exit(u8),
}

pub mod constants {
  pub const ZOOM_100: f32 = 1.125;
  pub const ZOOM_PLUS: f32 = 1.125;
}


pub trait EscherWindow: Sized {
  fn get_native_window(&self) -> &window::Window;

  fn get_native_window_id(&self) -> window::WindowId {
    self.get_native_window().id()
  }

  fn try_send_event(&self, event: EscherEvent) -> Result<(), EventLoopClosed<EscherEvent>>;

  fn send_event(&self, event: EscherEvent) {
    self.try_send_event(event).unwrap_or_default()
  }


  fn get_toplevel(&self) -> &Self;
  fn get_mut_toplevel(&mut self) -> &mut Self;
  // fn move_to_toplevel(self) -> Self;

  fn get_parent(&self) -> Option<&Self>;
  fn get_mut_parent(&mut self) -> Option<&mut Self>;
  // fn move_to_parent(self) -> Option<Self>;

  fn get_children(&self) -> &HashMap<window::WindowId, Self>;
  fn get_mut_children(&mut self) -> &mut HashMap<window::WindowId, Self>;

  fn reparent_child(&mut self, child_id: window::WindowId, new_parent: &mut Self) -> Result<(), error::ReparentError> {
    match self.get_mut_children().remove(&child_id) {
      Some(child) => {
        if let Some(collision) = new_parent.get_mut_children().insert(child_id, child) {
          panic!("Bug in winit: window id collision for {:?}", collision.get_native_window_id())
        } else {
          Ok(())
        }
      },
      None => Err(error::ReparentError::PathNotFound)
    }
  }

  fn reparent_child_to(&mut self, child_id: window::WindowId, path: &Vec<window::WindowId>, is_relative: bool) -> Result<(), error::ReparentError> {
    if is_relative {
      if path.is_empty() {
        if self.get_children().contains_key(&child_id) {
          Ok(())
        } else {
          Err(error::ReparentError::PathNotFound)
        }
      } else {
        let mut new_parent_ptr = self as *mut Self;
        for curr_id in path {
          let new_parent = unsafe { new_parent_ptr.as_mut().unwrap() };
          match new_parent.get_mut_children().get_mut(&curr_id) {
            Some(child) => new_parent_ptr = child,
            None => return Err(error::ReparentError::PathNotFound),
          };
        }
        self.reparent_child(child_id, unsafe { new_parent_ptr.as_mut().unwrap() })
      }
    } else {
      let toplevel = self.get_mut_toplevel() as *mut Self;
      unsafe {
        self.reparent_child(child_id, toplevel.as_mut().unwrap())?;
        toplevel.as_mut().unwrap().reparent_child_to(child_id, path, true)
      }
    }
  }


  fn ctrl_modifier(&self) -> bool {
    let toplevel = self.get_toplevel();
    if (self as *const Self) != (toplevel as *const Self) {
      panic!("Window {:?} doesn't implement ctrl_modifier", self.get_native_window_id())
    } else {
      toplevel.ctrl_modifier()
    }
  }
  fn shift_modifier(&self) -> bool {
    let toplevel = self.get_toplevel();
    if (self as *const Self) != (toplevel as *const Self) {
      panic!("Window {:?} doesn't implement shift_modifier", self.get_native_window_id())
    } else {
      toplevel.shift_modifier()
    }
  }


  fn ui(&mut self, ctx: &egui::Context);
}


pub struct UI {
  pub img_hnd: Vec<TextureHandle>,
  test_var: usize,
  event_loop_proxy: event_loop::EventLoopProxy<EscherEvent>,

  pub lctrl_modifier: bool,
  pub rctrl_modifier: bool,
  pub license_status: bool,
  fps: f64,
  last_frame_time: time::Instant,
}


impl UI {
  pub fn new(event_loop_proxy: event_loop::EventLoopProxy<EscherEvent>, ctx: &egui::Context) -> Self {
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

  pub fn dispatch_event(&self, event: EscherEvent) -> bool {
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
          self.dispatch_event(EscherEvent::Exit(0));
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

