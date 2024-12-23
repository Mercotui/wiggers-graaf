use crate::arrangement::Arrangement;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

pub fn init_webgl_context(canvas_id: &str) -> Result<WebGlRenderingContext, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(canvas_id).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let gl: WebGlRenderingContext = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()
        .unwrap();

    gl.viewport(
        0,
        0,
        canvas.width().try_into().unwrap(),
        canvas.height().try_into().unwrap(),
    );

    Ok(gl)
}

pub fn create_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, JsValue> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| JsValue::from_str("Unable to create shader object"))?;

    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
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

pub fn setup_shaders(gl: &WebGlRenderingContext) -> Result<WebGlProgram, JsValue> {
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
        WebGlRenderingContext::VERTEX_SHADER,
        vertex_shader_source,
    )
    .expect("Failed to compile vertex shader");
    let fragment_shader = create_shader(
        &gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        fragment_shader_source,
    )
    .expect("Failed to compile fragment shader");

    let shader_program = gl.create_program().unwrap();
    gl.attach_shader(&shader_program, &vertex_shader);
    gl.attach_shader(&shader_program, &fragment_shader);
    gl.link_program(&shader_program);

    if gl
        .get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS)
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

pub fn setup_vertices(
    gl: &WebGlRenderingContext,
    vertice_data: &[f32],
    shader_program: &WebGlProgram,
) {
    let vertices_array = unsafe { js_sys::Float32Array::view(&vertice_data) };

    let vertex_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &vertices_array,
        WebGlRenderingContext::DYNAMIC_DRAW,
    );

    let coordinates_location: u32 = gl.get_attrib_location(&shader_program, "coordinates") as u32;
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.vertex_attrib_pointer_with_i32(
        coordinates_location,
        2,
        WebGlRenderingContext::FLOAT,
        false,
        6 * 4,
        0,
    );
    gl.enable_vertex_attrib_array(coordinates_location);

    let point_size_location: u32 = gl.get_attrib_location(&shader_program, "point_size") as u32;
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.vertex_attrib_pointer_with_i32(
        point_size_location,
        1,
        WebGlRenderingContext::FLOAT,
        false,
        6 * 4,
        2 * 4,
    );
    gl.enable_vertex_attrib_array(point_size_location);

    let color_location: u32 = gl.get_attrib_location(&shader_program, "color") as u32;
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.vertex_attrib_pointer_with_i32(
        color_location,
        3,
        WebGlRenderingContext::FLOAT,
        false,
        6 * 4,
        3 * 4,
    );
    gl.enable_vertex_attrib_array(color_location);
}

pub fn draw(canvas_id: &str, arrangement: &Arrangement) -> Result<WebGlRenderingContext, JsValue> {
    let gl: WebGlRenderingContext = init_webgl_context(canvas_id)?;
    let shader_program: WebGlProgram = setup_shaders(&gl)?;

    setup_vertices(&gl, &arrangement.points, &shader_program);

    gl.draw_arrays(
        WebGlRenderingContext::POINTS,
        0,
        (arrangement.points.len() / 6) as i32,
    );

    Ok(gl)
}
