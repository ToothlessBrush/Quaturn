use super::nodes::{
    camera::Camera3D, directional_light::DirectionalLight, empty::Empty, model::Model, ui::UI,
};
use crate::engine::renderer::shader::Shader;
use egui_gl_glfw::egui::util::id_type_map::SerializableAny;
use nalgebra_glm::{self as glm, Mat4, Vec3};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub trait Ready: Node {
    fn ready(&mut self);
}

pub trait Behavior: Node {
    fn behavior(&mut self, context: &mut super::GameContext);
}

#[derive(Debug, Clone)]
pub struct NodeTransform {
    pub position: Vec3,
    pub rotation: glm::Quat,
    pub scale: Vec3,
    pub matrix: Mat4,
}

impl Default for NodeTransform {
    fn default() -> Self {
        let mut transform = Self {
            position: glm::vec3(0.0, 0.0, 0.0),
            rotation: glm::quat_identity(),
            scale: glm::vec3(1.0, 1.0, 1.0),
            matrix: glm::identity(),
        };
        transform.update_matrix();
        transform
    }
}

impl PartialEq for NodeTransform {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.rotation == other.rotation
            && self.scale == other.scale
            && self.matrix == other.matrix
    }
}

impl NodeTransform {
    pub fn new(position: Vec3, rotation: glm::Quat, scale: Vec3) -> Self {
        let mut transform = Self {
            position,
            rotation,
            scale,
            matrix: glm::identity(),
        };
        transform.update_matrix();
        transform
    }

    fn update_matrix(&mut self) {
        self.matrix = glm::translation(&self.position)
            * glm::quat_to_mat4(&self.rotation)
            * glm::scaling(&self.scale);
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) -> &mut Self {
        self.position = position;
        self.update_matrix();
        self
    }

    pub fn get_rotation(&self) -> glm::Quat {
        self.rotation
    }

    pub fn get_rotation_euler_xyz(&self) -> Vec3 {
        glm::quat_euler_angles(&self.rotation)
    }

    pub fn set_rotation(&mut self, rotation: glm::Quat) -> &mut Self {
        self.rotation = rotation;
        self.update_matrix();
        self
    }

    pub fn set_euler_xyz(&mut self, degrees: Vec3) -> &mut Self {
        let radians = glm::radians(&degrees);
        self.rotation = glm::quat_angle_axis(radians.x, &glm::vec3(1.0, 0.0, 0.0))
            * glm::quat_angle_axis(radians.y, &glm::vec3(0.0, 1.0, 0.0))
            * glm::quat_angle_axis(radians.z, &glm::vec3(0.0, 0.0, 1.0));
        self.update_matrix();
        self
    }

    pub fn get_scale(&self) -> Vec3 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: Vec3) -> &mut Self {
        self.scale = scale;
        self.update_matrix();
        self
    }

    pub fn get_forward_vector(&self) -> Vec3 {
        glm::quat_rotate_vec3(&self.rotation, &glm::vec3(0.0, 0.0, 1.0))
    }

    pub fn get_right_vector(&self) -> Vec3 {
        glm::quat_rotate_vec3(&self.rotation, &glm::vec3(1.0, 0.0, 0.0))
    }

    pub fn get_up_vector(&self) -> Vec3 {
        glm::quat_rotate_vec3(&self.rotation, &glm::vec3(0.0, 1.0, 0.0))
    }

    pub fn scale(&mut self, scale: Vec3) -> &mut Self {
        self.scale.x *= scale.x;
        self.scale.y *= scale.y;
        self.scale.z *= scale.z;
        self.update_matrix();
        self
    }

    pub fn translate(&mut self, translation: Vec3) -> &mut Self {
        self.position += translation;
        self.update_matrix();
        self
    }

    pub fn rotate(&mut self, axis: glm::Vec3, degrees: f32) -> &mut Self {
        self.rotation =
            glm::quat_angle_axis(glm::radians(&glm::vec1(degrees)).x, &axis) * self.rotation;
        self.update_matrix();
        self
    }

    pub fn rotate_euler_xyz(&mut self, degrees: Vec3) -> &mut Self {
        let radians = glm::radians(&degrees);
        self.rotation = glm::quat_angle_axis(radians.x, &glm::vec3(1.0, 0.0, 0.0))
            * glm::quat_angle_axis(radians.y, &glm::vec3(0.0, 1.0, 0.0))
            * glm::quat_angle_axis(radians.z, &glm::vec3(0.0, 0.0, 1.0))
            * self.rotation;
        self.update_matrix();
        self
    }
}

// pub trait Casts: Any {
//     fn as_any(&self) -> &dyn Any;
//     fn as_any_mut(&mut self) -> &mut dyn Any;
// }

// impl<T: Any> Casts for T {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn as_any_mut(&mut self) -> &mut dyn Any {
//         self
//     }
// }

// TODO: Implement a more efficient way to cast to a specific trait

pub trait Transformable {
    fn apply_transform<F>(&mut self, operation: &mut F) -> &mut Self
    where
        F: FnMut(&mut NodeTransform);
}

