use epaint::{Color32, Rgba};
use egui_winit::egui::{self, style};

pub struct VisualsColorMap {
  visuals: egui::Visuals,
  color_map: Box<dyn Fn(&Color32) -> Color32>,
}

impl VisualsColorMap {
  pub fn new(visuals: egui::Visuals, color_map: impl Fn(&Color32) -> Color32 + 'static) -> Self {
    Self {
      visuals,
      color_map: Box::new(color_map)
    }
  }

  pub fn set(mut self, new_visuals: Option<egui::Visuals>, new_color_map: Option<impl Fn(&Color32) -> Color32 + 'static>) -> Self {
    if let Some(visuals) = new_visuals {
      self.visuals = visuals;
    }
    if let Some(color_map) = new_color_map {
      self.color_map = Box::new(color_map);
    }
    self
  }

  pub fn with_rgba_to_srgba(visuals: Option<egui::Visuals>) -> Self {
    Self::new(visuals.unwrap_or_default(), |color| {
      let [r, g, b, a] = Rgba::from(*color)
        .to_array()
        .map(|x| if x <= 0. {0} else if x>=1. {255} else {(x*255.).round() as _});
      Color32::from_rgba_unmultiplied(r, g, b, a)
    })
  }

  pub fn map_state(mut self) -> Self {
    let mut_mapper = |c: &mut Color32| { *c = self.color_map.as_ref()(c) };
    Self::map_visuals(&mut self.visuals, &mut_mapper);
    self
  }

  fn map_visuals(visuals: &mut egui::Visuals, mut_mapper: &impl Fn(&mut Color32)) {
    visuals.override_text_color.as_mut().map(mut_mapper);
    Self::map_widgets(&mut visuals.widgets, mut_mapper);
    Self::map_selection(&mut visuals.selection, mut_mapper);
    mut_mapper(&mut visuals.hyperlink_color);
    mut_mapper(&mut visuals.faint_bg_color);
    mut_mapper(&mut visuals.extreme_bg_color);
    mut_mapper(&mut visuals.code_bg_color);
    mut_mapper(&mut visuals.warn_fg_color);
    mut_mapper(&mut visuals.error_fg_color);
    Self::map_shadow(&mut visuals.window_shadow, mut_mapper);
    Self::map_shadow(&mut visuals.popup_shadow, mut_mapper);
  }

  fn map_widgets(widgets: &mut style::Widgets, mut_mapper: &impl Fn(&mut Color32)) {
    Self::map_widget_visuals(&mut widgets.noninteractive, mut_mapper);
    Self::map_widget_visuals(&mut widgets.inactive, mut_mapper);
    Self::map_widget_visuals(&mut widgets.hovered, mut_mapper);
    Self::map_widget_visuals(&mut widgets.active, mut_mapper);
    Self::map_widget_visuals(&mut widgets.open, mut_mapper);
  }

  fn map_widget_visuals(widget_visuals: &mut style::WidgetVisuals, mut_mapper: &impl Fn(&mut Color32)) {
    mut_mapper(&mut widget_visuals.bg_fill);
    Self::map_stroke(&mut widget_visuals.fg_stroke, mut_mapper);
    Self::map_stroke(&mut widget_visuals.bg_stroke, mut_mapper);
  }

  fn map_stroke(stroke: &mut egui::Stroke, mut_mapper: &impl Fn(&mut Color32)) {
    mut_mapper(&mut stroke.color);
  }

  fn map_selection(selection: &mut style::Selection, mut_mapper: &impl Fn(&mut Color32)) {
    mut_mapper(&mut selection.bg_fill);
    Self::map_stroke(&mut selection.stroke, mut_mapper);
  }

  fn map_shadow(shadow: &mut epaint::Shadow, mut_mapper: &impl Fn(&mut Color32)) {
    mut_mapper(&mut shadow.color);
  }

  pub fn unwrap(self) -> egui::Visuals {
    self.visuals
  }
}



