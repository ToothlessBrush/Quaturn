//pub mod engine;
use engine::game_context::nodes::{camera::Camera3D, model::Model, ui::UI};
use engine::game_context::GameContext;
use engine::renderer::shader::Shader;
use engine::Engine;
use quaturn::{egui, engine, glfw, glm};

//use engine::Engine;

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

fn main() {
    let mut engine = Engine::init("top 10 windows", WINDOW_WIDTH, WINDOW_HEIGHT);

    engine.set_clear_color(1.0, 1.0, 1.0, 1.0);

    let mut cursor_locked = false;

    let toggle_cursor_lock = |context: &mut GameContext, lock: bool| {
        context.lock_cursor(lock);
    };

    engine
        .context
        .nodes
        .add_model("model", Model::new("res/scenes/shadow_test/scene.gltf"))
        //.rotate_euler_xyz(glm::Vec3::new(-90.0, 0.0, 0.0)) // y+ is up
        .define_ready(|_model| {
            //ran before the first frame
            println!("model ready");
        })
        .define_behavior(|model, context| {
            //ran every frame
            //println!("model behavior");
        });

    let camera_pos = glm::vec3(20.0, 20.0, 20.0);

    engine
        .context
        .nodes
        .add_camera(
            "camera",
            Camera3D::new(
                camera_pos,
                (glm::vec3(0.0, 2.0, 0.0) - camera_pos).normalize(),
                0.78539,
                WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
                0.1,
                10000.0,
            ),
        )
        .define_ready(|_camera| {
            //ran before the first frame
            println!("camera ready");
        })
        .define_behavior(move |camera, context| {
            if cursor_locked {
                camera.take_input(&context.input, context.frame.time_delta.as_secs_f32());
            }

            if context.input.keys.contains(&glfw::Key::Escape) {
                context.window.set_should_close(true);
            }

            if context
                .input
                .mouse_button_just_pressed
                .contains(&glfw::MouseButton::Button3)
            {
                toggle_cursor_lock(context, !cursor_locked);
                cursor_locked = !cursor_locked;
            }
        });

    let shader = engine
        .context
        .nodes
        .add_shader("default", Shader::new("res/shaders/default"));

    shader.bind();
    shader.set_uniform4f("lightColor", 1.0, 1.0, 1.0, 1.0);

    let ui = UI::init(&mut engine.context.window);
    engine
        .context
        .nodes
        .add_ui("debug_panel", ui)
        .define_ui(move |ctx, context| {
            //engine borrowed here

            //extract camera info

            let camera: &mut Camera3D = context.nodes.get_camera("camera");
            let (mut camera_pos_x, mut camera_pos_y, mut camera_pos_z) = (
                camera.get_position().x,
                camera.get_position().y,
                camera.get_position().z,
            );

            let (mut camera_rotation_x, mut camera_rotation_y, mut camera_rotation_z) = (
                camera.get_orientation_angles().x,
                camera.get_orientation_angles().y,
                camera.get_orientation_angles().z,
            );

            //ui to be drawn every frame
            egui::Window::new("Debug Panel").show(ctx, |ui| {
                ui.label("Hello World!");
                if ui.button("print").clicked() {
                    println!("Hello World!");
                }
                ui.label("Camera Position");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.add(egui::DragValue::new(&mut camera_pos_x));
                    ui.label("Y:");
                    ui.add(egui::DragValue::new(&mut camera_pos_y));
                    ui.label("Z:");
                    ui.add(egui::DragValue::new(&mut camera_pos_z));
                });
                ui.label("Camera Rotation");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.add(egui::DragValue::new(&mut camera_rotation_x));
                    ui.label("Y:");
                    ui.add(egui::DragValue::new(&mut camera_rotation_y));
                    ui.label("Z:");
                    ui.add(egui::DragValue::new(&mut camera_rotation_z));
                });
                ui.add(egui::Slider::new(&mut camera.move_speed, 0.0..=1000.0).text("Move Speed"));
            });

            //reassign camera position and rotation from ui
            camera.set_position(glm::vec3(camera_pos_x, camera_pos_y, camera_pos_z));
            camera.set_orientation_angles(glm::vec3(
                camera_rotation_x,
                camera_rotation_y,
                camera_rotation_z,
            ));
        });

    engine.begin();
}
