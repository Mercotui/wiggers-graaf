use crate::arrangement::Arrangement;
use glam::{Affine2, FloatExt, Mat3, Vec2};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation,
    WebGlVertexArrayObject,
};

const SCALE_SPEED_FACTOR: f32 = 1.0;

pub struct Renderer {
    canvas: HtmlCanvasElement,
    gl: WebGl2RenderingContext,
    shaders: WebGlProgram,
    // TODO(Menno 28.04.2025) These scale values could be converted to a (affine) transform matrix?
    canvas_to_gl_scale_x: f32,
    canvas_to_gl_scale_y: f32,
    scale: f32,
    translation: Vec2,
    should_update_view_transform: bool,
    view_transform_location: WebGlUniformLocation,
    vao: WebGlVertexArrayObject,
    vertex_count: i32,
}

fn get_canvas(canvas_id: &str) -> Result<HtmlCanvasElement, JsValue> {
    // Access DOM
    let document = web_sys::window()
        .ok_or(JsValue::from_str("Unable to access the window"))?
        .document()
        .ok_or(JsValue::from_str("Unable to access the DOM"))?;
    // Get canvas element from DOM
    document
        .get_element_by_id(canvas_id)
        .ok_or(JsValue::from_str(&format!(
            "Could not find canvas: {}",
            canvas_id
        )))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_x| {
            JsValue::from_str(&format!(
                "Element with ID {} does not appear to be a canvas",
                canvas_id
            ))
        })
}

fn create_context(canvas: &HtmlCanvasElement) -> Result<WebGl2RenderingContext, JsValue> {
    let gl: WebGl2RenderingContext = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;
    Ok(gl)
}

fn create_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, JsValue> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| JsValue::from_str("Unable to create shader object"))?;

    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(JsValue::from_str(
            &gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown error creating shader".into()),
        ))
    }
}

fn setup_shaders(gl: &WebGl2RenderingContext) -> Result<WebGlProgram, JsValue> {
    let vertex_shader_source = "
        uniform mat3 view_transform;
        attribute vec2 coordinates;
        attribute float point_size;
        attribute vec3 color;

        varying vec4 f_color;

        void main(void) {
            f_color = vec4(color.r, color.g, color.b, 1.0);
            vec3 transformed_vertex = view_transform * vec3(coordinates, 1.0);
            gl_Position = vec4(transformed_vertex, 1.0);
            gl_PointSize = point_size;
        }
        ";

    let fragment_shader_source = "
        precision mediump float;

        varying vec4 f_color;

        void main(void) {
            gl_FragColor = f_color;
        }
        ";

    let vertex_shader = create_shader(
        &gl,
        WebGl2RenderingContext::VERTEX_SHADER,
        vertex_shader_source,
    )
    .expect("Failed to compile vertex shader");
    let fragment_shader = create_shader(
        &gl,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        fragment_shader_source,
    )
    .expect("Failed to compile fragment shader");

    let shader_program = gl.create_program().unwrap();
    gl.attach_shader(&shader_program, &vertex_shader);
    gl.attach_shader(&shader_program, &fragment_shader);
    gl.link_program(&shader_program);

    if gl
        .get_program_parameter(&shader_program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        // Set the shader program as active.
        gl.use_program(Some(&shader_program));
        Ok(shader_program)
    } else {
        Err(JsValue::from_str(
            &gl.get_program_info_log(&shader_program)
                .unwrap_or_else(|| "Unknown error linking program".into()),
        ))
    }
}

fn setup_vao(gl: &WebGl2RenderingContext, shader_program: &WebGlProgram) -> WebGlVertexArrayObject {
    let vertex_array_object = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(&vertex_array_object));

    let vertex_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));

    let coordinates_location: u32 = gl.get_attrib_location(&shader_program, "coordinates") as u32;
    gl.vertex_attrib_pointer_with_i32(
        coordinates_location,
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        6 * 4,
        0,
    );
    gl.enable_vertex_attrib_array(coordinates_location);

    let point_size_location: u32 = gl.get_attrib_location(&shader_program, "point_size") as u32;
    gl.vertex_attrib_pointer_with_i32(
        point_size_location,
        1,
        WebGl2RenderingContext::FLOAT,
        false,
        6 * 4,
        2 * 4,
    );
    gl.enable_vertex_attrib_array(point_size_location);

    let color_location: u32 = gl.get_attrib_location(&shader_program, "color") as u32;
    gl.vertex_attrib_pointer_with_i32(
        color_location,
        3,
        WebGl2RenderingContext::FLOAT,
        false,
        6 * 4,
        3 * 4,
    );
    gl.enable_vertex_attrib_array(color_location);

    gl.bind_vertex_array(None);
    vertex_array_object
}

