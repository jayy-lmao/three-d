
use crate::*;

pub struct ShadedVertices {
    program: program::Program,
    instance_buffer: buffer::StaticVertexBuffer,
    ball_index_buffer: buffer::ElementBuffer,
    ball_vertex_buffer: buffer::StaticVertexBuffer,
    no_vertices: u32,
    pub color: Vec3,
    pub diffuse_intensity: f32,
    pub specular_intensity: f32,
    pub specular_power: f32,
    pub scale: f32
}

impl ShadedVertices
{
    pub fn new(gl: &Gl, positions: &[f32]) -> ShadedVertices
    {
        let program = program::Program::from_source(&gl,
                                                    include_str!("shaders/vertex_shaded.vert"),
                                                    include_str!("shaders/shaded.frag")).unwrap();

        let x = 0.525731112119133606;
        let z = 0.850650808352039932;

        let ball_positions = vec!(
           -x, 0.0, z, x, 0.0, z, -x, 0.0, -z, x, 0.0, -z,
           0.0, z, x, 0.0, z, -x, 0.0, -z, x, 0.0, -z, -x,
           z, x, 0.0, -z, x, 0.0, z, -x, 0.0, -z, -x, 0.0
        );
        let ball_indices = vec!(
           0,1,4, 0,4,9, 9,4,5, 4,8,5, 4,1,8,
           8,1,10, 8,10,3, 5,8,3, 5,3,2, 2,3,7,
           7,3,10, 7,10,6, 7,6,11, 11,6,0, 0,6,1,
           6,10,1, 9,11,0, 9,2,11, 9,5,2, 7,11,2
        );
        let ball_index_buffer = buffer::ElementBuffer::new_with(gl, &ball_indices).unwrap();
        let ball_vertex_buffer = buffer::StaticVertexBuffer::new_with_vec3(gl, &ball_positions).unwrap();
        let instance_buffer = buffer::StaticVertexBuffer::new_with_vec3(gl, positions).unwrap();

        ShadedVertices { program, instance_buffer, ball_index_buffer, ball_vertex_buffer, no_vertices: positions.len() as u32/3, color: vec3(1.0, 0.0, 0.0),
            diffuse_intensity: 0.5, specular_intensity: 0.2, specular_power: 5.0, scale: 1.0 }
    }

    pub fn update_positions(&mut self, positions: &[f32])
    {
        //TODO: self.instance_buffer.fill_with(positions);
    }

    pub fn render(&mut self, camera: &camera::Camera)
    {
        self.program.cull(state::CullType::BACK);
        self.program.depth_test(state::DepthTestType::LEQUAL);
        self.program.depth_write(true);

        self.program.add_uniform_float("diffuse_intensity", &self.diffuse_intensity).unwrap();
        self.program.add_uniform_float("specular_intensity", &self.specular_intensity).unwrap();
        self.program.add_uniform_float("specular_power", &self.specular_power).unwrap();

        self.program.add_uniform_int("use_texture", &0).unwrap();
        self.program.add_uniform_vec3("color", &self.color).unwrap();

        self.program.add_uniform_float("scale", &self.scale).unwrap();

        self.program.add_uniform_mat4("viewMatrix", camera.get_view()).unwrap();
        self.program.add_uniform_mat4("projectionMatrix", camera.get_projection()).unwrap();

        self.program.use_attribute_vec3_float_divisor(&self.instance_buffer, "translation", 0, 1).unwrap();

        self.program.use_attribute_vec3_float(&self.ball_vertex_buffer, "position", 0).unwrap();

        self.program.draw_elements_instanced(&self.ball_index_buffer, self.no_vertices);
    }
}
