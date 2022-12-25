use egui_winit::{
  egui,
  winit::{
    self,
    event_loop::{
      EventLoopProxy,
      EventLoopWindowTarget, ControlFlow,
    },
    window,
  }
};
use epaint::vec2;
use super::{EscherEvent, UIState, UIType, UI, simple::{SimpleWindow, WindowDrawRes}};

use crate::assets::{self, AssetManager};


pub struct MainWindow {
  pub expand_assets: bool,
  pub(super) inner: SimpleWindow,
  pub asset_manager: AssetManager,
}


impl MainWindow {
  pub fn redraw(&mut self, ctx: &egui::Context, window: &window::Window, state: &UIState, control_flow: &mut ControlFlow) -> WindowDrawRes {
    let inner = &mut unsafe {(self as *mut Self).as_mut()}.unwrap().inner;
    inner.redraw(ctx, window, state, control_flow, |ctx, state| self.ui(ctx, state))
  }

  pub fn resize(&mut self, width: Option<u32>, height: Option<u32>, scale: Option<f32>) {
    self.inner.resize(width, height, scale)
  }

  fn ui(&mut self, ctx: &egui::Context, state: &UIState) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui|
      self.ui_menu_bar(ui, &state.event_loop_proxy)
    );
    
    egui::TopBottomPanel::bottom("Timeline")
      .resizable(true)
      .show(ctx, |ui| {
        ui.centered_and_justified(|center_ui| center_ui.label("Timeline"));
    });
    egui::SidePanel::left("Assets")
      .resizable(true)
      .show_animated(ctx, self.expand_assets, |ui| {
        egui::ScrollArea::vertical().always_show_scroll(true).show(ui, |ui| {
          ui.horizontal_wrapped(|ui| {
            // for _ in 0..30 {
            //   ui.label("text");
            // }
            for asset in self.asset_manager.iter() {
              if let Ok(asset_lock) = asset.lock() {
                let tex_handle = asset_lock.get_texture_handle();
                for _ in 0..10 {
                  ui.image(tex_handle.id(), vec2(64., 64.));
                }
              }
            }
      })})});
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.centered_and_justified(|center_ui| center_ui.label("Video"));
    });

    // self.show_dialogs(ctx);
  }

  pub fn new(window_target: &EventLoopWindowTarget<EscherEvent>, scale_factor: f32) -> UI {
    let (mut res, inner) = SimpleWindow::new(
      window::WindowBuilder::new()
        .with_decorations(false)
        .with_resizable(true)
        .with_transparent(false)
        .with_title("escher")
        .with_inner_size(winit::dpi::PhysicalSize {
          width: 45*16,
          height: 45*9,
        }),
      window_target,
      scale_factor
    );

    // let img_hnd = vec![
    //   res.ctx.load_texture("uv_texture",
    //     (|| {
    //       let size = [256, 256];
    //       let mut rgba = Vec::with_capacity(size[0]*size[1]*4);
    //       for j in 0..size[1] {
    //         for i in 0..size[0] {
    //           let r = ((i as f32) / ((size[0]-1) as f32) * 255.).round() as _;
    //           let g = ((j as f32) / ((size[1]-1) as f32) * 255.).round() as _;
    //           rgba.push(r);
    //           rgba.push(g);
    //           rgba.push(0);
    //           rgba.push(255);
    //         }
    //       }
          
    //       egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_slice())
    //     })(),
    //     egui::TextureOptions::default()),
    //     res.ctx.load_texture("sample_texture",
    //     egui::ColorImage::example(),
    //     egui::TextureOptions::default()),
    // ];

    res.ui_impl = Some(UIType::Main(Box::new(
      Self {
        expand_assets: true,
        inner,
        asset_manager: AssetManager::default()
      }
    )));
    res
  }

  pub fn ui_menu_bar(&mut self, ui: &mut egui::Ui, event_proxy: &EventLoopProxy<EscherEvent>) {
    egui::menu::bar(ui, |ui| {
      ui.menu_button("File", |ui| {
        if ui.button("Assets").clicked() {
          self.expand_assets = !self.expand_assets;
        }
        ui.separator();
        if ui.button("Exit").clicked() {
          event_proxy.send_event(EscherEvent::Exit(0)).unwrap();
        }
      });

      ui.menu_button("Edit", |_| {});

      ui.menu_button("Help", |ui| {
        if ui.button("License").clicked() {
          event_proxy.send_event(EscherEvent::NewDialog).unwrap()
        }
      });
    });
  }


}

