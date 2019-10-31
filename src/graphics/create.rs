use std::ops::Deref;

use glium::Display;
use glium::index::PrimitiveType;
use glium::texture::RawImage2d;
use glium::texture::texture2d::Texture2d;

use cgmath::Vector3;
use cgmath::Vector2;

use image::DynamicImage;
use image::GenericImageView;

use super::Vertex3d;
use super::Vertex2d;
use super::TexturelessSceneObject3d;
use super::TexturedSceneObject3d;
use super::TexturelessSceneObject2d;
use super::TexturedSceneObject2d;

pub struct SceneObjectCreator<'a> {
    display: &'a Display,
    buffer3d: Vec<Vertex3d>,
    buffer2d: Vec<Vertex2d>,
}

impl<'a> SceneObjectCreator<'a> {
    pub(crate) fn new(display: &'a Display) -> Self {
        SceneObjectCreator {
            display,
            buffer3d: Vec::new(),
            buffer2d: Vec::new(),
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

    pub fn create_textureless3d(
        &mut self,
        vertices: &[Vector3<f32>],
        indices: &[u32],
    ) -> TexturelessSceneObject3d {
        self.buffer3d.clear();
        self.buffer3d.extend(vertices.iter().map(|v| Vertex3d {
            position: (*v).into(),
            texture_position: [0.0, 0.0],
        }));
        let vertex_buffer = glium::VertexBuffer::new(self.display, &self.buffer3d).unwrap();
        let index_buffer = glium::IndexBuffer::new(
            self.display,
            PrimitiveType::TrianglesList,
            indices
        ).unwrap();

        TexturelessSceneObject3d {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn create_textured3d<T: Deref<Target = Texture2d>>(
        &mut self,
        vertices: &[(Vector3<f32>, Vector2<f32>)],
        indices: &[u32],
        texture: T,
    ) -> TexturedSceneObject3d<T> {
        self.buffer3d.clear();
        self.buffer3d.extend(vertices.iter().map(|v| Vertex3d {
            position: v.0.into(),
            texture_position: v.1.into(),
        }));
        let vertex_buffer = glium::VertexBuffer::new(self.display, &self.buffer3d).unwrap();
        let index_buffer = glium::IndexBuffer::new(
            self.display,
            PrimitiveType::TrianglesList,
            indices
        ).unwrap();

        TexturedSceneObject3d {
            vertex_buffer,
            index_buffer,
            texture,
        }
    }

    pub fn create_textureless2d(
        &mut self,
        vertices: &[Vector2<f32>],
        indices: &[u32],
    ) -> TexturelessSceneObject2d {
        self.buffer2d.clear();
        self.buffer2d.extend(vertices.iter().map(|v| Vertex2d {
            position: (*v).into(),
            texture_position: [0.0, 0.0],
        }));
        let vertex_buffer = glium::VertexBuffer::new(self.display, &self.buffer2d).unwrap();
        let index_buffer = glium::IndexBuffer::new(
            self.display,
            PrimitiveType::TrianglesList,
            indices
        ).unwrap();

        TexturelessSceneObject2d {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn create_textured2d<T: Deref<Target = Texture2d>>(
        &mut self,
        vertices: &[(Vector2<f32>, Vector2<f32>)],
        indices: &[u32],
        texture: T,
    ) -> TexturedSceneObject2d<T> {
        self.buffer2d.clear();
        self.buffer2d.extend(vertices.iter().map(|v| Vertex2d {
            position: [v.0.x, v.0.y],
            texture_position: v.1.into(),
        }));
        let vertex_buffer = glium::VertexBuffer::new(self.display, &self.buffer2d).unwrap();
        let index_buffer = glium::IndexBuffer::new(
            self.display,
            PrimitiveType::TrianglesList,
            indices
        ).unwrap();

        TexturedSceneObject2d {
            vertex_buffer,
            index_buffer,
            texture,
        }
    }
}
