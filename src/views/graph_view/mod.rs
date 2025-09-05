// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::board::BoardId;
use crate::graph::Graph;
use crate::views::frame_scheduler::{FrameScheduler, OnFrameCb};
use crate::views::graph_view::arrangement::Arrangement;
use crate::views::graph_view::controls::{ControlEvent, Controls};
use crate::views::graph_view::renderer::Renderer;
use crate::views::resize_observer::ResizeObserver;
use crate::views::utils::get_element_of_type;
use euclid::approxeq::ApproxEq;
use euclid::{Scale, Size2D, Transform2D, Vector2D};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::Duration;
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;

pub mod arrangement;
mod controls;
mod renderer;

/// This represents the view's content coordinate space, dynamic axes depending on the content size
struct ContentSpace;

impl ContentSpace {
    /// The offset of the content from the border, half a unit
    pub const PADDING: Vector2D<f32, Self> = Vector2D::new(0.5, 0.5);

    pub fn add_padding(size: Size2D<f32, ContentSpace>) -> Size2D<f32, ContentSpace> {
        // Calculate the additional size of the padding on all sides
        let total_padding_x = Self::PADDING.x * 2.0;
        let total_padding_y = Self::PADDING.y * 2.0;
        Size2D::new(size.width + total_padding_x, size.height + total_padding_y)
    }
}

/// This represents the OpenGL clip space, where the canvas x and y are both represented in [-1.0, 1.0]
struct ClipSpace;

impl ClipSpace {
    /// The clip-space coordinate space starts at [-1.0, -1.0]
    const CLIP_SPACE_OFFSET: Vector2D<f32, ClipSpace> = Vector2D::new(-1.0, -1.0);
    pub fn transform_from_canvas(
        size: Size2D<f32, CanvasSpace>,
    ) -> Transform2D<f32, CanvasSpace, ClipSpace> {
        let scale_x = 2.0 / size.width;
        let scale_y = 2.0 / size.height;
        Transform2D::scale(scale_x, scale_y).then_translate(Self::CLIP_SPACE_OFFSET)
    }
    pub fn transform_from_content(
        canvas_size: Size2D<f32, CanvasSpace>,
        content_size: Size2D<f32, ContentSpace>,
        zoom: Scale<f32, ClipSpace, ClipSpace>,
        translation: Vector2D<f32, ClipSpace>,
    ) -> Transform2D<f32, ContentSpace, ClipSpace> {
        let content_scale_x = (2.0 * zoom.get()) / content_size.width;
        let content_scale_y = content_scale_x * (canvas_size.width / canvas_size.height);
        Transform2D::scale(content_scale_x, content_scale_y)
            .pre_translate(ContentSpace::PADDING)
            .then_translate(translation)
    }
}

/// The decay factor of the drag velocity, per second
const TRANSLATION_DRAG_FACTOR: f32 = 0.5;

/// Zero velocity
const TRANSLATION_VELOCITY_ZERO: Vector2D<f32, ClipSpace> = Vector2D::new(0.0, 0.0);

/// This represents the Canvas coordinate system, where the canvas is represented in [0, pixel size]
struct CanvasSpace;

/// The velocity of a user scrolling
const _ZOOM_SPEED: Scale<f32, ClipSpace, ClipSpace> = Scale::new(1.0);

/// The minimum zoom level, this fits the whole contents into the clip-space, with some padding.
const _ZOOM_MINIMUM: Scale<f32, ClipSpace, ClipSpace> = Scale::new(1.0);

/// The maximum zoom level
const _ZOOM_MAXIMUM: Scale<f32, ClipSpace, ClipSpace> = Scale::new(5.0);

