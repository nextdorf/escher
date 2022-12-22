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

use super::hierarchy::{Entity, Hierarchy, InteriorKind, InteriorRef};
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
  //TODO: Add the disired control flow as atttribute to entity
}

pub struct UIState {
  event_loop_proxy: EventLoopProxy<EscherEvent>,
  modifier: util::EventModifier,
  toplevel_id: UIId,

  fps: f64,
  last_frame_time: time::Instant,
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
  mutate_control_flow: Option<ControlFlow>, //TODO: Refactor this with Controlflow being owned by entity
  fully_consumed_event: bool,
  drop: bool,
}
impl UIResult {
  pub fn default(id: UIId) -> Self {
    Self { id, mutate_control_flow: None, fully_consumed_event: false, drop: false }
  }
  pub fn with(id: UIId, control_flow: ControlFlow) -> Self {
    Self { id, mutate_control_flow: Some(control_flow), fully_consumed_event: false, drop: false }
  }
}


pub struct FullUIResult;

pub struct UIError;

pub type UIId = window::WindowId;

impl<'a> Entity<UIId, UIInput<'a>, UIState, UIResult> for UI {
  fn get_id(&self) -> UIId {
    self.window.id()
  }

  fn run(&mut self, state: &UIState, input: &UIInput) -> Option<UIResult> {
    //TODO: Pass desired control flow as mut ref

    if let Some(ui_impl) = &mut self.ui_impl {
      match ui_impl {
        UIType::Main(main_window) => {
          match input.kind {
            UIInputKind::Redraw => Some(match main_window.redraw(&self.ctx, &self.window, state) {
              main::MainWindowDrawRes::InvaldRenderFrame => todo!(),
              main::MainWindowDrawRes::NoRedrawScheduled => UIResult::with(self.get_id(), ControlFlow::Wait),
              main::MainWindowDrawRes::RedrawNextFrame => UIResult::with(self.get_id(), ControlFlow::Poll),
              main::MainWindowDrawRes::RedrawScheduled(dtime) => UIResult::with(self.get_id(), ControlFlow::WaitUntil(state.current_time + dtime)),
            }),
            UIInputKind::WindowEvent(event) => {
              let egui_winit_state_result = main_window.egui_winit_state.on_event(&self.ctx, &event);
              let mut drop_window = false;
              if egui_winit_state_result.consumed {
                match event {
                  WindowEvent::Resized(PhysicalSize { width, height}) =>
                    main_window.resize(Some(*width), Some(*height), None),
                  WindowEvent::CloseRequested => drop_window = true, 
                  _ => {}
                }
              }
              if egui_winit_state_result.repaint {
                self.window.request_redraw();
              }
              if drop_window || egui_winit_state_result.consumed {
                Some(UIResult {
                  id: self.get_id(), 
                  mutate_control_flow: None,
                  fully_consumed_event: egui_winit_state_result.consumed,
                  drop: drop_window,
                })
              } else {
                None
              }
            },
            UIInputKind::Resize { width, height, scale } => {
              main_window.resize(width, height, scale);
              None
            },
          }
        },
      }
    } else {
      None
    }
  }
}

impl<'event> Hierarchy<UIId, UI, UIInput<'event>, FullUIInput<'event>, UIState, UIResult, FullUIResult, UIError,> for UIHierarchy {
  fn represent(&self, _state_kind: InteriorKind, _entities_kind: InteriorKind) -> (InteriorRef<UIState>, InteriorRef<HashMap<UIId, UI>>) {
    (InteriorRef::AsRef(&self.state), InteriorRef::AsRef(&self.entities))
  }

  fn represent_mut<'a, 'b, 'c: 'a + 'b>(&'c mut self, _state_kind: InteriorKind, _entities_kind: InteriorKind) -> (InteriorRef<'a, UIState>, InteriorRef<'b, HashMap<UIId, UI>>) {
    (InteriorRef::AsMut(&mut self.state), InteriorRef::AsMut(&mut self.entities))
  }

  fn accumulate_results(&mut self, results: Vec<UIResult>) -> Result<Option<(Option<HashSet<UIId>>, FullUIInput<'event>)>, UIError> {
    todo!()
  }

  fn run(&mut self, ids: Option<HashSet<UIId>>, input: FullUIInput<'event>) -> Result<FullUIResult, UIError> {
    let FullUIInput { event, window_target, control_flow} = input;
    self.state.current_time = time::Instant::now();
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
      Some(UIType::Main(main_window)) => {main_window.redraw(&self.ctx, &self.window, state);},
      None => {},
    }
  }

  pub fn resize(&mut self, width: Option<u32>, height: Option<u32>, scale: Option<f32>) {
    match &mut self.ui_impl {
      Some(UIType::Main(main_window)) => main_window.resize(width, height, scale),
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
      current_time: time::Instant::now(),
    };

    Self { state, entities }
  }
}