impl Renderer {
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let canvas: HtmlCanvasElement = get_canvas(canvas_id)?;
        let gl: WebGl2RenderingContext = create_context(&canvas)?;
        let shaders: WebGlProgram = setup_shaders(&gl)?;
        let view_transform_location: WebGlUniformLocation = gl
            .get_uniform_location(&shaders, "view_transform")
            .ok_or(JsValue::from_str(
                "Can't retrieve view_transform uniform location from shaders",
            ))?;
        let vao: WebGlVertexArrayObject = setup_vao(&gl, &shaders);

        // Create instance
        Ok(Renderer {
            canvas,
            gl,
            shaders,
            canvas_to_gl_scale_x: 0.0,
            canvas_to_gl_scale_y: 0.0,
            scale: 1.0,
            // TODO(Menno 28.04.2025) This offset value seems arbitrary, it might need to be dynamic
            translation: Vec2 { x: -0.97, y: -0.97 },
            should_update_view_transform: true,
            view_transform_location,
            vao,
            vertex_count: 0,
        })
    }

    pub fn set_data(&mut self, arrangement: &Arrangement) {
        self.gl.bind_vertex_array(Some(&self.vao));

        // TODO(Menno 24.04.2025) Double check if this data needs to be backed for the lifetime of
        //  the buffer, or only for the buffer_data call
        let vertices_array = unsafe { js_sys::Float32Array::view(&arrangement.points) };

        self.gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vertices_array,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        self.vertex_count = (arrangement.points.len() / 6) as i32;
        self.gl.bind_vertex_array(None);
    }

    pub fn resize(&mut self) {
        let width = self.canvas.client_width();
        let height = self.canvas.client_height();
        self.canvas_to_gl_scale_x = 2.0 / width as f32;
        self.canvas_to_gl_scale_y = 2.0 / height as f32;
        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);
        self.gl.viewport(0, 0, width, height);
    }

    fn update_view_transform(&mut self) {
        let view_transform: Mat3 = Mat3::from_scale_angle_translation(
            // TODO(Menno 28.04.2025) this scale should be adapted automatically by the layout size
            Vec2 {
                x: self.scale * 0.019,
                y: self.scale * 0.019,
            },
            0.0,
            self.translation,
        );

        self.gl.use_program(Some(&self.shaders));
        self.gl.uniform_matrix3fv_with_f32_array(
            Some(&self.view_transform_location),
            false,
            &view_transform.to_cols_array(),
        );
        self.gl.use_program(None);
        self.should_update_view_transform = false;
    }

    pub fn accumulate_scale(&mut self, delta_scale: f32) {
        let scaled_delta_scale = delta_scale * self.canvas_to_gl_scale_y * SCALE_SPEED_FACTOR;
        self.scale = (self.scale + scaled_delta_scale).clamp(1.0, 5.0);
        self.should_update_view_transform = true;
    }

    pub fn accumulate_translation(&mut self, delta_x: f32, delta_y: f32) {
        let scaled_delta_x = delta_x * self.canvas_to_gl_scale_x;
        let scaled_delta_y = delta_y * self.canvas_to_gl_scale_y;
        // TODO(Menno 28.04.2025) The translation should be applied in a different coordinate system
        self.translation.x = (self.translation.x + scaled_delta_x).clamp(-2.0, 0.0);
        self.translation.y = (self.translation.y + scaled_delta_y).clamp(-2.0, 0.0);
        self.should_update_view_transform = true;
    }

    pub fn draw(&mut self) {
        if self.should_update_view_transform {
            self.update_view_transform();
        }

        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        self.gl.use_program(Some(&self.shaders));
        self.gl.bind_vertex_array(Some(&self.vao));
        self.gl
            .draw_arrays(WebGl2RenderingContext::POINTS, 0, self.vertex_count);
        self.gl.bind_vertex_array(None);
        self.gl.use_program(None);
    }
}
