use std::{sync::Arc, path::Path, ffi::{OsStr, CStr, CString, OsString}};

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
use escher_video::{RawImageRef, VideoStream};
use super::{EscherEvent, UIState, UIType, UI, simple::{SimpleWindow, WindowDrawRes}};

use crate::{assets::{self, AssetManager}, wgpustate::util::EscherWGPUCallbackFn};


static mut frame_buffer: Vec<u8> = Vec::new();

pub struct MainWindow {
  pub expand_assets: bool,
  pub(super) inner: SimpleWindow,
  pub asset_manager: AssetManager,
  // pub active_frame: Option<RawImageRef<'static>>,
  render_texture_id: usize,
  video_stream: Option<escher_video::VideoStream>
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
            for asset in self.asset_manager.iter() {
              if let Ok(asset_lock) = asset.lock() {
                for _ in 0..10 {
                  ui.add(asset_lock.as_widget());
                }
              }
            }
      })})});
    egui::CentralPanel::default().show(ctx, |ui| {
      // ui.centered_and_justified(|center_ui| center_ui.label("Video"));
      let (rect, _resp) = ui.allocate_exact_size(vec2(128., 128.), egui::Sense::hover());
      // self.video_stream.as_mut().unwrap().decode_frames(1, true).expect(".");
      ui.painter_at(rect).add(egui::PaintCallback {
        rect,
        callback: Arc::new(EscherWGPUCallbackFn::RenderFrame(
          self.render_texture_id,
          match &self.video_stream { //FIXME: BAD!
            Some(vs) => Some(unsafe{std::mem::transmute::<RawImageRef<'_>, RawImageRef<'static>>(vs.decoded_frm())}),
            // Some(vs) => Some(vs.decoded_frm()),
            None => None
          },
        )),
      });
      //TODO: Render frame with ffmpeg
    });

    // self.show_dialogs(ctx);
  }

  pub fn new(window_target: &EventLoopWindowTarget<EscherEvent>, scale_factor: f32) -> UI {
    let (mut res, mut inner) = SimpleWindow::new(
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

    // let active_frame = RawImageRef::new_dummy_rgba32(unsafe {&mut frame_buffer}, 512, 512);
    // let render_texture_id = inner.render_state.new_user_texture(
    //   wgpu::Extent3d { width: active_frame.width() as _, height: active_frame.height() as _, depth_or_array_layers: 1 },
    //   match active_frame.pix_fmt() {
    //     escher_video::AVPixelFormat::AV_PIX_FMT_RGBA => wgpu::TextureFormat::Rgba8Unorm,
    //     _ => todo!()
    //   },
    //   active_frame.planes()[0]
    // );
    // let active_frame = Some(active_frame);
    
    let video_stream: VideoStream = Ok(escher_video::PartialVideoStream::new()).and_then(|pvs| {
      pvs.open_format_context_from_path(Path::new("local/bunny_1080p_60fps.mp4"))?
        .open_codec_context(0, 16, -1)?
        .create_sws_context(1280, 720, escher_video::AVPixelFormat::AV_PIX_FMT_RGBA, escher_video::SWS_Scaling::Bilinear)?
        .create_pkt_frm()?
        .fmapMut(|vs| {
          vs.seek(2., escher_video::Seek::empty())?;
          vs.decode_frames(0, true)?;
          Ok(())
        })?
        .try_into()
    }).expect("ERROR");
    let active_frame = video_stream.decoded_frm();
    let render_texture_id = inner.render_state.new_user_texture(
      wgpu::Extent3d { width: active_frame.width() as _, height: active_frame.height() as _, depth_or_array_layers: 1 },
      match active_frame.pix_fmt() {
        escher_video::AVPixelFormat::AV_PIX_FMT_RGBA => wgpu::TextureFormat::Rgba8Unorm,
        _ => todo!()
      },
      active_frame.planes()[0]
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
        video_stream: Some(video_stream),
        render_texture_id,
        // active_frame: Some(active_frame),
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

