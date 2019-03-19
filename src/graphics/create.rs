use glium::Display;
use glium::index::PrimitiveType;

use cgmath::Vector3;

use super::Vertex;
use super::SceneObject;

pub struct SceneObjectCreator<'a> {
    display: &'a Display,
    buffer: Vec<Vertex>,
}

impl<'a> SceneObjectCreator<'a> {
    pub(crate) fn new(display: &'a Display) -> Self {
        SceneObjectCreator {
            display,
            buffer: Vec::new(),
        }
    }

    pub fn create(&mut self, vertices: &[Vector3<f32>], indices: &[u32]) -> SceneObject {
        self.buffer.clear();
        self.buffer.extend(vertices.iter().map(|v| Vertex { position: (*v).into() }));

        let vertex_buffer = glium::VertexBuffer::new(
            self.display,
            &self.buffer
        ).unwrap();

        let index_buffer = glium::IndexBuffer::new(
            self.display,
            PrimitiveType::TrianglesList,
            indices
        ).unwrap();

        SceneObject {
            vertex_buffer,
            index_buffer,
        }
    }
}
