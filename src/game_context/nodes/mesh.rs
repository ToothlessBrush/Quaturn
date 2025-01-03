//! Mesh module for the mesh struct and its implementation
//!
//! The `mesh` module provides a struct for managing the mesh of a model, including vertices, indices, textures, and material properties.

use nalgebra_glm as glm; // Importing the nalgebra_glm crate for mathematical operations

use super::camera::Camera3D;
use super::model::Vertex;
use crate::renderer::buffers::{
    index_buffer::IndexBuffer, vertex_array::VertexArray, vertex_buffer::VertexBuffer,
    vertex_buffer_layout::VertexBufferLayout,
};
use crate::renderer::{shader::Shader, texture::Texture, Renderer};

use std::rc::Rc; //reference counted pointer

/// Material properties for the mesh
#[derive(Debug)]
pub struct MaterialProperties {
    /// Base color factor of the material
    pub base_color_factor: glm::Vec4,
    /// Metallic factor of the material
    pub metallic_factor: f32,
    /// Roughness factor of the material
    pub roughness_factor: f32,
    /// Double sided property of the material
    pub double_sided: bool,
    /// Alpha mode of the material
    pub alpha_mode: String,
    /// Alpha cutoff of the material
    pub alpha_cutoff: f32,
}

/// Mesh struct for managing the mesh of a model
pub struct Mesh {
    _vertices: Vec<Vertex>,
    /// Indices of the mesh
    pub indices: Vec<u32>,
    /// Textures of the mesh
    textures: Vec<Rc<Texture>>,
    /// Material properties of the mesh
    pub material_properties: MaterialProperties,
    /// Vertex array of the mesh
    vertex_array: VertexArray,
    /// Index buffer of the mesh
    index_buffer: IndexBuffer,
}

impl Mesh {
    /// Creates a new mesh
    ///
    /// Mesh Is not a Node it is a struct that holds the data for a mesh use Model to create a node with a mesh
    ///
    /// # Arguments
    /// - `vertices` - The vertices of the mesh
    /// - `indices` - The indices of the mesh
    /// - `textures` - The textures of the mesh
    /// - `material_properties` - The material properties of the mesh
    ///
    /// # Returns
    /// The new mesh
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        textures: Vec<Rc<Texture>>,
        material_properties: MaterialProperties,
    ) -> Mesh {
        // println!("{:?}", material_properties);

        let va = VertexArray::new();

        va.bind();

        let vb = VertexBuffer::new(&vertices);

        let mut layout = VertexBufferLayout::new();
        layout.push::<f32>(3); //positions (x, y, z) (location 0 in the shader)
        layout.push::<f32>(3); //normals (location 1 in the shader)
        layout.push::<f32>(4); //color (r, g, b, a) (location 2 in the shader)
        layout.push::<f32>(2); //texture coordinates (u, v) (location 3 in the shader)
        va.add_buffer(&vb, &layout);

        let ib = IndexBuffer::new(&indices);

        va.unbind();
        vb.unbind();
        ib.unbind();

        Mesh {
            _vertices: vertices,
            indices,
            textures,
            material_properties,
            vertex_array: va,
            index_buffer: ib,
        }
    }

    /// Draw the mesh with the shader uniform and shader binding handled in Model
    ///
    /// # Arguments
    /// - `shader` - The shader to draw the mesh with
    /// - `camera` - The camera to draw the mesh with
    pub fn draw(&self, shader: &mut Shader, camera: &Camera3D) {
        //bind stuff
        shader.bind();
        self.vertex_array.bind();
        self.index_buffer.bind();

        let mut num_diffuse: u32 = 0;
        let mut num_specular: u32 = 0;

        //set the texture unifroms based on the type of texture
        for i in 0..self.textures.len() {
            let tex_type = &self.textures[i].tex_type;
            let mut num: String = "0".to_string();
            if tex_type == "diffuse" {
                shader.set_uniform_bool("useTexture", true);
                num = num_diffuse.to_string();
                num_diffuse += 1;
            }
            if tex_type == "specular" {
                num = num_specular.to_string();
                num_specular += 1;
                //break;
            }
            let uniform_name = format!("{}{}", tex_type, num);

            //set the unifrom for the texture in the shader
            //println!("setting uniform: {} to slot {}", uniform_name, i);

            self.textures[i].tex_unit(shader, &uniform_name, i as u32); //set the sampler2d uniform to the texture unit
            self.textures[i].bind(i as u32); //bind the texture to the texture unit
        }

        let camera_pos = camera.get_position();
        shader.set_uniform3f("camPos", camera_pos.x, camera_pos.y, camera_pos.z);

        shader.set_uniform_mat4f("u_VP", &camera.get_vp_matrix());

        shader.set_uniform4f(
            "baseColorFactor",
            self.material_properties.base_color_factor.x,
            self.material_properties.base_color_factor.y,
            self.material_properties.base_color_factor.z,
            self.material_properties.base_color_factor.w,
        );

        if self.material_properties.alpha_mode == "MASK" {
            shader.set_uniform_bool("useAlphaCutoff", true);
            shader.set_uniform1f("alphaCutoff", self.material_properties.alpha_cutoff);
        }

        shader.set_uniform1f("u_SpecularStrength", 0.5);

        Renderer::draw(self);

        // reset stuff
        self.textures.iter().for_each(|t| t.unbind()); //unbind the textures
        shader.set_uniform_bool("useTexture", false); //set the useTexture uniform to false (default)
        shader.set_uniform_bool("useAlphaCutoff", false); //set the useAlphaCutoff uniform to false (default)
    }

    /// Draw the mesh with the shadow shader uniform and shader binding handled in Model
    pub fn draw_shadow(&self) {
        self.vertex_array.bind();
        self.index_buffer.bind();

        Renderer::draw(self);
    }
}
