pub mod event;
pub mod main;
pub mod dialogs;
mod error;
mod simple;
mod util;

use std::{collections::{HashMap, HashSet}, time, cmp};

use egui_winit::{
  egui, 
  winit::{
    event_loop::{
      EventLoopWindowTarget,
      ControlFlow, EventLoopProxy, EventLoop
    },
    window,
    event::{Event, WindowEvent, StartCause}, dpi::PhysicalSize
  }
};

use self::dialogs::LicenseDialog;

use super::hierarchy::{Entity, Hierarchy, InteriorKind, InteriorRef};
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
  mutate_control_flow: bool, //TODO: Refactor this with Controlflow being owned by entity
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

impl<'a> Entity<UIId, UIInput<'a>, UIState, UIResult> for UI {
  fn get_id(&self) -> UIId {
    self.window.id()
  }

  fn run(&mut self, state: &UIState, input: &UIInput) -> Option<UIResult> {
    //TODO: Add modifier handling and rescaling.
    if let Some(ui_impl) = &mut self.ui_impl {
      match ui_impl {
        UIType::Main(main_window) => {
          let main_window = main_window.as_mut();
          match input.kind {
            UIInputKind::Redraw => match main_window.redraw(&self.ctx, &self.window, state, &mut self.control_flow) {
              simple::WindowDrawRes::InvaldRenderFrame => todo!(),
              simple::WindowDrawRes::NoRedrawScheduled(true) | simple::WindowDrawRes::RedrawNextFrame(true)
                | simple::WindowDrawRes::RedrawScheduled(_) => Some(UIResult::with_new_control_flow(self.get_id())),
              simple::WindowDrawRes::NoRedrawScheduled(false) | simple::WindowDrawRes::RedrawNextFrame(false) =>
                None,
            },
            UIInputKind::WindowEvent(event) => {
              let egui_winit_state_result = main_window.inner.egui_winit_state.on_event(&self.ctx, &event);
              let mut drop_window = false;
              if !egui_winit_state_result.consumed {
                match event {
                  WindowEvent::Resized(PhysicalSize { width, height}) =>
                    main_window.resize(Some(*width), Some(*height), None),
                  WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } =>
                    main_window.resize(Some(new_inner_size.width), Some(new_inner_size.height), Some(*scale_factor as _)),
                  WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    state.event_loop_proxy.send_event(EscherEvent::Exit(0)).unwrap_or_default();
                    drop_window = true
                  },
                  _ => {}
                }
              }
              if egui_winit_state_result.repaint {
                self.window.request_redraw();
              }
              if drop_window || egui_winit_state_result.consumed {
                Some(UIResult {
                  id: self.get_id(), 
                  mutate_control_flow: false,
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
        
        UIType::License(license_dialog) => {
          let license_dialog = license_dialog.as_mut();
          match input.kind {
            UIInputKind::Redraw => match license_dialog.redraw(&self.ctx, &self.window, state, &mut self.control_flow) {
              simple::WindowDrawRes::InvaldRenderFrame => todo!(),
              simple::WindowDrawRes::NoRedrawScheduled(true) | simple::WindowDrawRes::RedrawNextFrame(true)
                | simple::WindowDrawRes::RedrawScheduled(_) => Some(UIResult::with_new_control_flow(self.get_id())),
              simple::WindowDrawRes::NoRedrawScheduled(false) | simple::WindowDrawRes::RedrawNextFrame(false) =>
                None,
            },
            UIInputKind::WindowEvent(event) => {
              let egui_winit_state_result = license_dialog.inner.egui_winit_state.on_event(&self.ctx, &event);
              let mut drop_window = false;
              if !egui_winit_state_result.consumed {
                match event {
                  WindowEvent::Resized(PhysicalSize { width, height}) =>
                  license_dialog.resize(Some(*width), Some(*height), None),
                  WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } =>
                  license_dialog.resize(Some(new_inner_size.width), Some(new_inner_size.height), Some(*scale_factor as _)),
                  WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    drop_window = true
                  },
                  _ => {}
                }
              }
              if egui_winit_state_result.repaint {
                self.window.request_redraw();
              }
              if drop_window || egui_winit_state_result.consumed {
                Some(UIResult {
                  id: self.get_id(), 
                  mutate_control_flow: false,
                  fully_consumed_event: egui_winit_state_result.consumed,
                  drop: drop_window,
                })
              } else {
                None
              }
            },
            UIInputKind::Resize { width, height, scale } => {
              license_dialog.resize(width, height, scale);
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

fn cmp_control_flow(a: &ControlFlow, b: &ControlFlow) -> cmp::Ordering {
  match a {
    ControlFlow::ExitWithCode(_) => match b {
      ControlFlow::ExitWithCode(_) => cmp::Ordering::Equal,
      _ => cmp::Ordering::Less
    },
    ControlFlow::Poll => match b {
      ControlFlow::ExitWithCode(_) => cmp::Ordering::Greater,
      ControlFlow::Poll => cmp::Ordering::Equal,
      _ => cmp::Ordering::Less,
    },
    ControlFlow::WaitUntil(time_a) => match b {
      ControlFlow::ExitWithCode(_) | ControlFlow::Poll => cmp::Ordering::Greater,
      ControlFlow::WaitUntil(time_b) => time_a.cmp(time_b),
      ControlFlow::Wait => cmp::Ordering::Less,
    },
    ControlFlow::Wait => match b {
      ControlFlow::Wait => cmp::Ordering::Equal,
      _ => cmp::Ordering::Greater
    },
  }
}

impl<'event> Hierarchy<UIId, UI, UIInput<'event>, FullUIInput<'event>, FullUIInput<'event>, UIState, UIResult, FullUIResult, UIError,> for UIHierarchy {
  fn represent(&self, _state_kind: InteriorKind, _entities_kind: InteriorKind) -> (InteriorRef<UIState>, InteriorRef<HashMap<UIId, UI>>) {
    (InteriorRef::AsRef(&self.state), InteriorRef::AsRef(&self.entities))
  }

  fn represent_mut<'a, 'b, 'c: 'a + 'b>(&'c mut self, _state_kind: InteriorKind, _entities_kind: InteriorKind) -> (InteriorRef<'a, UIState>, InteriorRef<'b, HashMap<UIId, UI>>) {
    (InteriorRef::AsMut(&mut self.state), InteriorRef::AsMut(&mut self.entities))
  }

  fn accumulate_results(&mut self, results: Vec<UIResult>, input: FullUIInput<'event>) -> Result<Option<(Option<HashSet<UIId>>, FullUIInput<'event>)>, UIError> {
    let err = false;
    let mut control_flow = None;
    let entities: *mut HashMap<UIId, UI> = &mut self.entities as _;
    for UIResult {id, mutate_control_flow, drop, ..} in results {
      if drop {
        unsafe{ entities.as_mut() }.unwrap().remove(&id);
      } else if mutate_control_flow {
        if let Some(entity) = unsafe { entities.as_ref() }.unwrap().get(&id) {
          let current_control_flow = &entity.control_flow;
          match control_flow {
            Some(old_control_flow) if cmp_control_flow(old_control_flow, current_control_flow) != cmp::Ordering::Greater => {},
            _ => control_flow = Some(current_control_flow),
          }
        }
      }
    }
    if err {
      Err(UIError)
    } else if let Some(&control_flow) = control_flow {
      *input.control_flow = control_flow;
      Ok(Some((None, input)))
    } else {
      Ok(None)
    }
  }

  fn run(&mut self, ids: Option<HashSet<UIId>>, input: FullUIInput<'event>) -> Result<FullUIResult, UIError> {
    // let FullUIInput { event, window_target, control_flow} = input;
    self.state.current_time = time::Instant::now();
    let mut results = Vec::new();
    match &input.event {
      Event::UserEvent(EscherEvent::Exit(err_code)) => *input.control_flow = ControlFlow::ExitWithCode(*err_code as _),
      Event::MainEventsCleared => { },
      Event::RedrawRequested(id) | Event::UserEvent(EscherEvent::RequestRedraw {id}) => {
        if ids.is_none() || ids.unwrap().contains(&id) {
          if let Some(ui) = self.entities.get_mut(&id) {
            let ui_input = UIInput { kind: UIInputKind::Redraw, control_flow: input.control_flow };
            if let Some(res) = ui.run(&self.state, &ui_input) {
              results.push(res);
            }
          }
        }
      },
      Event::WindowEvent { window_id: id, event } => {
        if ids.is_none() || ids.unwrap().contains(&id) {
          if let Some(ui) = self.entities.get_mut(&id) {
            let ui_input = UIInput { kind: UIInputKind::WindowEvent(&event), control_flow: input.control_flow };
            if let Some(res) = ui.run(&self.state, &ui_input) {
              results.push(res);
            }
          }
        }
      },
      Event::UserEvent(EscherEvent::Rescale(scale)) => {
        let ui_input = UIInput {
          kind: UIInputKind::Resize { width: None, height: None, scale: Some(*scale) },
          control_flow: input.control_flow
        };
        match ids {
          Some(ids) => {
            results = Vec::with_capacity(ids.len());
            for id in ids {
              if let Some(ui) = self.entities.get_mut(&id) {
                if let Some(res) = ui.run(&self.state, &ui_input) {
                  results.push(res);
                }
              }
            }
          },
          None => {
            results = Vec::with_capacity(self.entities.len());
            for ui in self.entities.values_mut() {
              if let Some(res) = ui.run(&self.state, &ui_input) {
                results.push(res);
              }
            }
          },
        }
      },
      Event::UserEvent(EscherEvent::NewDialog) => {
        let new_dialog = LicenseDialog::new(input.window_target, constants::ZOOM_100);
        let id = new_dialog.get_id();
        self.entities.insert(id, new_dialog);
      },
      Event::NewEvents(start_cause) => {
        let req_time = match start_cause {
          StartCause::Poll => Some(self.state.current_time),
          StartCause::ResumeTimeReached { requested_resume, .. } => Some(*requested_resume),
          _ => None
        };
        if let Some(req_time) = req_time {
          // results = Vec::with_capacity(self.entities.len());
          for ui in self.entities.values() {
            let req_redraw = match ui.control_flow {
              ControlFlow::Poll => true,
              ControlFlow::WaitUntil(until_time) if until_time <= req_time => true,
              _ => false,
            };
            if req_redraw {
              ui.window.request_redraw();
            }
          }
        }
      },
      Event::LoopDestroyed => println!("Escher is ending!"),
      _ => {}
    }
    
    match self.accumulate_results(results, input) {
      Ok(None) => Ok(FullUIResult),
      Ok(Some((ids, _new_input))) => {assert!(ids.is_none()); Ok(FullUIResult)},
      Err(err) => Err(err),
    }
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
      window,
      control_flow: ControlFlow::Poll,
    }
  }

  /*
  pub fn redraw(&mut self, state: &UIState) {
    match &mut self.ui_impl {
      Some(UIType::Main(main_window)) => {main_window.redraw(&self.ctx, &self.window, state, &mut self.control_flow);},
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
        let egui_winit_state_result = main_window.inner.egui_winit_state.on_event(&self.ctx, &event);
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
  */
}


impl UIHierarchy {
  /*
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
  */

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
