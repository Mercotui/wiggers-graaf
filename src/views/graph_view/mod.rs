// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::board::BoardId;
use crate::graph::Graph;
use crate::views::frame_scheduler::{FrameScheduler, OnFrameCb};
use crate::views::graph_view::arrangement::Arrangement;
use crate::views::graph_view::camera::FlickableCamera;
use crate::views::graph_view::controls::{ControlEvent, Controls};
use crate::views::graph_view::renderer::Renderer;
use crate::views::resize_observer::ResizeObserver;
use crate::views::utils::{get_element_of_type, CanvasSpace};
use euclid::approxeq::ApproxEq;
use euclid::Size2D;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::Duration;
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;

pub mod arrangement;
mod camera;
mod controls;
mod renderer;

pub struct GraphView {
    _self_ref: Weak<RefCell<Self>>,
    frame_scheduler: FrameScheduler,
    _resize_observer: ResizeObserver,
    _controls: Rc<RefCell<Controls>>,
    canvas: HtmlCanvasElement,
    canvas_needs_size_update: bool,
    canvas_size: Size2D<f32, CanvasSpace>,
    previous_frame_timestamp: Option<Duration>,
    camera: FlickableCamera,
    renderer: Renderer,
}

impl GraphView {
    pub fn new(canvas_id: &str) -> Result<Rc<RefCell<Self>>, JsValue> {
        let canvas: HtmlCanvasElement = get_element_of_type(canvas_id)?;
        let renderer: Renderer = Renderer::new(&canvas)?;

        let view = Rc::new_cyclic(|self_ref| {
            let self_ref_for_on_frame_cb = self_ref.clone();
            let self_ref_for_resize_observer_cb = self_ref.clone();
            let self_ref_for_mouse_event_cb = self_ref.clone();

            RefCell::new(Self {
                _self_ref: self_ref.clone(),
                frame_scheduler: FrameScheduler::new(Box::new(move |timestamp: Duration| {
                    self_ref_for_on_frame_cb
                        .upgrade()
                        .unwrap()
                        .borrow_mut()
                        .draw(timestamp);
                }) as Box<OnFrameCb>),
                _resize_observer: ResizeObserver::new(
                    &canvas,
                    Box::new(move |width, height| {
                        self_ref_for_resize_observer_cb
                            .upgrade()
                            .unwrap()
                            .borrow_mut()
                            .resize(width, height);
                    }),
                ),
                _controls: Controls::new(
                    &canvas,
                    Box::new(move |event: ControlEvent| {
                        let self_ref_rc = self_ref_for_mouse_event_cb.upgrade().unwrap();
                        let mut self_ref = self_ref_rc.borrow_mut();
                        self_ref.camera.handle_pointer_event(event);
                        self_ref
                            .frame_scheduler
                            .schedule()
                            .expect("Could not schedule frame");
                    }),
                )
                .expect("Could not create graph controls"),
                canvas,
                canvas_needs_size_update: false,
                canvas_size: Size2D::zero(),
                previous_frame_timestamp: None,
                camera: FlickableCamera::new(),
                renderer,
            })
        });

        Ok(view)
    }

    fn resize(&mut self, width: f64, height: f64) {
        self.canvas_needs_size_update = true;
        self.renderer.set_viewport(width as i32, height as i32);
        self.canvas_size = Size2D::new(width as f32, height as f32);
        self.camera.resize_canvas(self.canvas_size);
        self.schedule_draw();
    }

    fn schedule_draw(&mut self) {
        self.frame_scheduler.schedule().unwrap();
    }

    fn draw(&mut self, timestamp: Duration) {
        let previous_timestamp = self.previous_frame_timestamp.get_or_insert(timestamp);
        let delta_time = previous_timestamp.as_secs_f32() - timestamp.as_secs_f32();
        // Update translation in case of flick
        // self.translation += self.translation_velocity * delta_time;
        // self.translation_velocity -= TRANSLATION_DRAG * delta_time;

        // Resize canvas if needed
        if self.canvas_needs_size_update {
            self.canvas_needs_size_update = false;
            self.canvas.set_width(self.canvas_size.width as u32);
            self.canvas.set_height(self.canvas_size.height as u32);
        }

        // Draw
        self.renderer.draw(self.camera.view_transform());

        // Schedule next draw if needed
        // if !self
        //     .translation_velocity
        //     .approx_eq(&TRANSLATION_VELOCITY_ZERO)
        // {
        self.schedule_draw();
        // }
    }

    pub fn set_data(&mut self, graph: &Graph, active_state: BoardId) {
        // Create an arrangement from the graph data
        let arrangement = Arrangement::new(graph, active_state);

        // Upload the data to the GPU
        let vertices_array = unsafe { js_sys::Float32Array::view(&arrangement.points) };
        self.renderer.set_data(&vertices_array);
        self.camera.resize_content(Size2D::new(
            arrangement.width as f32,
            arrangement.height as f32,
        ));
        self.schedule_draw();
    }
}
