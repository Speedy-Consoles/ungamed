mod graphics;

use std::time::Instant;
use std::time::Duration;
use std::hash::Hash;
use std::str::FromStr;

use glium::glutin::Event as WinitEvent;
use glium::glutin::EventsLoop;
use glium::glutin::WindowEvent;

use controls::Controls;
use self::graphics::Graphics;

pub use controls::ControlBind;
pub use controls::ControlEvent;
pub use controls::FireTrigger;
pub use controls::HoldableTrigger;
pub use controls::ValueTrigger;
pub use controls::ValueTargetTrait;
pub use controls::VirtualKeyCode;

pub use cgmath;
pub use glium::glutin::dpi::LogicalSize;
pub use self::graphics::SceneObject;
pub use self::graphics::create::SceneObjectCreator;
pub use self::graphics::render::SceneSettings;
pub use self::graphics::render::SceneRenderer;
pub use self::graphics::render::SceneObjectRenderer;
pub use self::graphics::render::Projection;
pub use self::graphics::render::Camera;

#[derive(Debug)]
pub enum Event<FireTarget, SwitchTarget, ValueTarget> {
    ControlEvent(ControlEvent<FireTarget, SwitchTarget, ValueTarget>),
    CloseRequested,
}

pub trait Game {
    type FireTarget: Copy + Eq + Hash + FromStr + ToString;
    type SwitchTarget: Copy + Eq + Hash + FromStr + ToString;
    type ValueTarget: ValueTargetTrait + Copy + Eq + Hash + FromStr + ToString;
    fn title() -> &'static str;
    fn optimal_window_size() -> LogicalSize;
    fn new(
        scene_object_creator: SceneObjectCreator,
        binds: &mut Vec<ControlBind<Self::FireTarget, Self::SwitchTarget, Self::ValueTarget>>
    ) -> Self;
    fn handle_event(
        &mut self,
        event: Event<Self::FireTarget, Self::SwitchTarget, Self::ValueTarget>
    );
    fn tick(&mut self);
    fn render(&self, renderer: SceneRenderer);
    fn finished(&self) -> bool;
}

pub struct Application<G: Game> {
    game: G,
    graphics: Graphics,
    events_loop: EventsLoop,
    controls: Controls<G::FireTarget, G::SwitchTarget, G::ValueTarget>
}

impl<G: Game> Application<G> {
    pub fn new() -> Self {
        let events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_dimensions(G::optimal_window_size())
            .with_title(G::title());
        let context = glium::glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();
        let mut binds = Vec::new();
        let mut graphics = Graphics::new(display, G::optimal_window_size());
        let game = G::new(graphics.object_creator(), &mut binds);
        let mut controls = Controls::new();
        binds.into_iter().for_each(|bind| controls.add_bind(bind));
        Self {
            game,
            graphics,
            events_loop,
            controls,
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
            let graphics = &mut self.graphics;
            let controls = &mut self.controls;
            self.events_loop.poll_events(|event| {
                match event {
                    WinitEvent::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                        graphics.set_view_port_size(size);
                    },
                    WinitEvent::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                        game.handle_event(Event::CloseRequested);
                    }
                    WinitEvent::DeviceEvent { event, device_id } => controls.process(device_id, event),
                    _ => (),
                }
                controls.get_events().for_each(|e| game.handle_event(Event::ControlEvent(e)));
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

    use strum_macros::EnumString;
    use strum_macros::ToString;

    use crate::Application;
    use crate::ControlBind;
    use crate::Game;
    use crate::Event;
    use crate::LogicalSize;
    use crate::SceneObjectCreator;
    use crate::SceneRenderer;
    use crate::SceneObject;
    use crate::cgmath::Vector3;
    use crate::cgmath::Matrix4;
    use crate::cgmath::Rad;
    use crate::ValueTargetTrait;

    const NUM_TICKS: u64 = 81;
    const TICK_RATE: u32 = 50;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, ToString, EnumString)]
    enum FireTarget {
        LMBFire,
        MWUpFire,
        MWDownFire,
        GHFire,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, ToString, EnumString)]
    enum SwitchTarget {
        RMBSwitch,
        GHSwitch,
        Key0Switch,
        AMMBSwitch,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, ToString, EnumString)]
    enum ValueTarget {
        MouseX,
    }

    impl ValueTargetTrait for ValueTarget {
        fn base_factor(&self) -> f64 {
            1.0
        }
    }

    struct TestGame {
        cube: SceneObject,
        cube_rotation: f32,
        num_ticks: u64,
        num_renders: Cell<u64>,
    }

    impl Game for TestGame {
        type FireTarget = FireTarget;
        type SwitchTarget = SwitchTarget;
        type ValueTarget = ValueTarget;

        fn title() -> &'static str {
            "Test Game"
        }

        fn optimal_window_size() -> LogicalSize {
            LogicalSize::new(50.0, 50.0)
        }

        fn new(
            mut scene_object_creator: SceneObjectCreator,
            _binds: &mut Vec<ControlBind<FireTarget, SwitchTarget, ValueTarget>>,
        ) -> Self {
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

        fn handle_event(&mut self, event: Event<FireTarget, SwitchTarget, ValueTarget>) {
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
