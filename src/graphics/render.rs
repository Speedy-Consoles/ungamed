use std::f64::consts::PI;

use glium::Frame;
use glium::Surface;
use glium::DrawParameters;
use glium::Program;
use glium::uniform;

use cgmath::PerspectiveFov;
use cgmath::Ortho;
use cgmath::Matrix4;
use cgmath::Vector3;
use cgmath::Rad;

use super::SceneObject;

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
    pub background_color: Vector3<f32>,
    pub camera: Camera,
    pub ambient_light_color: Vector3<f32>,
    pub directional_light_dir: Vector3<f32>,
    pub directional_light_color: Vector3<f32>,
}

impl Default for SceneSettings {
    fn default() -> Self {
        SceneSettings {
            background_color: Vector3::new(1.0, 0.5, 1.0),
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
            ambient_light_color: Vector3::new(0.5, 0.5, 0.5),
            directional_light_dir: Vector3::new(-0.2, -0.4, -1.0),
            directional_light_color: Vector3::new(0.8, 0.6, 0.7),
        }
    }
}

pub struct SceneObjectRenderer<'a> {
    frame: &'a mut Frame,
    program: &'a Program,
    draw_parameters: &'a DrawParameters<'a>,
    world_to_screen_matrix: Matrix4<f32>,
    ambient_light_color: Vector3<f32>,
    directional_light_dir: Vector3<f32>,
    directional_light_color: Vector3<f32>,
}

impl<'a> SceneObjectRenderer<'a> {
    pub fn draw(&mut self, object: &SceneObject, object_to_world_matrix: &Matrix4<f32>) {
        let object_to_world_matrix_uniform: [[f32; 4]; 4] = (*object_to_world_matrix).into();
        // TODO The following uniforms only change per frame, not per draw. Can we optimize this?
        let world_to_screen_matrix_uniform: [[f32; 4]; 4] = self.world_to_screen_matrix.into();
        let ambient_light_color_uniform: [f32; 3] = self.ambient_light_color.into();
        let directional_light_dir_uniform: [f32; 3] = self.directional_light_dir.into();
        let directional_light_color_uniform: [f32; 3] = self.directional_light_color.into();
        let uniforms = uniform! {
            object_to_world_matrix:      object_to_world_matrix_uniform,
            world_to_screen_matrix:      world_to_screen_matrix_uniform,
            ambient_light_color:         ambient_light_color_uniform,
            directional_light_dir:       directional_light_dir_uniform,
            directional_light_color:     directional_light_color_uniform,
        };

        self.frame.draw(
            &object.vertex_buffer,
            &object.index_buffer,
            self.program,
            &uniforms,
            self.draw_parameters,
        ).unwrap();
    }
}

pub struct SceneRenderer<'a> {
    frame: &'a mut Frame,
    program: &'a Program,
    draw_parameters: &'a DrawParameters<'a>,
    screen_ratio: f64,
    optimal_screen_ratio: f64,
}

impl<'a> SceneRenderer<'a> {
    pub(crate) fn new(
        frame: &'a mut Frame,
        program: &'a Program,
        draw_parameters: &'a DrawParameters<'a>,
        screen_ratio: f64,
        optimal_screen_ratio: f64,
    ) -> Self {
        SceneRenderer {
            frame,
            program,
            draw_parameters,
            screen_ratio,
            optimal_screen_ratio,
        }
    }

    pub fn set_scene_settings(self, settings: &SceneSettings) -> SceneObjectRenderer<'a> {
        // clear frame with color from scene settings
        self.frame.clear_color(
            settings.background_color.x,
            settings.background_color.y,
            settings.background_color.z,
            1.0
        );
        self.frame.clear_depth(1.0);

        // move content to object renderer and return it
        SceneObjectRenderer {
            frame: self.frame,
            program: self.program,
            draw_parameters: self.draw_parameters,
            world_to_screen_matrix: settings.camera.as_matrix(
                self.screen_ratio,
                self.optimal_screen_ratio,
            ),
            ambient_light_color: settings.ambient_light_color,
            directional_light_dir: settings.directional_light_dir,
            directional_light_color: settings.directional_light_color,
        }
    }


}

