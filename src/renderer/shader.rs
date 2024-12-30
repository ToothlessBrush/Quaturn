use colored::*;
use nalgebra_glm as glm; // Importing the nalgebra_glm crate for mathematical operations

pub struct Shader {
    m_renderer_id: u32,
    m_uniform_location_cache: std::collections::HashMap<String, i32>,
}

impl Default for Shader {
    fn default() -> Self {
        Shader::new(
            "res/shaders/default/default.vert",
            "res/shaders/default/default.frag",
            None,
        )
    }
}

impl Shader {
    /// Creates a new shader object, optionally with a geometry shader
    pub fn new(vertex_path: &str, fragment_path: &str, geometry_path: Option<&str>) -> Shader {
        println!("Compiling shader {:?}... ", vertex_path);
        let vertex_shader =
            std::fs::read_to_string(vertex_path).expect("Failed to read vertex shader file");
        println!("Compiling shader {:?}... ", fragment_path);
        let fragment_shader =
            std::fs::read_to_string(fragment_path).expect("Failed to read fragment shader file");

        let geometry_shader = if let Some(path) = geometry_path {
            println!("Compiling shader {:?}... ", path);
            Some(std::fs::read_to_string(path).expect("Failed to read geometry shader file"))
        } else {
            None
        };

        Shader {
            m_renderer_id: Self::create_shader(
                &vertex_shader,
                &fragment_shader,
                geometry_shader.as_deref(),
            ),
            m_uniform_location_cache: std::collections::HashMap::new(),
        }
    }

    /// Compiles and links shaders, including an optional geometry shader
    fn create_shader(
        vertex_shader: &str,
        fragment_shader: &str,
        geometry_shader: Option<&str>,
    ) -> u32 {
        let program = unsafe { gl::CreateProgram() };
        let vs = Self::compile_shader(gl::VERTEX_SHADER, vertex_shader);
        let fs = Self::compile_shader(gl::FRAGMENT_SHADER, fragment_shader);

        unsafe {
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);

            if let Some(gs_src) = geometry_shader {
                let gs = Self::compile_shader(gl::GEOMETRY_SHADER, gs_src);
                gl::AttachShader(program, gs);
                gl::DeleteShader(gs); // Clean up after attaching
            }

            gl::LinkProgram(program);
            gl::ValidateProgram(program);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);
        }

        program
    }

    /// Compiles individual shader stages
    fn compile_shader(type_: u32, source: &str) -> u32 {
        let id = unsafe { gl::CreateShader(type_) };
        let c_str = std::ffi::CString::new(source).unwrap();

        unsafe {
            gl::ShaderSource(id, 1, &c_str.as_ptr(), std::ptr::null());
            gl::CompileShader(id);

            let mut result = gl::FALSE as i32;
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut result);
            if result == gl::FALSE as i32 {
                let mut length = 0;
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut length);
                let mut message = vec![0; length as usize];
                gl::GetShaderInfoLog(
                    id,
                    length,
                    std::ptr::null_mut(),
                    message.as_mut_ptr() as *mut i8,
                );
                println!(
                    "Failed to compile {:?} shader!",
                    match type_ {
                        gl::VERTEX_SHADER => "Vertex",
                        gl::FRAGMENT_SHADER => "Fragment",
                        gl::GEOMETRY_SHADER => "Geometry",
                        _ => "Unknown",
                    }
                );
                println!(
                    "{}",
                    std::str::from_utf8(&message).expect("Shader info log is not valid UTF-8")
                );
                gl::DeleteShader(id);
                return 0;
            }
        }

        id
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.m_renderer_id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn set_uniform1i(&mut self, name: &str, value: i32) {
        unsafe {
            gl::Uniform1i(self.get_uniform_location(name), value);
        }
    }

    pub fn set_uniform1f(&mut self, name: &str, value: f32) {
        unsafe {
            gl::Uniform1f(self.get_uniform_location(name), value);
        }
    }

    pub fn set_uniform3f(&mut self, name: &str, v0: f32, v1: f32, v2: f32) {
        unsafe {
            gl::Uniform3f(self.get_uniform_location(name), v0, v1, v2);
        }
    }

    pub fn set_uniform4f(&mut self, name: &str, v0: f32, v1: f32, v2: f32, v3: f32) {
        unsafe {
            gl::Uniform4f(self.get_uniform_location(name), v0, v1, v2, v3);
        }
    }

    pub fn set_uniform_mat4f(&mut self, name: &str, matrix: &glm::Mat4) {
        unsafe {
            gl::UniformMatrix4fv(
                self.get_uniform_location(name),
                1,
                gl::FALSE,
                matrix.as_ptr(),
            );
        }
    }

    pub fn set_uniform_bool(&mut self, name: &str, value: bool) {
        unsafe {
            gl::Uniform1i(self.get_uniform_location(name), value as i32);
        }
    }

    pub fn get_uniform_location(&mut self, name: &str) -> i32 {
        //get from cache since gpu -> cpu is forbidden by the computer gods
        if self.m_uniform_location_cache.contains_key(name) {
            return self.m_uniform_location_cache[name];
        }

        //get the location of the uniform if not in the cache
        let c_str = std::ffi::CString::new(name).unwrap();
        let location = unsafe {
            let location = gl::GetUniformLocation(self.m_renderer_id, c_str.as_ptr());
            if location == -1 {
                println!(
                    "{}",
                    format!("Warning: uniform '{:?}' doesn't exist!", name).yellow()
                );
            }
            location
        };

        self.m_uniform_location_cache
            .insert(name.to_string(), location);
        location
    }
}