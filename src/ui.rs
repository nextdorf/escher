pub mod event;
pub mod main;
mod error;
mod util;

use std::collections::HashMap;

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


pub trait PartialEscherWindow<T: EscherWindow>: Sized {
  fn modifier(&self) -> &util::EventModifier;

  fn ui(&mut self, ctx: &egui::Context);
}
pub trait EscherWindow: PartialEscherWindow<Self> {
  /// A reference to the window
  fn get_native_window(&self) -> &window::Window;

  /// The unique identifier of the window.
  fn get_native_window_id(&self) -> window::WindowId {
    self.get_native_window().id()
  }

  /// Emits an event.
  /// 
  /// # Example
  /// `event_loop_proxy.send_event(event)`
  fn try_send_event(&self, event: EscherEvent) -> Result<(), EventLoopClosed<EscherEvent>>;

  /// Similar to `try_send_event` but ignores errors.
  fn send_event(&self, event: EscherEvent) {
    self.try_send_event(event).unwrap_or_default()
  }


  /// Reference to the toplevel window
  fn get_toplevel(&self) -> &Self;
  fn get_mut_toplevel(&mut self) -> &mut Self;

  /// Reference to the parent window. If it is `None` then `self` should be its own toplevel. If it
  /// is `Some(parent)` then `parent.get_children` should contain `self`.
  fn get_parent(&self) -> Option<&Self>;
  fn get_mut_parent(&mut self) -> Option<&mut Self>;

  /// A HashMap of all direct children
  fn get_children(&self) -> &HashMap<window::WindowId, Self>;
  // fn get_mut_children(&mut self) -> &mut HashMap<window::WindowId, Self>;
  /// Defacto `self.get_children().get_mut(..)`. If `None` then element does not exist.
  fn access_child(&mut self, id: &window::WindowId) -> Option<&mut Self>;
  /// Defacto `self.get_children().insert(..)` + some internal book keeping. If `Some` then there
  /// was an id collision. Probably means there is a bug in winit/X11/wayland
  fn insert_child(&mut self, child: Self) -> Option<Self>;
  /// Defacto `self.get_children().remove(..)` + some internal book keeping. If `None` then there
  /// is no item with said `id`
  fn remove_child(&mut self, id: &window::WindowId) -> Option<Self>;

  /// Iterates over all children. `InnerT` refers to the type which used internally to cast the
  /// children into. `ResultT` refers to the resulting type.
  fn iter_over_all_children<'a, InnerT, ResultT>(&'a self) -> ResultT where
    InnerT: FromIterator<(window::WindowId, &'a Self)> + IntoIterator<Item = (window::WindowId, &'a Self)>,
    ResultT: FromIterator<(window::WindowId, &'a Self)>
  {
    let self_window = std::iter::once((self.get_native_window_id(), self));
    let children = self.get_children();
    if children.is_empty() {
      self_window.collect()
    } else {
      self_window.chain(children.values().flat_map(|c| c.iter_over_all_children::<InnerT, InnerT>())).collect()
    }
  }
  /// `self.iter_over_all_children::<InnerT, _>()`. In the default implementation `InnerT = Vec<_>`
  /// is picked.
  fn collect_all_children(&self) -> HashMap<window::WindowId, &Self> {
    self.iter_over_all_children::<Vec<_>, _>()
  }


  /// Removes child with id `child_id` and adds it as a child to `new_parent`.
  fn reparent_child(&mut self, child_id: window::WindowId, new_parent: &mut Self) -> Result<(), error::ReparentError> {
    match self.remove_child(&child_id) {
      Some(child) => {
        if let Some(collision) = new_parent.insert_child(child) {
          panic!("Bug in winit: window id collision for {:?}", collision.get_native_window_id())
        } else {
          Ok(())
        }
      },
      None => Err(error::ReparentError::PathNotFound)
    }
  }

  /// Removes child with id `child_id` and adds it as a child to the `UI` found at `start.access_child(path[0]).access_child(path[1]).(..)`.
  /// If `is_relative` then `start` is `self`, otherwise `start` is `toplevel`.
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
          match new_parent.access_child(curr_id) {
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
}


pub enum UIState<'a> {
  Main(main::MainEscherWindow<'a>),
} 

pub struct UI<'toplevel, 'parent, 'inner> {
  ui_state: UIState<'inner>,
  window: window::Window,
  toplevel: &'toplevel main::MainEscherWindow<'toplevel>,
  parent: Option<&'parent UI<'toplevel, 'parent, 'parent>>,
  children: HashMap<window::WindowId, UI<'toplevel, 'inner, 'inner>>,
  event_loop_proxy: event_loop::EventLoopProxy<EscherEvent>,
}

impl PartialEscherWindow<Self> for UI<'_, '_, '_> {
  fn ui(&mut self, ctx: &egui::Context) {
    match self.ui_state {
      UIState::Main(w) => w.ui(ctx)
    }
  }
}

impl EscherWindow for UI<'_, '_, '_> {
  fn get_native_window(&self) -> &window::Window {
    &self.window
  }

  fn try_send_event(&self, event: EscherEvent) -> Result<(), EventLoopClosed<EscherEvent>> {
    self.event_loop_proxy.send_event(event)
  }

  fn send_event(&self, event: EscherEvent) {
    self.try_send_event(event).expect("Cannot send event")
  }

  fn get_toplevel(&self) -> &Self {
    self.toplevel.get_owning_ui()
  }

  fn get_mut_toplevel(&mut self) -> &mut Self {
    self.toplevel.get_mut_owning_ui()
  }

  fn get_parent(&self) -> Option<&Self> {
    self.parent
  }

  fn get_mut_parent(&mut self) -> Option<&mut Self> {
    if let Some(parent) = self.parent {
      Some(&mut parent)
    } else {
      None
    }
  }

  fn get_children(&self) -> &HashMap<window::WindowId, Self> {
    &self.children
  }

  fn get_mut_children(&mut self) -> &mut HashMap<window::WindowId, Self> {
    &mut self.children
  }
}

impl UI<'_, '_, '_> {
  pub fn get_fps(&self) -> f64 {
    self.toplevel.get_fps()
  }

  fn collect_all_children_cached(&mut self) -> &HashMap<window::WindowId, &Self> {
    self.toplevel
      .all_children_cache
      .get_or_insert_with(|| self.collect_all_children())

  }
}

// pub fn run_ui(ui: &mut UI) {
//   ui.toplevel
// }

