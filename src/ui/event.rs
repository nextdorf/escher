use std::time;

use egui_winit::{
  egui,
  State,
  winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget},
    window,
  },
};

use super::{EscherEvent, UI};
use crate::wgpustate::WgpuState;


pub fn wait_at_most_until(control_flow: &mut ControlFlow, t: time::Instant) {
  if let Some(new_control_flow) = match *control_flow {
    ControlFlow::Wait | ControlFlow::Poll => Some(ControlFlow::WaitUntil(t)),
    ControlFlow::WaitUntil(t2) if t2 > t => Some(ControlFlow::WaitUntil(t)),
    _ => None
  }{
    *control_flow = new_control_flow;
  }
}

static mut START_TIME_STORE: Option<time::Instant> = None;
pub fn start_time() -> &'static time::Instant { unsafe {START_TIME_STORE.as_ref().unwrap()} }

pub fn handle_events(event: Event<EscherEvent>, _window_target: &EventLoopWindowTarget<EscherEvent>, control_flow: &mut ControlFlow,
  window: &window::Window, win_state: &mut State, ctx: &egui::Context, render_state: &mut WgpuState, ui_state: &mut UI)
{
  // let event_str = format!("{:?}", event);

  match event {
    Event::WindowEvent { window_id, event } if window_id==window.id() => {
      if !win_state.on_event(&ctx, &event) {
        match event {
          WindowEvent::Resized(PhysicalSize { width, height}) =>
            render_state.resize(Some(width), Some(height), None, win_state),
          WindowEvent::CloseRequested | WindowEvent::Destroyed => 
            *control_flow = ControlFlow::Exit,
          WindowEvent::KeyboardInput { input, .. } => {
            if let KeyboardInput { virtual_keycode: Some(VirtualKeyCode::LControl), state,.. } = input {
              ui_state.lctrl_modifier = state == ElementState::Pressed;
            }
            else if let KeyboardInput { virtual_keycode: Some(VirtualKeyCode::RControl), state,.. } = input {
              ui_state.rctrl_modifier = state == ElementState::Pressed;
            }
            else if let KeyboardInput { virtual_keycode: Some(keycode), state: ElementState::Pressed,.. } = input {
              match keycode {
                VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                VirtualKeyCode::F10 => println!("FPS: {}", ui_state.get_fps()),
                _ => {}
              }
            }
            if ui_state.ctrl_modifier() {
              if let KeyboardInput { virtual_keycode: Some(keycode), state: ElementState::Pressed,.. } = input {
                match keycode {
                  VirtualKeyCode::Plus | VirtualKeyCode::NumpadAdd => {
                    let scale_factor = win_state.pixels_per_point() * super::constants::ZOOM_PLUS;
                    render_state.resize(None, None, Some(scale_factor), win_state);
                  },
                  VirtualKeyCode::Minus | VirtualKeyCode::NumpadSubtract => {
                    let scale_factor = win_state.pixels_per_point() / super::constants::ZOOM_PLUS;
                    render_state.resize(None, None, Some(scale_factor), win_state);
                  },
                  VirtualKeyCode::Key0 | VirtualKeyCode::Numpad0 =>
                    render_state.resize(None, None, Some(super::constants::ZOOM_100), win_state),
                  _ => {}
                }
              }
            }
          },
          WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => {
            render_state.resize(Some(new_inner_size.width), Some(new_inner_size.height), Some(scale_factor as _), win_state);
          },
          _ => {}
        }
      }
      window.request_redraw();
    },
    Event::DeviceEvent { .. } => {
      //TODO
      window.request_redraw();
    },
    Event::RedrawRequested(window_id) if window_id != window.id() => { },
    Event::RedrawRequested(..) => {
      render_state.update_window_size_bind_group(false);
  
      let _did_render = render_state.redraw(|| {
        let raw_input = win_state.take_egui_input(&window);
        let full_output = ctx.run(raw_input, |ctx| ui_state.ui(ctx));
        let time_until_repaint = full_output.repaint_after;
        if time_until_repaint.is_zero() {
          *control_flow = ControlFlow::Poll;
        } else if time_until_repaint == time::Duration::MAX {
          *control_flow = ControlFlow::Wait;
        } else {
          // wait_at_most_until(control_flow, time::Instant::now() + time_until_repaint);
          *control_flow = ControlFlow::WaitUntil(time::Instant::now() + time_until_repaint);
        }

        win_state.handle_platform_output(&window, &ctx, full_output.platform_output);
        let paint_jobs = ctx.tessellate(full_output.shapes);
        let texture_delta = full_output.textures_delta;
        
        (texture_delta, paint_jobs)
      }).and(Some(true))
      .unwrap_or_else(|| {
        eprintln!("Incomplete rendering");
        false
      });

      ui_state.update_fps();
    },
    Event::UserEvent(EscherEvent::Exit(err_code)) =>
      *control_flow = if err_code==0 {ControlFlow::Exit} else {ControlFlow::ExitWithCode(err_code as _)},
    Event::MainEventsCleared => {
    },
    Event::UserEvent(EscherEvent::RequestRedrawPath { .. }) => {
      *control_flow = ControlFlow::Poll;
      window.request_redraw();
    },
    Event::NewEvents(start_cause) => match start_cause {
      StartCause::Init  => unsafe {START_TIME_STORE.get_or_insert(time::Instant::now());},
      StartCause::ResumeTimeReached { .. } => window.request_redraw(),
      StartCause::Poll => window.request_redraw(),
      StartCause::WaitCancelled { .. } => {}
    },
    _ => {}
  }

  // if *control_flow != ControlFlow::Wait {
  //   eprintln!("[{:?}] {:?}: {:?}", ((time::Instant::now() - *start_time()).as_secs_f32()*60.) as usize, *control_flow, event_str)
  // }
}

