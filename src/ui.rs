pub mod event;
pub mod main;
pub mod dialogs;
mod error;
mod simple;
mod util;

use std::{collections::HashMap, time};

use egui_winit::{
  egui, 
  winit::{
    event_loop::{
      EventLoopWindowTarget,
      ControlFlow, EventLoopProxy, EventLoop
    },
    window,
    event::{Event, WindowEvent}
  }
};

use super::hierarchy::Entity;
// use crate::wgpustate::WgpuState;


#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum EscherEvent {
  RequestRedraw{ id: UIId },
  Rescale(f32),
  Exit(u8),
  NewDialog,
}

pub mod constants {
  pub const ZOOM_100: f32 = 1.125;
  pub const ZOOM_PLUS: f32 = 1.125;
}


pub enum UIType {
  Main(Box<main::MainWindow>),
  License(Box<dialogs::LicenseDialog>),
  // Dynamic(Box<dyn Entity>),
} 

/// Generic abstraction over UIType. It handles the window system and window hierarchy. For the
/// specification see `ui_state: UIType<'inner>`
pub struct UI {
  pub ctx: egui::Context,
  pub ui_impl: Option<UIType>,
  pub window: window::Window,
  pub control_flow: ControlFlow,
}

pub struct UIState {
  event_loop_proxy: EventLoopProxy<EscherEvent>,
  modifier: util::EventModifier,
  toplevel_id: UIId,
  ui_scale: f32,

  current_time: time::Instant,
}

pub struct UIHierarchy {
  state: UIState,
  entities: HashMap<window::WindowId, UI>,
}

pub struct FullUIInput<'a> {
  pub event: Event<'a, EscherEvent>,
  pub window_target: &'a EventLoopWindowTarget<EscherEvent>,
  pub control_flow: &'a mut ControlFlow,
}

pub enum UIInputKind<'a> {
  Redraw,
  WindowEvent(&'a WindowEvent<'a>),
  Resize {width: Option<u32>, height: Option<u32>, scale: Option<f32>}
}
pub struct UIInput<'a> {
  pub kind: UIInputKind<'a>,
  pub control_flow: &'a mut ControlFlow
}

#[derive(Debug)]
pub struct UIResult {
  id: UIId,
  mutate_control_flow: bool,
  fully_consumed_event: bool,
  drop: bool,
}
impl UIResult {
  pub fn default(id: UIId) -> Self {
    Self { id, mutate_control_flow: false, fully_consumed_event: false, drop: false }
  }
  pub fn with_new_control_flow(id: UIId) -> Self {
    Self { id, mutate_control_flow: true, fully_consumed_event: false, drop: false }
  }
  pub fn maybe_with(id: UIId, mutate_control_flow: bool) -> Option<Self> {
    if mutate_control_flow {
      Some(Self::with_new_control_flow(id))
    } else {
      None
    }
  }
}


pub struct FullUIResult;

#[derive(Debug)]
pub struct UIError;

pub type UIId = window::WindowId;


impl UI {
  pub fn new(partial_window: window::WindowBuilder, ui_impl: Option<UIType>, window_target: &EventLoopWindowTarget<EscherEvent>) -> Self {
    let window = partial_window.build(window_target).unwrap();

    let ctx = egui::Context::default();
    let mut style = (*ctx.style()).clone();
    style.visuals = crate::util::VisualsColorMap::with_rgba_to_srgba(Some(style.visuals))
      .map_state()
      .unwrap();
    ctx.set_style(style);
  
    Self {
      ctx,
      ui_impl,
      window,
      control_flow: ControlFlow::Poll,
    }
  }
 
}


impl UIHierarchy {
  pub fn new_escher_ui(event_loop: &EventLoop<EscherEvent>, scale_factor: f32) -> Self {
    let main_ui = main::MainWindow::new(&event_loop, scale_factor);
    let main_id = main_ui.get_id();
    let entities = HashMap::from([(main_id, main_ui)]);

    let state = UIState {
      event_loop_proxy: event_loop.create_proxy(),
      modifier: util::EventModifier::default(),
      toplevel_id: main_id,
      ui_scale: scale_factor,
      current_time: time::Instant::now(),
    };

    Self { state, entities }
  }

  pub fn get_toplevel_id(&self) -> UIId {
    self.state.toplevel_id
  } 
}