pub struct GraphView {
    _self_ref: Weak<RefCell<Self>>,
    frame_scheduler: FrameScheduler,
    _resize_observer: ResizeObserver,
    _controls: Rc<RefCell<Controls>>,
    canvas: HtmlCanvasElement,
    canvas_needs_size_update: bool,
    canvas_size: Size2D<f32, CanvasSpace>,
    content_size: Size2D<f32, ContentSpace>,
    canvas_to_clip: Transform2D<f32, CanvasSpace, ClipSpace>,
    zoom: Scale<f32, ClipSpace, ClipSpace>,
    translation: Vector2D<f32, ClipSpace>,
    translation_velocity: Vector2D<f32, ClipSpace>,
    view_transform: [f32; 9],
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
                        self_ref_for_mouse_event_cb
                            .upgrade()
                            .unwrap()
                            .borrow_mut()
                            .handle_pointer_event(event)
                    }),
                )
                .expect("Could not create graph controls"),
                canvas,
                canvas_needs_size_update: false,
                canvas_size: Size2D::zero(),
                content_size: Size2D::zero(),
                canvas_to_clip: Transform2D::identity(),
                zoom: Scale::identity(),
                translation: ClipSpace::CLIP_SPACE_OFFSET,
                translation_velocity: Vector2D::zero(),
                view_transform: [0.0; 9],
                renderer,
            })
        });

        view.borrow_mut().recalculate_view_transform();
        Ok(view)
    }

    fn resize(&mut self, width: f64, height: f64) {
        self.canvas_needs_size_update = true;
        self.renderer.set_viewport(width as i32, height as i32);
        self.canvas_size = Size2D::new(width as f32, height as f32);
        self.canvas_to_clip = ClipSpace::transform_from_canvas(self.canvas_size);
        self.recalculate_view_transform();
        self.schedule_draw();
    }

    fn schedule_draw(&mut self) {
        self.frame_scheduler.schedule().unwrap();
    }

    fn draw(&mut self, timestamp: Duration) {
        // Update translation in case of flick
        self.translation += self.translation_velocity * timestamp.as_secs_f32();
        // self.translation_velocity *= TRANSLATION_DRAG_FACTOR * timestamp.as_secs_f32();

        // Resize canvas if needed
        if self.canvas_needs_size_update {
            self.canvas_needs_size_update = false;
            self.canvas.set_width(self.canvas_size.width as u32);
            self.canvas.set_height(self.canvas_size.height as u32);
        }

        // Draw
        self.renderer.draw(&self.view_transform);

        // Schedule next draw if needed
        if !self
            .translation_velocity
            .approx_eq(&TRANSLATION_VELOCITY_ZERO)
        {
            self.schedule_draw();
        }
    }

    pub fn set_data(&mut self, graph: &Graph, active_state: BoardId) {
        // Create an arrangement from the graph data
        let arrangement = Arrangement::new(graph, active_state);

        // Upload the data to the GPU
        let vertices_array = unsafe { js_sys::Float32Array::view(&arrangement.points) };
        self.renderer.set_data(&vertices_array);

        // Store the content's size with padding applied
        self.content_size = ContentSpace::add_padding(Size2D::new(
            arrangement.width as f32,
            arrangement.height as f32,
        ));
        self.recalculate_view_transform();
        self.schedule_draw();
    }

    fn handle_pointer_event(&mut self, event: ControlEvent) {
        match event {
            ControlEvent::Down(_coordinates) => {
                // Cancel a flick if it was still going
                self.translation_velocity = Vector2D::zero();
            }
            ControlEvent::Move(delta_coordinates) => self.handle_translation(Vector2D::new(
                delta_coordinates.x as f32,
                -delta_coordinates.y as f32,
            )),
            ControlEvent::Up(velocity) => {
                // Store the current drag velocity so that the chart can be flicked
                self.translation_velocity = self
                    .canvas_to_clip
                    .transform_vector(Vector2D::new(velocity.x as f32, -velocity.y as f32));
            }
        }
    }

    fn _accumulate_zoom(&mut self, zoom_movement: f32, target_x: f32, target_y: f32) {
        let target_begin = self
            .canvas_to_clip
            .transform_vector(Vector2D::new(target_x, target_y));
        let transform_begin = ClipSpace::transform_from_content(
            self.canvas_size,
            self.content_size,
            self.zoom,
            self.translation,
        )
        .inverse()
        .unwrap();
        let _target_content = transform_begin.transform_vector(target_begin);

        // Convert zoom movement from canvas pixels to clip space delta
        let _zoom_movement_clip = self
            .canvas_to_clip
            .transform_vector(Vector2D::new(0.0, zoom_movement));

        // TODO(Menno 10.05.2025) Apply the zoom and readjust the translation so that the target x and y remain at the
        //  same content space point.
        self.recalculate_view_transform();
        self.schedule_draw();
    }

    fn handle_translation(&mut self, translation: Vector2D<f32, CanvasSpace>) {
        // TODO(Menno 04.05.2025) Clamp this translation
        self.translation += self.canvas_to_clip.transform_vector(translation);
        self.recalculate_view_transform();
        self.schedule_draw();
    }

    fn recalculate_view_transform(&mut self) {
        let transform = ClipSpace::transform_from_content(
            self.canvas_size,
            self.content_size,
            self.zoom,
            self.translation,
        );
        let [m11, m12, m21, m22, m31, m32] = transform.to_array();
        self.view_transform = [m11, m12, 0.0, m21, m22, 0.0, m31, m32, 1.0];
    }
}
