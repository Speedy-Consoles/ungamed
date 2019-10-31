use std::f64::consts::PI;
use std::ops::Deref;

use glium::Frame;
use glium::Surface;
use glium::DrawParameters;
use glium::Program;
use glium::uniform;
use glium::texture::texture2d::Texture2d;

use glium_text::TextSystem;
use glium_text::FontTexture;
use glium_text::TextDisplay;

use cgmath::PerspectiveFov;
use cgmath::Ortho;
use cgmath::Matrix4;
use cgmath::Matrix3;
use cgmath::Vector3;
use cgmath::Rad;

use super::color::Color;
use super::TexturelessSceneObject3d;
use super::TexturedSceneObject3d;
use super::TexturelessSceneObject2d;
use super::TexturedSceneObject2d;
use super::LogicalSize;

pub const TEXT_NUM_LINES: u64 = 50; // Number of text lines that cover the whole vertical on the screen
const TEXT_MARGIN: f64 = 0.2; // Line height relative space between lines and to the screen borders,
const TEXT_LINE_HEIGHT: f64 = 2.0 / ((1.0 + TEXT_MARGIN) * TEXT_NUM_LINES as f64 + TEXT_MARGIN);

#[derive(Clone)]
pub enum Projection {
    Central {
        y_fov: f32,
        near: f32,
        far: f32,
    },
    Orthogonal {
        height: f32,
        near: f32,
        far: f32,
    }
}

impl Projection {
    pub fn as_matrix(&self, mut screen_ratio: f64, optimal_screen_ratio: f64) -> Matrix4<f32> {
        if screen_ratio <= 0.0 {
            screen_ratio = optimal_screen_ratio;
        }
        match self {
            &Projection::Central { y_fov, near, far } => {
                // perspective
                let cropped_y_fov = if screen_ratio > optimal_screen_ratio {
                    let tan = (y_fov as f64 / 2.0).tan();
                    let or = optimal_screen_ratio;
                    let sr = screen_ratio;
                    ((tan * or / sr).atan() * 2.0) as f32
                } else {
                    y_fov
                };
                let projection = PerspectiveFov {
                    fovy: Rad(cropped_y_fov),
                    aspect: screen_ratio as f32,
                    near,
                    far,
                };
                projection.into()
            },
            &Projection::Orthogonal { height, near, far } => {
                let d = (height / 2.0) as f64;
                let xd;
                let yd;
                if screen_ratio > optimal_screen_ratio {
                    xd = (d * optimal_screen_ratio) as f32;
                    yd = (d / screen_ratio) as f32;
                } else {
                    xd = (d * screen_ratio) as f32;
                    yd = d as f32;
                }
                Ortho {
                    left:   -xd,
                    right:   xd,
                    bottom: -yd,
                    top:     yd,
                    near,
                    far,
                }.into()
            }
        }
    }
}

#[derive(Clone)]
pub struct Camera {
    pub projection: Projection,
    pub translation_rotation: Matrix4<f32>,
}

impl Camera {
    pub fn as_matrix(&self, screen_ratio: f64, optimal_screen_ratio: f64) -> Matrix4<f32> {
        self.projection.as_matrix(screen_ratio, optimal_screen_ratio) * self.translation_rotation
    }
}

// TODO make colors own type (maybe use another crate?)
#[derive(Clone)]
pub struct SceneSettings {
    pub background_color: Color,
    pub camera: Camera,
    pub ambient_light_color: Color,
    pub directional_light_dir: Vector3<f32>,
    pub directional_light_color: Color,
}

impl Default for SceneSettings {
    fn default() -> Self {
        SceneSettings {
            background_color: Color::new(1.0, 0.5, 1.0),
            camera: Camera {
                projection: Projection::Central {
                    y_fov: (PI / 3.0) as f32,
                    near: 0.1,
                    far: 100.0,
                },
                translation_rotation: Matrix4::from_angle_x(Rad(-(PI * 0.1) as f32))
                    * Matrix4::from_angle_z(Rad(-(PI * 0.25) as f32))
                    * Matrix4::from_translation(-Vector3::new(1.0, -1.0, 5.0)),
            },
            ambient_light_color: Color::new(0.5, 0.5, 0.5),
            directional_light_dir: Vector3::new(-0.2, -0.4, -1.0),
            directional_light_color: Color::new(0.8, 0.6, 0.7),
        }
    }
}

