// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::views::graph_view::controls::ControlEvent;
use crate::views::utils::CanvasSpace;
use euclid::{Scale, Size2D, Transform2D, Vector2D};

/// This represents the view's content coordinate space, dynamic axes depending on the content size
pub struct ContentSpace;

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

/// The decay of the drag velocity, in clipspace per second
const TRANSLATION_DRAG: f32 = 0.5;

/// Zero velocity
const TRANSLATION_VELOCITY_ZERO: Vector2D<f32, ClipSpace> = Vector2D::new(0.0, 0.0);

/// The velocity of a user scrolling
const _ZOOM_SPEED: Scale<f32, ClipSpace, ClipSpace> = Scale::new(1.0);

/// The minimum zoom level, this fits the whole contents into the clip-space, with some padding.
const _ZOOM_MINIMUM: Scale<f32, ClipSpace, ClipSpace> = Scale::new(1.0);

/// The maximum zoom level
const _ZOOM_MAXIMUM: Scale<f32, ClipSpace, ClipSpace> = Scale::new(5.0);

pub struct FlickableCamera {
    canvas_size: Size2D<f32, CanvasSpace>,
    content_size: Size2D<f32, ContentSpace>,
    canvas_to_clip: Transform2D<f32, CanvasSpace, ClipSpace>,
    zoom: Scale<f32, ClipSpace, ClipSpace>,
    translation: Vector2D<f32, ClipSpace>,
    translation_velocity: Vector2D<f32, ClipSpace>,
}

impl FlickableCamera {
    pub fn new() -> Self {
        Self {
            canvas_size: Size2D::zero(),
            content_size: Size2D::zero(),
            canvas_to_clip: Transform2D::identity(),
            zoom: Scale::identity(),
            translation: ClipSpace::CLIP_SPACE_OFFSET,
            translation_velocity: Vector2D::zero(),
        }
    }

    pub fn handle_pointer_event(&mut self, event: ControlEvent) {
        match event {
            ControlEvent::Down(_coordinates) => {
                // Cancel a flick if it was still going
                self.translation_velocity = Vector2D::zero();
            }
            ControlEvent::Move(delta_coordinates) => {
                // TODO(Menno 04.05.2025) Clamp this translation
                self.accumulate_translation(self.canvas_to_clip.transform_vector(Vector2D::new(
                    delta_coordinates.x as f32,
                    -delta_coordinates.y as f32,
                )));
            }
            ControlEvent::Up(velocity) => {
                // Store the current drag velocity so that the chart can be flicked
                self.translation_velocity = self
                    .canvas_to_clip
                    .transform_vector(Vector2D::new(velocity.x as f32, velocity.y as f32));
            }
        }
    }

    pub fn resize_canvas(&mut self, size: Size2D<f32, CanvasSpace>) {
        self.canvas_to_clip = ClipSpace::transform_from_canvas(size);
    }

    pub fn resize_content(&mut self, size: Size2D<f32, ContentSpace>) {
        // Store the content's size with padding applied
        self.content_size = ContentSpace::add_padding(size);
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
    }

    fn accumulate_translation(&mut self, translation: Vector2D<f32, ClipSpace>) {
        // TODO(Menno 04.05.2025) Clamp this translation
        self.translation += translation;
    }

    pub fn view_transform(&mut self) -> [f32; 9] {
        let transform = ClipSpace::transform_from_content(
            self.canvas_size,
            self.content_size,
            self.zoom,
            self.translation,
        );
        let [m11, m12, m21, m22, m31, m32] = transform.to_array();
        [m11, m12, 0.0, m21, m22, 0.0, m31, m32, 1.0]
    }
}