// implement the Transformable trait for all types that implement the Node trait
impl<T: Node> Transformable for T {
    fn apply_transform<F>(&mut self, mut operation: &mut F) -> &mut Self
    where
        F: FnMut(&mut NodeTransform),
    {
        operation(self.get_transform());
        if let Some(model) = self.as_any_mut().downcast_mut::<Model>() {
            for node in &mut model.nodes {
                operation(&mut node.transform);
            }
        }

        for child in self.get_children().get_all_mut().values_mut() {
            let child_node: &mut dyn Node = &mut **child;
            apply_transform(child_node, operation);
        }
        self
    }
}

pub fn apply_transform<F>(node: &mut dyn Node, operation: &mut F)
where
    F: FnMut(&mut NodeTransform),
{
    operation(node.get_transform());

    if let Some(model) = node.as_any_mut().downcast_mut::<Model>() {
        for node in &mut model.nodes {
            operation(&mut node.transform);
        }
    }

    for child in node.get_children().get_all_mut().values_mut() {
        let child_node: &mut dyn Node = &mut **child;
        apply_transform(child_node, operation);
        println!("processing children");
    }
}

pub trait Node: Any {
    fn get_model_matrix(&mut self) -> &glm::Mat4 {
        &self.get_transform().matrix
    }

    fn get_transform(&mut self) -> &mut NodeTransform;

    fn get_children(&mut self) -> &mut NodeManager;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// cast to Ready trait if it implements it
    fn as_ready(&mut self) -> Option<&mut dyn Ready> {
        None
    }

    fn as_behavior(&mut self) -> Option<&mut dyn Behavior> {
        None
    }
}

pub trait Drawable {
    fn draw(&mut self, shader: &mut Shader, camera: &Camera3D);
    fn draw_shadow(&mut self, shader: &mut Shader, light_space_matrix: &Mat4);
}

pub struct NodeManager {
    nodes: HashMap<String, Box<dyn Node>>,
    pub shaders: HashMap<String, Box<Shader>>,
    pub shadow_shader: Option<Shader>,
    pub active_camera: String,
    pub active_shader: String,
}

impl Default for NodeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeManager {
    pub fn new() -> NodeManager {
        NodeManager {
            nodes: HashMap::new(),
            shaders: HashMap::new(),
            active_camera: String::new(),
            active_shader: String::new(),
            shadow_shader: None,
        }
    }

    // pub fn add<T: Node + 'static>(&mut self, name: &str, node: T) -> &mut T {
    //     self.nodes.insert(name.to_string(), Box::new(node));

    //     //if it's the first camera added then set it as the active camera if type is Camera3D
    //     if std::any::type_name::<T>() == std::any::type_name::<Camera3D>()
    //         && self.active_camera.is_empty()
    //     {
    //         self.active_camera = name.to_string();
    //     }

    //     self.nodes
    //         .get_mut(name)
    //         .unwrap()
    //         .as_any_mut()
    //         .downcast_mut::<T>()
    //         .unwrap()
    // }

    pub fn add<T: Node + 'static>(&mut self, name: &str, node: T) -> &mut T {
        // Insert the node into the map
        self.nodes.insert(name.to_string(), Box::new(node));

        // If it's the first camera added, set it as the active camera
        if std::any::type_name::<T>() == std::any::type_name::<Camera3D>()
            && self.active_camera.is_empty()
        {
            self.active_camera = name.to_string();
        }

        // Safely downcast and return the node
        self.nodes
            .get_mut(name)
            .and_then(|node| node.as_any_mut().downcast_mut::<T>())
            .expect("Failed to downcast the node")
    }

    pub fn ready(&mut self) {
        for node in self.nodes.values_mut() {
            if let Some(node) = node.as_ready() {
                node.ready();
            }
            // recursively call ready on all children
            node.get_children().ready();
        }
    }

    pub fn behavior(&mut self, context: &mut super::GameContext) {
        for node in self.nodes.values_mut() {
            if let Some(node) = node.as_behavior() {
                node.behavior(context);
            }
            // recursively call behavior on all children
            node.get_children().behavior(context);
        }
    }

    pub fn get_all(&self) -> &HashMap<String, Box<dyn Node>> {
        &self.nodes
    }

    pub fn get_all_mut(&mut self) -> &mut HashMap<String, Box<dyn Node>> {
        &mut self.nodes
    }

    pub fn get<T: 'static + Node>(&self, name: &str) -> Option<&T> {
        self.nodes
            .get(name)
            .and_then(|node| node.as_any().downcast_ref::<T>())
    }

    pub fn get_mut<T: 'static + Node>(&mut self, name: &str) -> Option<&mut T> {
        self.nodes
            .get_mut(name)
            .and_then(|node| node.as_any_mut().downcast_mut::<T>())
    }

    // get all nodes of a specific type as an iterator
    pub fn get_iter<T: 'static + Node>(&mut self) -> impl Iterator<Item = &mut T> {
        self.nodes
            .values_mut()
            .filter_map(|node| node.as_any_mut().downcast_mut::<T>())
    }

    pub fn get_vec<T: 'static + Node>(&mut self) -> Vec<&mut T> {
        self.nodes
            .values_mut()
            .filter_map(|node| node.as_any_mut().downcast_mut::<T>())
            .collect()
    }

    pub fn add_shader(&mut self, name: &str, shader: Shader) -> &mut Shader {
        self.shaders.insert(name.to_string(), Box::new(shader));
        if self.active_shader.is_empty() {
            self.active_shader = name.to_string();
        }
        self.shaders.get_mut(name).unwrap()
    }
}