pub struct SceneRenderer<'a> {
    frame: &'a mut Frame,
    world_program: &'a Program,
    overlay_program: &'a Program,
    draw_parameters: &'a DrawParameters<'a>,
    white_texture: &'a Texture2d,
    screen_ratio: f64,
    optimal_window_size: LogicalSize,
    text_system: &'a TextSystem,
    text_display: &'a mut TextDisplay<Box<FontTexture>>,
}

impl<'a> SceneRenderer<'a> {
    pub(crate) fn new(
        frame: &'a mut Frame,
        world_program: &'a Program,
        overlay_program: &'a Program,
        draw_parameters: &'a DrawParameters<'a>,
        white_texture: &'a Texture2d,
        screen_ratio: f64,
        optimal_window_size: LogicalSize,
        text_system: &'a TextSystem,
        text_display: &'a mut TextDisplay<Box<FontTexture>>,
    ) -> Self {
        SceneRenderer {
            frame,
            world_program,
            overlay_program,
            draw_parameters,
            white_texture,
            screen_ratio,
            optimal_window_size,
            text_system,
            text_display,
        }
    }

    pub fn start_object_rendering(self, settings: &SceneSettings) -> SceneObjectRenderer<'a> {
        // clear frame with color from scene settings
        self.frame.clear_color(
            settings.background_color.r,
            settings.background_color.g,
            settings.background_color.b,
            1.0
        );
        self.frame.clear_depth(1.0);

        let optimal_screen_ratio = self.optimal_window_size.width / self.optimal_window_size.height;

        // move content to object renderer and return it
        SceneObjectRenderer {
            frame: self.frame,
            world_program: self.world_program,
            overlay_program: self.overlay_program,
            draw_parameters: self.draw_parameters,
            white_texture: self.white_texture,
            screen_ratio: self.screen_ratio,
            optimal_window_size: self.optimal_window_size,
            world_to_screen_matrix: settings.camera.as_matrix(
                self.screen_ratio,
                optimal_screen_ratio,
            ),
            ambient_light_color: settings.ambient_light_color,
            directional_light_dir: settings.directional_light_dir,
            directional_light_color: settings.directional_light_color,
            text_system: self.text_system,
            text_display: self.text_display,
        }
    }
}

pub struct SceneObjectRenderer<'a> {
    frame: &'a mut Frame,
    world_program: &'a Program,
    overlay_program: &'a Program,
    draw_parameters: &'a DrawParameters<'a>,
    white_texture: &'a Texture2d,
    screen_ratio: f64,
    optimal_window_size: LogicalSize,
    world_to_screen_matrix: Matrix4<f32>,
    ambient_light_color: Color,
    directional_light_dir: Vector3<f32>,
    directional_light_color: Color,
    text_system: &'a TextSystem,
    text_display: &'a mut TextDisplay<Box<FontTexture>>,
}

impl<'a> SceneObjectRenderer<'a> {
    pub fn draw_textureless(
        &mut self,
        object: &TexturelessSceneObject3d,
        color: Color,
        object_to_world_matrix: &Matrix4<f32>
    ) {
        let object_to_world_matrix_uniform: [[f32; 4]; 4] = (*object_to_world_matrix).into();
        // TODO The following uniforms only change per frame, not per draw. Can we optimize this?
        let world_to_screen_matrix_uniform: [[f32; 4]; 4] = self.world_to_screen_matrix.into();
        let ambient_light_color_uniform: [f32; 3] = self.ambient_light_color.into();
        let directional_light_dir_uniform: [f32; 3] = self.directional_light_dir.into();
        let directional_light_color_uniform: [f32; 3] = self.directional_light_color.into();
        let color_uniform: [f32; 3] = color.into();
        let uniforms = uniform! {
            object_to_world_matrix:      object_to_world_matrix_uniform,
            world_to_screen_matrix:      world_to_screen_matrix_uniform,
            ambient_light_color:         ambient_light_color_uniform,
            directional_light_dir:       directional_light_dir_uniform,
            directional_light_color:     directional_light_color_uniform,
            color:                       color_uniform,
            tex:                         self.white_texture,
        };

        self.frame.draw(
            &object.vertex_buffer,
            &object.index_buffer,
            self.world_program,
            &uniforms,
            self.draw_parameters,
        ).unwrap();
    }

