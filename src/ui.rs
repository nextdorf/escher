pub mod event;
pub mod main;
mod error;
mod util;

use std::{collections::HashMap, time, rc::{Rc, Weak}};

use egui_winit::{
  egui, 
  winit::{
    self, 
    event_loop::{
      EventLoopClosed,
      EventLoopWindowTarget,
      ControlFlow, EventLoop, EventLoopProxy
    },
    window,
    event::Event
  }
};

// use crate::wgpustate::WgpuState;


#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum EscherEvent {
  RequestRedrawPath{ id: window::WindowId },
  Rescale(f32),
  Exit(u8),
}

pub mod constants {
  pub const ZOOM_100: f32 = 1.125;
  pub const ZOOM_PLUS: f32 = 1.125;
}

/*
 * Was die Herarchien angeht: Von oben nach unten immer Refcounted referenzieren, von unten nach
 * oben immer schwach/optional referenzieren. Der Grund ist, dass beidseitiges refcounten zu
 * zirkularem Refcounten führt. Die Konsequenz: Daten halten sich indirekt selbst am Leben.
 * Bei beidseitigem weak-ref werden die Daten direkt bei Verlassen der Scopes/Moven der Owner
 * gedroppt. 
 */

/// An abstraction over individual window types. Should be a sum type.
pub trait PartialEscherWindow<H, W> : Sized where H: EscherHierarchy<W, Self>, W: EscherWindow<H, Self> {
  fn get_owner(&self) -> Weak<W>;

  /// The ui code which is run by egui
  fn ui(&mut self, ctx: &egui::Context);
}
/// An abstraction over the abstract window type. We store PW as boxed Value 
pub trait EscherWindow<H, PW> : Sized where H: EscherHierarchy<Self, PW>, PW: PartialEscherWindow<H, Self> {
  fn as_rc(&self) -> Rc<Self>;

  /// A reference to the window
  fn get_native_window(&self) -> &window::Window;

  /// The unique identifier of the window.
  fn get_native_window_id(&self) -> window::WindowId {
    self.get_native_window().id()
  }


  /// Reference to the implementation of the window
  fn get_implementation(&self) -> &Box<PW>;
  fn get_mut_implementation(&mut self) -> &mut Box<PW>;

  /// Reference to the toplevel window
  fn get_hierarchy(&self) -> Weak<H>;
  // fn get_hierarchy(&self) -> &H;
  // fn get_mut_hierarchy(&mut self) -> &mut H;

  /// Reference to the parent window. If it is `None` then `self` should be its own toplevel. If it
  /// is `Some(parent)` then `parent.get_children` should contain `self`.
  fn get_parent(&self) -> Weak<Self>;
  // fn get_parent(&self) -> Option<&Self>;
  // fn get_mut_parent(&mut self) -> Option<&mut Self>;

  /// A HashMap of all direct children
  fn get_children(&self) -> &HashMap<window::WindowId, Rc<Self>>;
  // fn get_mut_children(&mut self) -> &mut HashMap<window::WindowId, Self>;
  /// Defacto `self.get_children().get_mut(..)`. If `None` then element does not exist.
  fn access_child(&mut self, id: &window::WindowId) -> Option<&mut Self>;
  /// Defacto `self.get_children().insert(..)` + some internal book keeping. Has to update its
  /// hierarchy, too. If `Some` then there was an id collision. Probably means there is a bug in
  /// winit/X11/wayland
  fn insert_child(&mut self, child: Rc<Self>) -> Option<Rc<Self>>;
  /// Defacto `self.get_children().remove(..)` + some internal book keeping. Has to update its
  /// hierarchy, too. If `None` then there is no item with said `id`
  fn remove_child(&mut self, id: &window::WindowId) -> Option<Rc<Self>>;

  /// Iterates over all children. `InnerT` refers to the type which used internally to cast the
  /// children into. `ResultT` refers to the resulting type.
  fn iter_over_all_children<InnerT, ResultT>(&self) -> ResultT where
    InnerT: FromIterator<(window::WindowId, Rc<Self>)> + IntoIterator<Item = (window::WindowId, Rc<Self>)>,
    ResultT: FromIterator<(window::WindowId, Rc<Self>)>
  {
    let self_window = std::iter::once((
      self.get_native_window_id(),
      self.as_rc().clone()
    ));
    let children = self.get_children();
    if children.is_empty() {
      self_window.collect()
    } else {
      self_window.chain(children.values().flat_map(|c| c.iter_over_all_children::<InnerT, InnerT>())).collect()
    }
  }


