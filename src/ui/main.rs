use std::{time, collections::HashMap, rc::{Weak, Rc}};
use epaint::TextureHandle;
use egui_winit::{
  egui,
  winit::{
    event::{Event, WindowEvent},
    event_loop::{
      EventLoopProxy,
      EventLoopWindowTarget,
      ControlFlow
    },
    window,
  }
};
use crate::wgpustate::WgpuState;

use super::{EscherEvent, PartialEscherWindow, UI, EscherWindow, util, UIHierarchy, EscherHierarchy};

pub struct MainEscherWindow {
  pub(super) owning_ui: Weak<UI>,
  pub img_hnd: Vec<TextureHandle>,
  test_var: usize,
  pub license_status: bool,
  render_state: WgpuState,
}

impl MainEscherWindow {
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

  pub fn get_owner(&self) -> Weak<UI> {
    self.owning_ui.clone()
  }
  
  pub fn new(owning_ui: Weak<UI>) -> Option<Self> {
    let (owning_ui, hierarchy) = match owning_ui.upgrade() {
      Some(owner) => match owner.hierarchy.upgrade() {
        Some(hierarchy) => (owner, hierarchy),
        None => return None
      },
      None => return None
    };
    let img_hnd = vec![
      owning_ui.ctx.load_texture("uv_texture",
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
        owning_ui.ctx.load_texture("sample_texture",
        egui::ColorImage::example(),
        egui::TextureOptions::default()),
    ];

    let surface_scale = hierarchy.egui_winit_state.pixels_per_point();
    let render_state = WgpuState::new(&owning_ui.window, surface_scale).unwrap();
    //  set_pixels_per_point(super::constants::ZOOM_100);
  
    Some(Self {
      img_hnd,
      test_var: 0,
      owning_ui: Rc::downgrade(&owning_ui),
      license_status: false,
      render_state,
    })
  }

  pub fn send_event(&self, event: EscherEvent) -> bool {
    match self.owning_ui.upgrade() {
      Some(owner) => match owner.hierarchy.upgrade() {
        Some(hierarchy) => hierarchy.try_send_event(event).is_ok(),
        None => false
      },
      None => false
    }
  }

  pub fn ui_menu_bar(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
    egui::menu::bar(ui, |ui| {
      ui.menu_button("File", |ui| {
        ui.separator();
        if ui.button("Exit").clicked() {
          self.send_event(EscherEvent::Exit(0));
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

