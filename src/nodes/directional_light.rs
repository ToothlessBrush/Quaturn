//! Directional light casts light on a scene from a single direction, like the sun. It is used to simulate sunlight in a scene. It is a type of light that is infinitely far away and has no attenuation. It is defined by a direction and a color. It can also cast shadows using a shadow map.
//!
//! ## Usage
//! add this to the node tree to add a directional light to the scene.
//!
//! ## Example
//! ```rust
//! use quaturn::Engine;
//! use quaturn::glm;
//! use quaturn::game_context::nodes::directional_light::DirectionalLight;
//!
//! let mut engine = Engine::init("Example", 800, 600);
//!
//! engine.context.nodes.add("directional_light", DirectionalLight::new(
//!     glm::vec3(1.0, 1.0, 1.0),
//!     glm::vec3(1.0, 1.0, 1.0),
//!     1.0,
//!     100.0,
//!     1024,
//! ));
//!
//! //engine.begin();
//! ```

use crate::components::{EventReceiver, NodeTransform};
use crate::context::scene::{Drawable, Node, Scene};
use crate::nodes::Model;
use crate::renderer::shader::Shader;
use crate::renderer::shadow_map::ShadowMap;
use crate::utils::color::Color;
use nalgebra_glm as glm;


use super::{NodeBuilder};

/// Directional light casts light on a scene from a single direction, like the sun. It is used to simulate sunlight in a scene. It is a type of light that is infinitely far away and has no attenuation. It is defined by a direction and a color. It can also cast shadows using a shadow map.
///
/// ## Usage
/// add this to the node tree to add a directional light to the scene.
#[derive(Clone)]
pub struct DirectionalLight {
    /// The transform of the directional light.
    transform: NodeTransform,
    /// The children of the directional light.
    children: Scene,

    events: EventReceiver,
    /// The color of the directional light.
    pub color: glm::Vec4,
    /// The intensity of the directional light.
    pub intensity: f32,
    /// The distance of the shadow cast by the directional light.
    shadow_distance: f32,
    /// The projection matrix of the shadow cast by the directional light.
    shadow_projections: glm::Mat4,
    /// The light space matrix of the shadow cast by the directional light.
    light_space_matrix: glm::Mat4,
    /// The shadow map of the directional light.
    shadow_map: ShadowMap,
}

impl Node for DirectionalLight {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&self) -> &Scene {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut crate::context::scene::Scene {
        &mut self.children
    }

    fn get_events(&mut self) -> &mut EventReceiver {
        &mut self.events
    }
}

impl DirectionalLight {
    /// creates a new directional light with the given direction, color, intensity, shadow distance, and shadow resolution.
    ///
    /// # Arguments
    /// - `direction` - The direction of the directional light.
    /// - `color` - The color of the directional light.
    /// - `intensity` - The intensity of the directional light.
    /// - `shadow_distance` - The distance of the shadow cast by the directional light.
    /// - `shadow_resolution` - The resolution of the shadow map of the directional light.
    ///
    /// # Returns
    /// The new directional light.
    pub fn new(
        // direction: glm::Vec3,
        // color: glm::Vec3,
        shadow_distance: f32,
        shadow_resolution: u32,
    ) -> DirectionalLight {
        let shadow_projections = glm::ortho(
            -shadow_distance / 2.0,
            shadow_distance / 2.0,
            -shadow_distance / 2.0,
            shadow_distance / 2.0,
            0.1,
            shadow_distance,
        );

        let direction = glm::vec3(0.0, 0.0, 1.0);
        let light_direction = glm::normalize(&direction);
        let light_position = light_direction * (shadow_distance / 2.0);
        let light_view = glm::look_at(
            &light_position,
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 1.0, 0.0),
        );
        let light_space_matrix = shadow_projections * light_view;

        let shadow_shader = Shader::from_slice(
            include_str!("../../res/shaders/depthShader/depthShader.vert"),
            include_str!("../../res/shaders/depthShader/depthShader.frag"),
            None,
        );

        let shadow_map = ShadowMap::gen_map(
            shadow_resolution as i32,
            shadow_resolution as i32,
            shadow_shader,
        );

        // calculate the rotation quaternion from the orientation vector
        let direction = glm::normalize(&direction);
        let reference = glm::vec3(0.0, 0.0, 1.0);

        // Handle parallel and anti-parallel cases
        let rotation_quat = if glm::dot(&direction, &reference).abs() > 0.9999 {
            if direction.z > 0.0 {
                glm::quat_identity() // No rotation needed
            } else {
                glm::quat_angle_axis(glm::pi::<f32>(), &glm::vec3(1.0, 0.0, 0.0))
                // 180-degree rotation
            }
        } else {
            let rotation_axis = glm::cross(&reference, &direction).normalize();
            let rotation_angle = glm::dot(&reference, &direction).acos();
            glm::quat_angle_axis(rotation_angle, &rotation_axis)
        };

        // println!("Directional Light Rotation: {:?}", rotation_quat);
        // println!("Directional Light Direction: {:?}", direction);

        let check_direction = glm::quat_rotate_vec3(&rotation_quat, &reference);
        // println!("Directional Light Check Direction: {:?}", check_direction);

        // Use a tolerance-based assertion for floating-point comparisons
        assert!((check_direction - direction).magnitude() < 1e-5);