  /// Removes child with id `child_id` and adds it as a child to `new_parent`.
  fn reparent_child(&mut self, child_id: window::WindowId, new_parent: &mut Self) -> Result<(), error::HierarchyError> {
    match self.remove_child(&child_id) {
      Some(child) => {
        if let Some(collision) = new_parent.insert_child(child) {
          panic!("Bug in winit: window id collision for {:?}", collision.get_native_window_id())
        } else {
          Ok(())
        }
      },
      None => Err(error::HierarchyError::PathNotFound)
    }
  }

  /// Removes child with id `child_id` and adds it as a child to the `UI` found at `start.access_child(path[0]).access_child(path[1]).(..)`.
  /// If `is_relative` then `start` is `self`, otherwise `start` is `toplevel`.
  fn reparent_child_at(&mut self, child_id: window::WindowId, new_parent_id: window::WindowId) -> Result<(), error::HierarchyError> {
    if let Some(hierarchy) = self.get_hierarchy().upgrade() {
      if let Some(new_parent) = hierarchy.get_all_children().get_mut(&new_parent_id) {
        self.reparent_child(child_id, new_parent)
      } else {
        Err(error::HierarchyError::PathNotFound)
      }
    } else {
      Err(error::HierarchyError::HierarchyNotFound)
    }
  }

  fn reparent_child_to(&mut self, child_id: window::WindowId, path: Vec<window::WindowId>, is_relative: bool) -> Result<(), error::HierarchyError> {
    match self.lookup_child(path, is_relative) {
      Ok(new_parent) => self.reparent_child(child_id, &mut new_parent),
      Err(e) => Err(e),
    }
  }

  fn lookup_child(&self, path: Vec<window::WindowId>, is_relative: bool) -> Result<Rc<Self>, error::HierarchyError> {
    if is_relative {
      let mut current = self;
      for id in path {
        if let Some(next) = current.get_children().get(&id) {
          current = next.as_ref();
        } else {
          return Err(error::HierarchyError::PathNotFound);
        }
      }
      Ok(current.as_rc())
    } else if let Some(hierarchy) = self.get_hierarchy().upgrade() {
      hierarchy.get_toplevel().lookup_child(path, true)
    } else {
      Err(error::HierarchyError::HierarchyNotFound)
    }
  }

  fn get_ctx(&self) -> &egui::Context;

  fn run_ui_ctx(&mut self, raw_input: egui::RawInput) -> egui::FullOutput {
    let ui_impl = self.get_implementation().as_mut();
    self.get_ctx().run(raw_input, |ctx| ui_impl.ui(ctx))
  }
}

/// Stores information about the toplevel window and all its children and handles window events
pub trait EscherHierarchy<W, PW> : Sized where W: EscherWindow<Self, PW>, PW: PartialEscherWindow<Self, W> {
  /// Emits an event.
  /// 
  /// # Example
  /// `event_loop_proxy.send_event(event)`
  fn try_send_event(&self, event: EscherEvent) -> Result<(), EventLoopClosed<EscherEvent>>;

  /// Similar to `try_send_event` but ignores errors.
  fn send_event(&self, event: EscherEvent) {
    self.try_send_event(event).unwrap_or_default()
  }

  fn get_egui_winit_state(&self) -> &egui_winit::State;

  /// Stores the modifier keys between window events, like ctrl, shift, alt, super
  fn modifier(&self) -> &util::EventModifier;

  fn get_toplevel(&self) -> Rc<W>;

  fn get_all_children(&mut self) -> &HashMap<window::WindowId, Rc<W>>;

  fn handle_events(&mut self, event: Event<EscherEvent>, _window_target: &EventLoopWindowTarget<EscherEvent>, control_flow: &mut ControlFlow);
}


// // impl<W, H, PW> Ord for W where W: EscherWindow<H, PW>, H: EscherHierarchy<W, PW>, PW: PartialEscherWindow<H, W> {
// impl<H, PW> Ord for EscherWindow<H, PW> {
//   fn cmp(&self, other: &Self) -> cmp::Ordering {
//     self.get_native_window_id().cmp(&other.get_native_window_id())
//   }
// }
// impl<W, H, PW> PartialOrd for W where W: EscherWindow<H, PW> {
//   fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
//     Some(self.cmp(other))
//   }
// }
// impl<W, H, PW> PartialEq for W where W: EscherWindow<H, PW> {
//   fn eq(&self, other: &Self) -> bool {
//     self.get_native_window_id() == other.get_native_window_id()
//   }
// }
// impl<W, H, PW> Eq for W where W: EscherWindow<H, PW> { }


/// The sum type over all possible kinds of windows. It handles the window-kind specific code
/// and variables. Every reference should hold an implementation of the PartialEscherWindow trait
pub enum UIType {
  Main(main::MainEscherWindow),
} 

