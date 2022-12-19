pub mod event;
pub mod main;
mod error;
mod util;

use std::{collections::{HashMap, HashSet}, time};

use egui_winit::{
  egui, 
  winit::{
    event_loop::{
      EventLoopWindowTarget,
      ControlFlow, EventLoopProxy, EventLoop
    },
    window,
    event::{Event, WindowEvent}, dpi::PhysicalSize
  }
};

use super::hierarchy::{Entity, Hierarchy};
// use crate::wgpustate::WgpuState;


#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum EscherEvent {
  RequestRedrawPath{ id: UIId },
  Rescale(f32),
  Exit(u8),
}

pub mod constants {
  pub const ZOOM_100: f32 = 1.125;
  pub const ZOOM_PLUS: f32 = 1.125;
}


pub enum UIType {
  Main(Box<main::MainWindow>),
  // Dynamic(Box<dyn Entity>),
} 

/// Generic abstraction over UIType. It handles the window system and window hierarchy. For the
/// specification see `ui_state: UIType<'inner>`
pub struct UI {
  pub ctx: egui::Context,
  pub ui_impl: Option<UIType>,
  pub window: window::Window,
}

pub struct UIState {
  event_loop_proxy: EventLoopProxy<EscherEvent>,
  modifier: util::EventModifier,
  toplevel_id: UIId,

  fps: f64,
  last_frame_time: time::Instant,
}

pub struct UIHierarchy {
  state: UIState,
  entities: HashMap<window::WindowId, UI>,
}

pub struct UIInput<'a> {
  pub event: Event<'a, EscherEvent>,
  pub window_target: &'a EventLoopWindowTarget<EscherEvent>,
  pub control_flow: &'a mut ControlFlow,
}

pub struct UIResult {}

pub struct UIError {}

pub type UIId = window::WindowId;

impl<'a> Entity<UIId, UIInput<'a>, UIState, UIResult, UIError, UIHierarchy> for UI {
  fn get_id(&self) -> UIId {
    self.window.id()
  }

  fn run(&mut self, state: &UIState, input: &UIInput) -> Option<UIResult> {
    if let Some(ui_impl) = &mut self.ui_impl {
      // match ui_impl {
      //   UIType::Main(w) => w.as_mut().ui(&self.ctx, state),
      // };
      None
    } else {
      None
    }
  }
}

impl<'a> Hierarchy<UIId, UI, UIInput<'a>, UIState, UIResult, UIError> for UIHierarchy {
  fn get_state(&self) -> &UIState {
    &self.state
  }

  fn update_entities<F, G>(&mut self, f: F) -> G where F: Fn(HashMap<UIId, UI>) -> (G, HashMap<UIId, UI>) {
    let res;
    let entities = std::mem::take(&mut self.entities);
    (res, self.entities) = f(entities);
    res
  }

  fn access_entity(&mut self, id: &UIId) -> Option<&mut UI> {
    self.entities.get_mut(id)
  }

  fn accumulate_results(&mut self, results: Vec<UIResult>) -> Result<Option<(Option<HashSet<UIId>>, UIInput<'a>)>, UIError> {
    todo!()
  }
}


impl UI {
  pub fn new(partial_window: window::WindowBuilder, ui_impl: Option<UIType>, window_target: &EventLoopWindowTarget<EscherEvent>, scale_factor: f32) -> Self {
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
      window
    }
  }

  pub fn redraw(&mut self, state: &UIState) {
    match &mut self.ui_impl {
      Some(UIType::Main(main_window)) => main_window.redraw(&self.ctx, &self.window, state),
      None => {},
    }
  }

  pub fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
    match &mut self.ui_impl {
      None => false,
      Some(UIType::Main(main_window)) => {
        let egui_winit_state_result = main_window.egui_winit_state.on_event(&self.ctx, &event);
        if egui_winit_state_result.consumed {
          match event {
            WindowEvent::Resized(PhysicalSize { width, height}) =>
              main_window.resize(Some(*width), Some(*height), None),
            _ => {}
          }
        }
        if egui_winit_state_result.repaint {
          self.window.request_redraw();
        }
        egui_winit_state_result.consumed
      }
    }
  }

}


impl UIHierarchy {
  pub fn handle_events(&mut self, event: Event<EscherEvent>, _window_target: &EventLoopWindowTarget<EscherEvent>, control_flow: &mut ControlFlow) {
    match event {
      Event::WindowEvent { window_id, event } => {
        let is_consumed = match self.access_entity(&window_id) {
          Some(ui_val) => ui_val.handle_window_event(&event),
          None => false,
        };
        if !is_consumed {
          match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed if window_id == self.state.toplevel_id =>
              *control_flow = ControlFlow::Exit,
            WindowEvent::ModifiersChanged(modifier_state) =>
              util::update_event_modifier(&mut self.state.modifier, modifier_state),
            _ => {}
          }
        }
      },
      Event::UserEvent(EscherEvent::Exit(err_code)) =>
        *control_flow = ControlFlow::ExitWithCode(err_code as _),
      Event::RedrawRequested(id) => if let Some(ui_impl) = self.entities.get_mut(&id) {
        ui_impl.redraw(&self.state)
      }
      _ => {}
    }
  }

  pub fn new_escher_ui(event_loop: &EventLoop<EscherEvent>, scale_factor: f32) -> Self {
    let main_ui = main::MainWindow::new(&event_loop, scale_factor);
    let main_id = main_ui.get_id();
    let entities = HashMap::from([(main_id, main_ui)]);

    let state = UIState {
      event_loop_proxy: event_loop.create_proxy(),
      modifier: util::EventModifier::default(),
      toplevel_id: main_id,
      fps: 60.,
      last_frame_time: time::Instant::now(),
    };

    Self { state, entities }
  }
}