        DirectionalLight {
            transform: NodeTransform::new(
                glm::vec3(0.0, 0.0, 0.0),
                rotation_quat,
                glm::vec3(1.0, 1.0, 1.0),
            ),
            children: Scene::new(),
            events: EventReceiver::new(),
            intensity: 1.0,
            color: Color::from_normalized(1.0, 1.0, 1.0, 1.0).into(),
            shadow_distance,
            shadow_projections,
            light_space_matrix,
            shadow_map,
        }
    }

    /// direction the lights coming from
    pub fn set_direction(&mut self, direction: glm::Vec3) -> &mut Self {
        // update projection
        let light_direction = glm::normalize(&direction);
        let light_position = light_direction * (self.shadow_distance / 2.0);
        let light_view = glm::look_at(
            &light_position,
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 1.0, 0.0),
        );
        self.light_space_matrix = self.shadow_projections * light_view;

        let reference = glm::vec3(0.0, 0.0, 1.0);

        // update rotation
        let rotation_quat = if glm::dot(&direction, &reference).abs() > 0.9999 {
            if direction.z > 0.0 {
                glm::quat_identity() // No rotation needed
            } else {
                glm::quat_angle_axis(glm::pi::<f32>(), &glm::vec3(1.0, 0.0, 0.0))
                // 180-degree rotation
            }
        } else {
            let rotation_axis = glm::cross(&reference, &direction).normalize();
            let rotation_angle = glm::dot(&reference, &direction).acos();
            glm::quat_angle_axis(rotation_angle, &rotation_axis)
        };

        self.transform.set_rotation(rotation_quat);

        self
    }

    /// sets the color of the light
    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color.into();
        self
    }

    pub fn set_intensity(&mut self, intensity: f32) -> &mut Self {
        self.intensity = intensity;
        self
    }
    /// renders the shadow map of the directional light
    ///
    /// # Arguments
    /// - `models` - The models to render the shadow map for.
    pub fn render_shadow_map(&mut self, root_nodes: Vec<&mut Box<dyn Node>>) {
        //let nodes = *root_nodes;

        let depth_shader = self.shadow_map.prepare_shadow_map();
        depth_shader.set_uniform("u_lightSpaceMatrix", self.light_space_matrix);

        for node in root_nodes {
            Self::draw_node_shadow(depth_shader, node, NodeTransform::default());
        }

        self.shadow_map.finish_shadow_map();
    }

    fn draw_node_shadow(
        shader: &mut Shader,
        node: &mut Box<dyn Node>,
        parent_transform: NodeTransform,
    ) {
        let world_transfrom = parent_transform + *node.get_transform();
        if let Some(model) = node.as_any_mut().downcast_mut::<Model>() {
            model.draw_shadow(shader, world_transfrom);
        }

        for child in node.get_children_mut() {
            Self::draw_node_shadow(shader, child.1, world_transfrom);
        }
    }

    /// binds the shadow map and light space matrix to the active shader for shaders that need shadow mapping
    ///
    /// # Arguments
    /// - `shader` - The shader to bind the shadow map and light space matrix to.
    pub fn bind_uniforms(&self, shader: &mut Shader) {
        let direction = glm::quat_rotate_vec3(&self.transform.rotation, &glm::vec3(0.0, 0.0, 1.0));
        // Bind shadow map and light space matrix to the active shader
        shader.bind();
        shader.set_uniform("u_lightSpaceMatrix", self.light_space_matrix);
        //shader.set_uniform1f("u_farShadowPlane", self.shadow_distance);
        shader.set_uniform("u_directLightDirection", direction);
        // Bind the shadow map texture to texture unit 2 (example)
        self.shadow_map.bind_shadow_map(shader, "shadowMap", 2);
    }

    /// get the far plane of the shadow cast by the directional light
    pub fn get_far_plane(&self) -> f32 {
        self.shadow_distance
    }

    /// set the far plane of the shadow cast by the directional light
    pub fn set_far_plane(&mut self, distance: f32) {
        self.shadow_distance = distance;
        self.shadow_projections = glm::ortho(
            -self.shadow_distance / 2.0,
            self.shadow_distance / 2.0,
            -self.shadow_distance / 2.0,
            self.shadow_distance / 2.0,
            0.1,
            self.shadow_distance,
        );
        let light_direction =
            glm::quat_rotate_vec3(&self.transform.rotation, &glm::vec3(0.0, 0.0, 1.0));
        let light_position = light_direction * (self.shadow_distance / 2.0); //self.shadow_distance;
        let light_view = glm::look_at(
            &light_position,
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 1.0, 0.0),
        );
        self.light_space_matrix = self.shadow_projections * light_view;
    }
}

pub trait DirectLightBuilder {
    fn set_direction(&mut self, direction: glm::Vec3) -> &mut Self;
    fn set_intensity(&mut self, intensity: f32) -> &mut Self;
    fn set_color(&mut self, color: Color) -> &mut Self;
    fn set_far_plane(&mut self, far: f32) -> &mut Self;
}

impl DirectLightBuilder for NodeBuilder<DirectionalLight> {
    fn set_direction(&mut self, direction: nalgebra_glm::Vec3) -> &mut Self {
        self.node.set_direction(direction);
        self
    }
    fn set_color(&mut self, color: Color) -> &mut Self {
        self.node.set_color(color);
        self
    }
    fn set_intensity(&mut self, intensity: f32) -> &mut Self {
        self.node.set_intensity(intensity);
        self
    }
    fn set_far_plane(&mut self, far: f32) -> &mut Self {
        self.node.set_far_plane(far);
        self
    }
}
