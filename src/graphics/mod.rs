pub mod render;
pub mod create;
pub mod color;

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

use self::create::SceneObjectCreator;
use self::render::SceneRenderer;

const VERTEX_SHADER_SOURCE: &'static str = include_str!("../../shader_src/vertex_shader.vert");
const FRAGMENT_SHADER_SOURCE: &'static str = include_str!("../../shader_src/fragment_shader.frag");
const GEOMETRY_SHADER_SOURCE: &'static str = include_str!("../../shader_src/geometry_shader.geo");

const TEXT_FONT_SIZE: u32 = 20;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    texture_position: [f32; 2],
}

implement_vertex!(Vertex, position, texture_position);

pub struct TexturelessSceneObject {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
}

pub struct TexturedSceneObject {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
}

pub struct Graphics {
    display: Display,
    program: Program,
    draw_parameters: DrawParameters<'static>,
    screen_ratio: f64,
    optimal_screen_ratio: f64,
    text_system: TextSystem,
    text_display: TextDisplay<Box<FontTexture>>,
    white_texture: Texture2d,
}

impl Graphics {
    pub fn new(display: Display, optimal_window_size: LogicalSize) -> Self {
        // load shader sources and create program
        let program = glium::Program::from_source(
            &display,
            VERTEX_SHADER_SOURCE,
            FRAGMENT_SHADER_SOURCE,
            Some(GEOMETRY_SHADER_SOURCE),
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
        let mut optimal_screen_ratio = 1.0;
        if optimal_window_size.width > 0.0 && optimal_window_size.height > 0.0 {
            optimal_screen_ratio = optimal_window_size.width / optimal_window_size.height;
        }

        // load font and create text system
        let font_file = include_bytes!("../../font/DejaVuSansMono.ttf");
        let font = FontTexture::new(&display, font_file.as_ref(), TEXT_FONT_SIZE).unwrap();
        let text_system = TextSystem::new(&display);
        let text_display = TextDisplay::new(&text_system, Box::new(font), "");

        // create an empty texture
        let white_texture = Texture2d::new(&display, vec![vec![(1.0, 1.0, 1.0, 1.0)]]).unwrap();

        // create the graphics
        Graphics {
            display,
            program,
            draw_parameters,
            screen_ratio: optimal_screen_ratio, // TODO this is ugly
            optimal_screen_ratio,
            text_system,
            text_display,
            white_texture,
        }
    }

    pub fn render<G: super::Game>(&mut self, game: &G) {
        // create new frame
        let mut frame = self.display.draw();

        // create the renderer
        let scene_renderer = SceneRenderer::new(
            &mut frame,
            &self.program,
            &self.draw_parameters,
            &self.white_texture,
            self.screen_ratio,
            self.optimal_screen_ratio,
            &self.text_system,
            &mut self.text_display,
        );

        // let the game render the scene via the renderer
        game.render(scene_renderer);

        // swap buffers
        frame.finish().unwrap(); // TODO maybe not unwrap?
    }

    pub fn set_view_port_size(&mut self, size: LogicalSize) {
        // make sure the ratio is valid and save it
        self.screen_ratio = if size.width <= 0.0 || size.height <= 0.0 {
            self.optimal_screen_ratio
        } else {
            size.width / size.height
        }
    }

    pub fn object_creator(&mut self) -> SceneObjectCreator {
        // give out scene object creator for the constructor of the game
        SceneObjectCreator::new(&self.display)
    }
}