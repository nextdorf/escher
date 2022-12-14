use egui_winit::winit::event::WindowEvent;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct EventModifier {
  ctrl_modifier: bool,
  shift_modifier: bool,
  alt_modifier: bool,
  super_modifier: bool,
}

impl EventModifier {
  pub fn ctrl(&self) -> bool { self.ctrl_modifier }
  pub fn shift(&self) -> bool { self.shift_modifier }
  pub fn alt(&self) -> bool { self.alt_modifier }
  pub fn super_key(&self) -> bool { self.super_modifier }
}

pub fn update_event_modifier(modifier: &mut EventModifier, event: WindowEvent) {
  match event {
    WindowEvent::ModifiersChanged(mod_key) => {
      modifier.ctrl_modifier = mod_key.ctrl();
      modifier.shift_modifier = mod_key.shift();
      modifier.alt_modifier = mod_key.alt();
      modifier.super_modifier = mod_key.logo();
    },
    _ => {}
  }
}

