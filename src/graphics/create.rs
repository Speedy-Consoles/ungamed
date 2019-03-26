use glium::Display;
use glium::index::PrimitiveType;
use glium::texture::RawImage2d;
use glium::texture::texture2d::Texture2d;

use cgmath::Vector3;
use cgmath::Vector2;

use image::DynamicImage;
use image::GenericImageView;

use super::Vertex;
use super::TexturelessSceneObject;
use super::TexturedSceneObject;

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

    pub fn create_texture(&mut self, image: DynamicImage) -> Texture2d {
        let image_dimensions = image.dimensions();
        let raw_image = RawImage2d::from_raw_rgba_reversed(
            &image.to_rgba().into_raw(),
            image_dimensions
        );
        Texture2d::new(self.display, raw_image).unwrap()
    }

    pub fn create_textureless(
        &mut self,
        vertices: &[Vector3<f32>],
        indices: &[u32],
    ) -> TexturelessSceneObject {
        self.buffer.clear();
        self.buffer.extend(vertices.iter().map(|v| Vertex {
            position: (*v).into(),
            texture_position: [0.0, 0.0],
        }));

        let vertex_buffer = glium::VertexBuffer::new(
            self.display,
            &self.buffer
        ).unwrap();

        let index_buffer = glium::IndexBuffer::new(
            self.display,
            PrimitiveType::TrianglesList,
            indices
        ).unwrap();

        TexturelessSceneObject {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn create_textured(
        &mut self,
        vertices: &[(Vector3<f32>, Vector2<f32>)],
        indices: &[u32],
    ) -> TexturedSceneObject {
        self.buffer.clear();
        self.buffer.extend(vertices.iter().map(|v| Vertex {
            position: v.0.into(),
            texture_position: v.1.into(),
        }));

        let vertex_buffer = glium::VertexBuffer::new(
            self.display,
            &self.buffer
        ).unwrap();

        let index_buffer = glium::IndexBuffer::new(
            self.display,
            PrimitiveType::TrianglesList,
            indices
        ).unwrap();

        TexturedSceneObject {
            vertex_buffer,
            index_buffer,
        }
    }
}
