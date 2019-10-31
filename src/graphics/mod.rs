pub mod render;
pub mod create;
pub mod color;

use std::ops::Deref;

use glium::Display;
use glium::DrawParameters;
use glium::Blend;
use glium::Program;
use glium::Depth;
use glium::glutin::dpi::LogicalSize;
use glium::implement_vertex;
use glium::VertexBuffer;
use glium::IndexBuffer;
use glium::texture::texture2d::Texture2d;

use glium_text::TextSystem;
use glium_text::TextDisplay;
use glium_text::FontTexture;

use crate::GameInfo;
use crate::GraphicsInfo;

use self::create::SceneObjectCreator;
use self::render::SceneRenderer;

const WORLD_VERTEX_SHADER_SOURCE: &'static str = include_str!("../../shader_src/world/vertex_shader.vert");
const WORLD_FRAGMENT_SHADER_SOURCE: &'static str = include_str!("../../shader_src/world/fragment_shader.frag");
const WORLD_GEOMETRY_SHADER_SOURCE: &'static str = include_str!("../../shader_src/world/geometry_shader.geo");

const OVERLAY_VERTEX_SHADER_SOURCE: &'static str = include_str!("../../shader_src/overlay/vertex_shader.vert");
const OVERLAY_FRAGMENT_SHADER_SOURCE: &'static str = include_str!("../../shader_src/overlay/fragment_shader.frag");
const OVERLAY_GEOMETRY_SHADER_SOURCE: &'static str = include_str!("../../shader_src/overlay/geometry_shader.geo");

const TEXT_FONT_SIZE: u32 = 20;

#[derive(Copy, Clone)]
pub struct Vertex3d {
    position: [f32; 3],
    texture_position: [f32; 2],
}
implement_vertex!(Vertex3d, position, texture_position);

#[derive(Copy, Clone)]
pub struct Vertex2d {
    position: [f32; 2],
    texture_position: [f32; 2],
}
implement_vertex!(Vertex2d, position, texture_position);

pub struct TexturelessSceneObject3d {
    vertex_buffer: VertexBuffer<Vertex3d>,
    index_buffer: IndexBuffer<u32>,
}

pub struct TexturedSceneObject3d<T: Deref<Target = Texture2d>> {
    vertex_buffer: VertexBuffer<Vertex3d>,
    index_buffer: IndexBuffer<u32>,
    texture: T,
}

pub struct TexturelessSceneObject2d {
    vertex_buffer: VertexBuffer<Vertex2d>,
    index_buffer: IndexBuffer<u32>,
}

pub struct TexturedSceneObject2d<T: Deref<Target = Texture2d>> {
    vertex_buffer: VertexBuffer<Vertex2d>,
    index_buffer: IndexBuffer<u32>,
    texture: T,
}

pub struct Graphics {
    world_program: Program,
    overlay_program: Program,
    draw_parameters: DrawParameters<'static>,
    screen_ratio: f64,
    optimal_window_size: LogicalSize,
    text_system: TextSystem,
    text_display: TextDisplay<Box<FontTexture>>,
    white_texture: Texture2d,
}

impl Graphics {
    pub fn new(display: &Display, mut optimal_window_size: LogicalSize) -> Self {
        // load shader sources and create programs
        let world_program = glium::Program::from_source(
            display,
            WORLD_VERTEX_SHADER_SOURCE,
            WORLD_FRAGMENT_SHADER_SOURCE,
            Some(WORLD_GEOMETRY_SHADER_SOURCE),
        ).unwrap();

        let overlay_program = glium::Program::from_source(
            display,
            OVERLAY_VERTEX_SHADER_SOURCE,
            OVERLAY_FRAGMENT_SHADER_SOURCE,
            Some(OVERLAY_GEOMETRY_SHADER_SOURCE),
        ).unwrap();

        // create draw parameters
        let draw_parameters = DrawParameters {
            depth: Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            blend: Blend::alpha_blending(),
            ..Default::default()
        };

        // make sure that optimal_screen_ratio is valid
        optimal_window_size.width = optimal_window_size.width.max(1.0);
        optimal_window_size.height = optimal_window_size.height.max(1.0);
        let optimal_screen_ratio = optimal_window_size.width / optimal_window_size.height;

        // load font and create text system
        let font_file = include_bytes!("../../font/DejaVuSansMono.ttf");
        let font = FontTexture::new(display, font_file.as_ref(), TEXT_FONT_SIZE).unwrap();
        let text_system = TextSystem::new(display);
        let text_display = TextDisplay::new(&text_system, Box::new(font), "");

        // create an empty texture
        let white_texture = Texture2d::new(display, vec![vec![(1.0, 1.0, 1.0, 1.0)]]).unwrap();

        // create the graphics
        Graphics {
            world_program,
            overlay_program,
            draw_parameters,
            screen_ratio: optimal_screen_ratio, // TODO this is ugly
            optimal_window_size,
            text_system,
            text_display,
            white_texture,
        }
    }

    pub fn render<A: super::Application>(
        &mut self,
        application: &A,
        display: &Display,
        game_info: Option<GameInfo<A::G>>,
        graphics_info: GraphicsInfo,
    ) {
        // create new frame
        let mut frame = display.draw();

        // create the renderer
        let scene_renderer = SceneRenderer::new(
            &mut frame,
            &self.world_program,
            &self.overlay_program,
            &self.draw_parameters,
            &self.white_texture,
            self.screen_ratio,
            self.optimal_window_size,
            &self.text_system,
            &mut self.text_display,
        );

        // let the game render the scene via the renderer
        application.render(game_info, graphics_info, scene_renderer);

        // swap buffers
        frame.finish().unwrap(); // TODO maybe not unwrap?
    }

    pub fn set_view_port_size(&mut self, size: LogicalSize) {
        // make sure the ratio is valid and save it
        let w = size.width.max(1.0);
        let h = size.height.max(1.0);
        self.screen_ratio = w / h;
    }

    pub fn object_creator<'a>(&mut self, display: &'a Display) -> SceneObjectCreator<'a> {
        // give out scene object creator for the constructor of the game
        SceneObjectCreator::new(display)
    }
}