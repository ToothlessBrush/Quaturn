extern crate nalgebra_glm as glm;
extern crate stb_image;

use glfw::Context;

//pub mod egui_backend;
pub mod graphics;
pub mod utils;

use graphics::buffers::{index_buffer, vertex_array, vertex_buffer, vertex_buffer_layout};
use graphics::renderer::{debug_message_callback, Renderer};
use graphics::shader;
use graphics::texture;
use utils::fps_manager::FPSManager;
use utils::rgb_color::Color;

fn main() {
    use glfw::fail_on_errors;
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    //create window with gl context
    let (mut window, events) = glfw
        .create_window(
            960,
            540,
            "Top 10 Windows Ever Made",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    //window.make_current();
    window.set_key_polling(true);

    //init gl and load the opengl function pointers
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        println!(
            "{}",
            std::ffi::CStr::from_ptr(gl::GetString(gl::VERSION) as *const i8)
                .to_str()
                .unwrap()
        );
    }

    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(Some(debug_message_callback), std::ptr::null());
    }

    let positions: [f32; 16] = [
        -50.0, -50.0, 0.0, 0.0, 50.0, -50.0, 1.0, 0.0, 50.0, 50.0, 1.0, 1.0, -50.0, 50.0, 0.0, 1.0,
    ];

    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let va = vertex_array::VertexArray::new();
    va.bind();

    let vb = vertex_buffer::VertexBuffer::new(&positions);

    let mut layout = vertex_buffer_layout::VertexBufferLayout::new();
    layout.push::<f32>(2);
    layout.push::<f32>(2);
    va.add_buffer(&vb, &layout);

    let ib = index_buffer::IndexBuffer::new(&indices);

    let proj: glm::Mat4 = glm::ortho(0.0, 960.0, 0.0, 540.0, -1.0, 1.0); //orthographic projection converts the pixel space to normalized device coordinates
    let transform: glm::Mat4 = glm::translate(&glm::Mat4::identity(), &glm::vec3(0.0, 0.0, 0.0)); //default translation

    let mut mvp = proj * transform;

    let mut shader = shader::Shader::new("res/shaders");
    shader.bind();
    shader.set_uniform4f("u_Color", 0.2, 0.8, 1.0, 1.0);

    shader.set_uniform_mat4f("u_MVP", &mvp);

    let texture = texture::Texture::new("res/textures/mogcat.png");
    texture.bind(0);
    shader.set_uniform1i("u_Texture", 0);

    va.unbind();
    vb.unbind();
    ib.unbind();
    shader.unbind();

    //this is where shit goes down\

    let renderer = Renderer::new();

    let translation_a: glm::Vec3 = glm::Vec3::new(100.0, 100.0, 0.0);
    let translation_b: glm::Vec3 = glm::Vec3::new(400.0, 100.0, 0.0);
    //let mut colors = Color::new(1.0, 0.0, 0.0);

    // Create an FPS counter
    let mut fps_counter = FPSManager::new();

    // Loop until the user closes the window
    while !window.should_close() {
        // Update the FPS counter
        fps_counter.update(|fps| {
            window.set_title(&format!("Top 10 Windows Ever Made | FPS: {}", fps));
        });

        // Render here
        renderer.clear();

        //bind shader program
        shader.bind();
        //shader.set_uniform4f("u_Color", colors.r, colors.g, colors.b, 1.0);
        // Draw the triangles
        //sets the mvp matrix to the position of a then b to render image twice
        {
            let model: glm::Mat4 = glm::translate(&glm::Mat4::identity(), &translation_a);
            mvp = proj * transform * model;
            shader.set_uniform_mat4f("u_MVP", &mvp);
            renderer.draw(&va, &ib, &shader);
        }

        {
            let model: glm::Mat4 = glm::translate(&glm::Mat4::identity(), &translation_b);
            mvp = proj * transform * model;
            shader.set_uniform_mat4f("u_MVP", &mvp);
            renderer.draw(&va, &ib, &shader);
        }

        for (_, event) in glfw::flush_messages(&events) {
            println!("{:?}", event);
            match event {
                glfw::WindowEvent::Close => window.set_should_close(true),
                //move left/right based on a/d keys
                _ => {}
            }
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}
