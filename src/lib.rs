mod graphics;

extern crate glium;
pub extern crate cgmath;

use std::time::Instant;
use std::time::Duration;

use glium::glutin::Event;
use glium::glutin::EventsLoop;
use glium::glutin::WindowEvent;
use glium::glutin::dpi::LogicalSize;

use self::graphics::Graphics;

pub use glium::glutin; // TODO can we eliminate this re-export?
pub use self::graphics::SceneObject;
pub use self::graphics::create::SceneObjectCreator;
pub use self::graphics::render::SceneSettings;
pub use self::graphics::render::SceneRenderer;
pub use self::graphics::render::SceneObjectRenderer;
pub use self::graphics::render::Projection;
pub use self::graphics::render::Camera;

pub trait Game {
    fn title() -> &'static str;
    fn optimal_window_size() -> LogicalSize;
    fn new(scene_object_creator: SceneObjectCreator) -> Self;
    fn handle_event(&mut self, event: Event);
    fn tick(&mut self);
    fn render(&self, renderer: SceneRenderer);
    fn finished(&self) -> bool;
}

pub struct Application<G: Game> {
    game: G,
    graphics: Graphics,
    events_loop: EventsLoop,
}

impl<G: Game> Application<G> {
    pub fn new() -> Self {
        let events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_dimensions(G::optimal_window_size())
            .with_title(G::title());
        let context = glium::glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();
        let mut scene = Graphics::new(display, G::optimal_window_size());
        Self {
            game: G::new(scene.object_creator()),
            graphics: scene,
            events_loop,
        }
    }

    pub fn run(&mut self, tick_rate: u32) {
        let now = Instant::now();
        let start_time = now;
        let mut num_ticks = 0;
        let mut num_renders = 0;
        let render_rate = 60;
        let render_phase = Duration::from_secs(0);
        let mut next_tick_time = now;
        let mut next_render_time = now;

        loop {
            let game = &mut self.game;
            let scene = &mut self.graphics;
            self.events_loop.poll_events(|event| {
                match event {
                    Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                        scene.set_view_port_size(size);
                    },
                    _ => (),
                }
                game.handle_event(event);
            });

            if Instant::now() >= next_tick_time {
                self.game.tick();
                num_ticks += 1;
                next_tick_time = start_time + Duration::from_secs(num_ticks) / tick_rate;
            }
            if Instant::now() >= next_render_time {
                self.graphics.render(&self.game);
                num_renders += 1;
                // TODO adapt render_phase and render_rate
                next_render_time = start_time + render_phase
                    + Duration::from_secs(num_renders) / render_rate;
            }
            if self.game.finished() {
                break;
            }
            let now = Instant::now();
            let next_loop_time = next_tick_time.min(next_render_time);
            if now < next_loop_time {
                std::thread::sleep(next_loop_time - now);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::time::Instant;
    use std::time::Duration;
    use std::cell::Cell;

    use crate::Game;
    use crate::Application;
    use crate::LogicalSize;
    use crate::Event;
    use crate::SceneObjectCreator;
    use crate::SceneRenderer;
    use crate::SceneObject;
    use crate::cgmath::Vector3;
    use crate::cgmath::Matrix4;
    use crate::cgmath::Rad;

    const NUM_TICKS: u64 = 81;
    const TICK_RATE: u32 = 50;

    struct TestGame {
        cube: SceneObject,
        cube_rotation: f32,
        num_ticks: u64,
        num_renders: Cell<u64>,
    }

    impl Game for TestGame {
        fn title() -> &'static str {
            "Test Game"
        }

        fn optimal_window_size() -> LogicalSize {
            LogicalSize::new(50.0, 50.0)
        }

        fn new(mut scene_object_creator: SceneObjectCreator) -> Self {
            let vertices = [
                Vector3::new(-0.5,  0.5, -0.5),
                Vector3::new(-0.5, -0.5, -0.5),
                Vector3::new(-0.5, -0.5,  0.5),
                Vector3::new(-0.5,  0.5,  0.5),
                Vector3::new( 0.5,  0.5, -0.5),
                Vector3::new( 0.5, -0.5, -0.5),
                Vector3::new( 0.5, -0.5,  0.5),
                Vector3::new( 0.5,  0.5,  0.5),
            ];

            let indices = [
                0, 1, 2,  0, 2, 3,
                1, 5, 6,  1, 6, 2,
                5, 4, 7,  5, 7, 6,
                4, 0, 3,  4, 3, 7,
                0, 4, 5,  0, 5, 1,
                3, 2, 6,  3, 6, 7u32,
            ];
            TestGame {
                cube: scene_object_creator.create(&vertices, &indices),
                cube_rotation: 0.0,
                num_ticks: 0,
                num_renders: Cell::new(0),
            }
        }

        fn handle_event(&mut self, event: Event) {
            eprintln!("{:?}", event);
        }

        fn tick(&mut self) {
            self.cube_rotation += 0.05;
            self.num_ticks += 1;
        }

        fn render(&self, renderer: SceneRenderer) {
            let mut renderer = renderer.set_scene_settings(&Default::default());
            renderer.draw(&self.cube, &Matrix4::from_angle_z(Rad (self.cube_rotation)));
            let x_cube = Matrix4::from_translation(Vector3::unit_x()) * Matrix4::from_scale(0.05);
            let z_cube = Matrix4::from_translation(Vector3::unit_y()) * Matrix4::from_scale(0.2);
            let y_cube = Matrix4::from_translation(Vector3::unit_z()) * Matrix4::from_scale(0.5);
            renderer.draw(&self.cube, &x_cube);
            renderer.draw(&self.cube, &y_cube);
            renderer.draw(&self.cube, &z_cube);
            self.num_renders.set(self.num_renders.get() + 1);
        }

        fn finished(&self) -> bool {
            self.num_ticks >= NUM_TICKS
        }
    }

    #[test]
    fn test_all() {
        let mut app: Application<TestGame> = Application::new();

        let start_time = Instant::now();
        app.run(TICK_RATE);
        let duration = Instant::now() - start_time;

        assert_eq!(app.game.num_ticks, NUM_TICKS);
        assert!(
            app.game.num_renders.get() >= NUM_TICKS,
            "left: {}, right: {}",
            app.game.num_renders.get(),
            NUM_TICKS
        );

        const NUM_MILLIS: u64 = (NUM_TICKS - 1) * 1000 / TICK_RATE as u64;
        const TARGET_DURATION: Duration = Duration::from_millis(NUM_MILLIS);
        if duration > TARGET_DURATION {
            assert!(
                duration - TARGET_DURATION < Duration::from_millis(10),
                "left: {:?}, right: {:?}",
                duration,
                TARGET_DURATION
            );
        } else {
            assert!(
                TARGET_DURATION - duration < Duration::from_millis(10),
                "left: {:?}, right: {:?}",
                duration,
                TARGET_DURATION
            );
        }
    }
}
