use quaturn::context::scene::Scene;
use quaturn::glfw::PWindow;
use quaturn::math;
use quaturn::nodes::{
    model::Primitive, Camera3D, Camera3DBuilder, DirectionalLight, DirectionalLightBuilder, Model,
    ModelBuilder, NodeBuilder,
};
use quaturn::utils::color;

/// get the screen resolution
use std::f32::consts::FRAC_PI_4;

pub struct MainScene;

impl MainScene {
    pub fn build(window: &PWindow) -> Scene {
        let mut scene = Scene::default();

        // add a pyramid node
        scene
            .add(
                "pyramid", // name
                // creates a NodeBuilder for a pyramid Model
                NodeBuilder::<Model>::create_primitive(Primitive::Pyramid)
                    // make it spin to demonstrate udate behavior
                    .on(quaturn::components::Event::Update, |model, ctx| {
                        model.transform.rotate_euler_xyz(math::vec3(
                            0.0,
                            90.0 * ctx.frame.time_delta.as_secs_f32(),
                            0.0,
                        ));
                    })
                    .build(),
            )
            .expect("failed to add pyramid");

        // add a ground to demonstrate shadows
        scene
            .add(
                "ground",
                NodeBuilder::<Model>::create_primitive(Primitive::Plane)
                    .with_position(math::vec3(0.0, -2.0, 0.0))
                    .with_scale_factor(10.0)
                    .build(),
            )
            .expect("faile to build ground");

        scene
            .add(
                "camera",
                NodeBuilder::<Camera3D>::create(
                    window.get_size(),
                    FRAC_PI_4, // fov in radians
                )
                // offset it back a bit
                .with_position(math::vec3(1.0, 1.0, -10.0))
                // look forward towards the scene center and slightly downward
                .set_orientation_vector(math::vec3(0.0, -0.2, 1.0))
                .build(),
            )
            .expect("failed to add camera");

        // add a sun to demonstrate light
        scene
            .add(
                "sun",
                NodeBuilder::<DirectionalLight>::create(
                    math::vec3(1.0, 1.0, -1.0),
                    color::WHITE.into(),
                )
                .build(),
            )
            .expect("failed to add Light");

        scene
    }
}