/// Generic abstraction over UIType. It handles the window system and window hierarchy. For the
/// specification see `ui_state: UIType<'inner>`
pub struct UI {
  rc_self: Weak<Self>,
  ctx: egui::Context,
  ui_implementation: Option<Box<UIType>>,
  window: window::Window,
  hierarchy: Weak<UIHierarchy>,
  parent: Weak<UI>,
  children: HashMap<window::WindowId, Rc<UI>>,
}

pub struct UIHierarchy {
  event_loop_proxy: EventLoopProxy<EscherEvent>,
  modifier: util::EventModifier,
  egui_winit_state: egui_winit::State,
  toplevel: Rc<UI>,
  all_children_cache: Option<HashMap<window::WindowId, Rc<UI>>>,

  fps: f64,
  last_frame_time: time::Instant,
}

impl PartialEscherWindow<UIHierarchy, UI> for UIType {
  fn ui(&mut self, ctx: &egui::Context) {
    match self {
      Self::Main(w) => w.ui(ctx),
    }
  }

  fn get_owner(&self) -> Weak<UI> {
    match self {
      Self::Main(w) => w.owning_ui,
    }
  }
}

impl EscherWindow<UIHierarchy, UIType> for UI {
  fn as_rc(&self) -> Rc<Self> {
    self.rc_self.upgrade().unwrap()
  }

  fn get_native_window(&self) -> &window::Window {
    &self.window
  }

  fn get_implementation(&self) -> &Box<UIType> {
    &self.ui_implementation.expect("UI was not implemented")
  }
  fn get_mut_implementation(&mut self) -> &mut Box<UIType> {
    &mut self.ui_implementation.expect("UI was not implemented")
  }

  fn get_hierarchy(&self) -> Weak<UIHierarchy> {
    self.hierarchy.clone()
  }


  fn get_parent(&self) -> Weak<Self> {
    self.parent.clone()
  }

  fn get_children(&self) -> &HashMap<window::WindowId, Rc<Self>> {
    &self.children
  }

  fn access_child(&mut self, id: &window::WindowId) -> Option<&mut Self> {
    match self.children.get_mut(id) {
      Some(ch) => Some(ch),
      None => None
    }
  }

  fn insert_child(&mut self, child: Rc<Self>) -> Option<Rc<Self>> {
    if let Some(hierarchy) = self.hierarchy.upgrade() {
      hierarchy.all_children_cache = None;
    }
    self.children.insert(child.get_native_window_id(), child)
  }

  fn remove_child(&mut self, id: &window::WindowId) -> Option<Rc<Self>> {
    if let Some(hierarchy) = self.hierarchy.upgrade() {
      hierarchy.all_children_cache = None;
    }
    self.children.remove(id)
  }

  fn get_ctx(&self) -> &egui::Context {
    &self.ctx
  }

  
}


impl UIHierarchy {
  pub fn new(event_loop: &EventLoop<EscherEvent>) -> Self {
    let toplevel = todo!();
    let egui_winit_state = egui_winit::State::new(event_loop);
    egui_winit_state.set_pixels_per_point(constants::ZOOM_100);
    
    Self {
      event_loop_proxy: event_loop.create_proxy(),
      modifier: util::EventModifier::default(),
      egui_winit_state,
      toplevel,
      all_children_cache: None,
      fps: 60.,
      last_frame_time: time::Instant::now(),
    }
  }
}

impl UI {
  pub fn new(event_loop_deref: &EventLoopWindowTarget<EscherEvent>, hierarchy: Weak<UIHierarchy>,
    window_builder: window::WindowBuilder, ui_implementation: Option<Box<UIType>>) -> Result<Rc<Self>, winit::error::OsError>
  {
    let ctx = egui::Context::default();
    ctx.set_style({
      let mut style = (*ctx.style()).clone();
      style.visuals = crate::util::VisualsColorMap::with_rgba_to_srgba(Some(style.visuals))
        .map_state()
        .unwrap();
      style
    });

    let window = window_builder.build(event_loop_deref)?;
  
    Ok(Rc::new_cyclic(|weak| Self {
      rc_self: weak.clone(),
      ctx,
      ui_implementation,
      window,
      hierarchy,
      parent: Weak::new(),
      children: HashMap::default()
    }))
  }
  
  pub fn new_main(event_loop_deref: &EventLoopWindowTarget<EscherEvent>, hierarchy: Weak<UIHierarchy>) -> Rc<Self> {
    let window_builder = winit::window::WindowBuilder::new()
      .with_decorations(true)
      .with_resizable(true)
      .with_transparent(false)
      .with_title("escher")
      .with_inner_size(winit::dpi::PhysicalSize {
        width: 45*16,
        height: 45*9,
      });

    let mut ret = Self::new(event_loop_deref, hierarchy, window_builder, None).unwrap();
    ret.ui_implementation = Some(Box::new(UIType::Main(
      main::MainEscherWindow::new(Rc::downgrade(&ret)).unwrap()
    )));
    ret
  }
  
}