    pub fn draw_textured<T: Deref<Target = Texture2d>>(
        &mut self,
        object: &TexturedSceneObject3d<T>,
        object_to_world_matrix: &Matrix4<f32>
    ) {
        let object_to_world_matrix_uniform: [[f32; 4]; 4] = (*object_to_world_matrix).into();
        // TODO The following uniforms only change per frame, not per draw. Can we optimize this?
        let world_to_screen_matrix_uniform: [[f32; 4]; 4] = self.world_to_screen_matrix.into();
        let ambient_light_color_uniform: [f32; 3] = self.ambient_light_color.into();
        let directional_light_dir_uniform: [f32; 3] = self.directional_light_dir.into();
        let directional_light_color_uniform: [f32; 3] = self.directional_light_color.into();
        let color_uniform: [f32; 3] = [1.0, 1.0, 1.0];
        let uniforms = uniform! {
            object_to_world_matrix:      object_to_world_matrix_uniform,
            world_to_screen_matrix:      world_to_screen_matrix_uniform,
            ambient_light_color:         ambient_light_color_uniform,
            directional_light_dir:       directional_light_dir_uniform,
            directional_light_color:     directional_light_color_uniform,
            color:                       color_uniform,
            tex:                         object.texture.deref(),
        };

        self.frame.draw(
            &object.vertex_buffer,
            &object.index_buffer,
            self.world_program,
            &uniforms,
            self.draw_parameters,
        ).unwrap();
    }

    pub fn start_overlay_rendering(self) -> OverlayRenderer<'a> {
        self.frame.clear_depth(1.0);
        OverlayRenderer::new(
            self.screen_ratio,
            self.optimal_window_size,
            self.frame,
            self.text_system,
            self.text_display,
            self.overlay_program,
            self.draw_parameters,
            self.white_texture,
        )
    }
}

pub struct OverlayRenderer<'a> {
    frame: &'a mut Frame,
    program: &'a Program,
    draw_parameters: &'a DrawParameters<'a>,
    white_texture: &'a Texture2d,
    overlay_to_screen_matrix: Matrix3<f32>,
    text_system: &'a TextSystem,
    text_display: &'a mut TextDisplay<Box<FontTexture>>,
}

impl<'a> OverlayRenderer<'a> {
    fn new(
        screen_ratio: f64,
        optimal_window_size: LogicalSize,
        frame: &'a mut Frame,
        text_system: &'a TextSystem,
        text_display: &'a mut TextDisplay<Box<FontTexture>>,
        program: &'a Program,
        draw_parameters: &'a DrawParameters<'a>,
        white_texture: &'a Texture2d,
    ) -> OverlayRenderer<'a> {
        let optimal_screen_ratio = optimal_window_size.width / optimal_window_size.height;
        let ratio_ratio = screen_ratio / optimal_screen_ratio;
        let x_scaling;
        let y_scaling;
        let x_offset;
        let y_offset;
        if ratio_ratio > 1.0 {
            x_scaling = (2.0 / ratio_ratio / optimal_window_size.width) as f32;
            y_scaling = (2.0 / optimal_window_size.height) as f32;
            x_offset = (-1.0 / ratio_ratio) as f32;
            y_offset = -1.0f32;
        } else {
            x_scaling = (2.0 / optimal_window_size.width) as f32;
            y_scaling = (2.0 * ratio_ratio / optimal_window_size.height) as f32;
            x_offset = -1.0f32;
            y_offset = -ratio_ratio as f32;
        }
        let overlay_to_screen_matrix = Matrix3::new(
            x_scaling, 0.0,       0.0,
            0.0,       y_scaling, 0.0,
            x_offset,  y_offset,  1.0f32,
        );

