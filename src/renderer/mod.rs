use crate::arrangement::Arrangement;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlVertexArrayObject,
};

pub struct Renderer {
    canvas: HtmlCanvasElement,
    gl: WebGl2RenderingContext,
    shaders: WebGlProgram,
    vao: WebGlVertexArrayObject,
    vertex_count: i32,
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
        attribute vec2 coordinates;
        attribute float point_size;
        attribute vec3 color;

        varying vec4 f_color;

        void main(void) {
            f_color = vec4(color.r, color.g, color.b, 1.0);
            gl_Position = vec4(coordinates, 0.0, 1.0);
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
        return Err(JsValue::from_str(
            &gl.get_program_info_log(&shader_program)
                .unwrap_or_else(|| "Unknown error linking program".into()),
        ));
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
        // Get canvas element
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id(canvas_id)
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()?;

        // Setup OpenGL objects
        let gl: WebGl2RenderingContext = create_context(&canvas)?;
        let shaders: WebGlProgram = setup_shaders(&gl)?;
        let vao: WebGlVertexArrayObject = setup_vao(&gl, &shaders);

        // Create instance
        Ok(Renderer {
            canvas,
            gl,
            shaders,
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
    }

    pub fn resize(&mut self) {
        let width = self.canvas.client_width();
        let height = self.canvas.client_height();
        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);
        self.gl.viewport(0, 0, width, height);
    }

    pub fn draw(&mut self) {
        self.gl.use_program(Some(&self.shaders));
        self.gl.bind_vertex_array(Some(&self.vao));
        self.gl
            .draw_arrays(WebGl2RenderingContext::POINTS, 0, self.vertex_count);
    }
}