        OverlayRenderer {
            frame,
            overlay_to_screen_matrix,
            text_system,
            text_display,
            program,
            draw_parameters,
            white_texture,
        }
    }

    pub fn draw_textureless(
        &mut self,
        object: &TexturelessSceneObject2d,
        color: Color,
        object_to_overlay_matrix: &Matrix3<f32>
    ) {
        let object_to_screen_matrix = self.overlay_to_screen_matrix * object_to_overlay_matrix;
        let object_to_screen_matrix_uniform: [[f32; 3]; 3] = object_to_screen_matrix.into();
        // TODO The following uniforms only change per frame, not per draw. Can we optimize this?
        let color_uniform: [f32; 3] = color.into();
        let uniforms = uniform! {
            object_to_screen_matrix:     object_to_screen_matrix_uniform,
            color:                       color_uniform,
            tex:                         self.white_texture,
        };

        self.frame.draw(
            &object.vertex_buffer,
            &object.index_buffer,
            self.program,
            &uniforms,
            self.draw_parameters,
        ).unwrap();
    }

    pub fn draw_textured<T: Deref<Target = Texture2d>>(
        &mut self,
        object: &TexturedSceneObject2d<T>,
        object_to_overlay_matrix: &Matrix3<f32>
    ) {
        let object_to_screen_matrix = self.overlay_to_screen_matrix * object_to_overlay_matrix;
        let object_to_screen_matrix_uniform: [[f32; 3]; 3] = object_to_screen_matrix.into();
        // TODO The following uniforms only change per frame, not per draw. Can we optimize this?
        let color_uniform: [f32; 3] = [1.0, 1.0, 1.0];
        let uniforms = uniform! {
            object_to_screen_matrix:     object_to_screen_matrix_uniform,
            color:                       color_uniform,
            tex:                         object.texture.deref(),
        };

        self.frame.draw(
            &object.vertex_buffer,
            &object.index_buffer,
            self.program,
            &uniforms,
            self.draw_parameters,
        ).unwrap();
    }

    pub fn draw_text(&mut self, line_number: u64, text: &str) {
        assert!(
            line_number < TEXT_NUM_LINES,
            "line_number can't be greater than {}",
            TEXT_NUM_LINES
        );
        self.text_display.set_text(text);

        let x_offset = TEXT_MARGIN;
        let y_offset = TEXT_MARGIN + (TEXT_NUM_LINES - 1 - line_number) as f64 * (1.0 + TEXT_MARGIN);
        let translation = Vector3::new(x_offset as f32, y_offset as f32, 0.0);
        let translation_matrix = Matrix4::from_translation(translation);

        // TODO
        /*let ratio_ratio = screen_ratio / optimal_screen_ratio;
        let x_scaling;
        let y_scaling;
        let x_offset;
        let y_offset;
        if ratio_ratio > 1.0 {
            x_scaling = (TEXT_LINE_HEIGHT / screen_ratio) as f32;
            y_scaling = TEXT_LINE_HEIGHT as f32;
            x_offset = (-1.0 / ratio_ratio) as f32;
            y_offset = -1.0;
        } else {
            x_scaling = (TEXT_LINE_HEIGHT / optimal_screen_ratio) as f32;
            y_scaling = (TEXT_LINE_HEIGHT * ratio_ratio) as f32;
            x_offset = -1.0;
            y_offset = (-1.0 * ratio_ratio) as f32;
        }
        let text_area_to_screen_matrix = Matrix4::new(
            x_scaling, 0.0, 0.0, 0.0,
            0.0, y_scaling, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            x_offset, y_offset, 0.0, 1.0f32,
        );*/

        let text_area_to_screen_matrix = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0f32,
        ); // TODO

        glium_text::draw(
            self.text_display,
            self.text_system,
            self.frame,
            text_area_to_screen_matrix * translation_matrix,
            (1.0, 1.0, 1.0, 1.0),
        );
    }
}

