mod graphics;

use std::time::Instant;
use std::time::Duration;
use std::hash::Hash;
use std::str::FromStr;

use glium::glutin::event::Event as WinitEvent;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::event::WindowEvent;
use glium::glutin::event_loop::ControlFlow;

use controls::Controls;
use self::graphics::Graphics;

pub use controls::ControlBind;
pub use controls::ControlEvent;
pub use controls::SwitchState;
pub use controls::FireTrigger;
pub use controls::HoldableTrigger;
pub use controls::ValueTrigger;
pub use controls::ValueTargetTrait;
pub use controls::VirtualKeyCode;

pub use cgmath;
pub use image;
pub use glium::glutin::dpi::LogicalSize;
pub use glium::texture::Texture2d;
pub use self::graphics::color::Color;
pub use self::graphics::TexturelessSceneObject;
pub use self::graphics::TexturedSceneObject;
pub use self::graphics::create::SceneObjectCreator;
pub use self::graphics::render::SceneSettings;
pub use self::graphics::render::SceneRenderer;
pub use self::graphics::render::SceneObjectRenderer;
pub use self::graphics::render::Projection;
pub use self::graphics::render::Camera;
pub use self::graphics::render::TEXT_NUM_LINES;
use std::collections::vec_deque::VecDeque;

#[derive(Debug)]
pub enum Event<FireTarget, SwitchTarget, ValueTarget> {
    ControlEvent(ControlEvent<FireTarget, SwitchTarget, ValueTarget>),
    WindowFocusChanged(bool),
    CloseRequested,
    GameEnded,
}

pub enum AppTransition<G> {
    NewSinglePlayer(G),
    PauseSinglePlayer,
    ResumeSinglePlayer,
    EndSinglePlayer,
    CloseApplication,
    None,
}

pub trait Application {
    type FireTarget: Copy + Eq + Hash + FromStr + ToString;
    type SwitchTarget: Copy + Eq + Hash + FromStr + ToString;
    type ValueTarget: ValueTargetTrait + Copy + Eq + Hash + FromStr + ToString;
    type G: Game;

    fn title() -> &'static str;
    fn optimal_window_size() -> LogicalSize;
    fn new(
        scene_object_creator: SceneObjectCreator,
        binds: &mut Vec<ControlBind<Self::FireTarget, Self::SwitchTarget, Self::ValueTarget>>
    ) -> Self;

    fn handle_event(
        &mut self,
        event: Event<Self::FireTarget, Self::SwitchTarget, Self::ValueTarget>,
        game_info: Option<GameInfo<Self::G>>,
    ) -> AppTransition<Self::G>;
    fn render(&self, game: Option<GameInfo<Self::G>>, renderer: SceneRenderer);
}

#[derive(Copy, Clone)]
pub enum GameStatus {
    Running,
    Ended,
}

pub trait Game {
    fn update(&mut self) -> GameStatus;
}

pub struct GameInfo<'a, G: Game> {
    game: &'a G,
    paused: bool,
}

struct GameData<G: Game> {
    game: G,
    pause_start: Option<Instant>,
    update_ref_time: Instant,
    num_updates: u64,
    update_rate: u32,
}

impl<G: Game> GameData<G> {
    fn game_info(&self) -> GameInfo<G> {
        GameInfo {
            game: &self.game,
            paused: self.paused(),
        }
    }

    fn paused(&self) -> bool {
        self.pause_start.is_some()
    }
}

struct AppWrapper<A: Application> {
    application: A,
    controls: Controls<A::FireTarget, A::SwitchTarget, A::ValueTarget>,
    graphics: Graphics,
    num_renders: u64,
    render_rate: u32,
    render_phase: Duration,
    game_data: Option<GameData<A::G>>,
    render_ref_time: Instant,
    closing: bool,
}

impl<A: Application> AppWrapper<A> {
    fn emit_event(
        &mut self,
        event: Event<A::FireTarget, A::SwitchTarget, A::ValueTarget>
    ) {
        let game_info = self.game_data.as_ref().map(|gd| gd.game_info());
        // TODO checking app transitions is ugly.
        // TODO Maybe pass some kind of control structure to handle_event.
        match self.application.handle_event(event, game_info) {
            AppTransition::NewSinglePlayer(g) => {
                match self.game_data {
                    None => {
                        let now = Instant::now();
                        self.game_data = Some(GameData {
                            game: g,
                            pause_start: None,
                            update_ref_time: now,
                            num_updates: 0,
                            update_rate: 50,
                        });
                    },
                    Some(_) => panic!("We already have a game!"),
                }
            },
            AppTransition::EndSinglePlayer => {
                assert!(self.game_data.is_some(), "No game to end!");
                self.game_data = None;
            },
            AppTransition::CloseApplication => self.closing = true,
            AppTransition::PauseSinglePlayer => {
                match self.game_data {
                    Some(ref mut gd) => {
                        assert!(gd.pause_start.is_none(), "Game already paused!");
                        gd.pause_start = Some(Instant::now())
                    },
                    None => panic!("No game to pause!"),
                }
            },
            AppTransition::ResumeSinglePlayer => {
                match self.game_data {
                    Some(ref mut gd) => {
                        match gd.pause_start {
                            Some(start) => {
                                gd.update_ref_time += Instant::now() - start;
                                gd.pause_start = None;
                            },
                            None => panic!("Game already running!"),
                        }
                    },
                    None => panic!("No game to resume!"),
                }
            },
            AppTransition::None => (),
        }
    }

    fn handle_event(&mut self, event: WinitEvent<()>) {
        match event {
            WinitEvent::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                self.graphics.set_view_port_size(size);
            },
            WinitEvent::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                self.emit_event(Event::CloseRequested);
            },
            WinitEvent::WindowEvent { event: WindowEvent::Focused(focused), .. } => {
                self.emit_event(Event::WindowFocusChanged(focused));
            },
            WinitEvent::DeviceEvent { event, device_id } => {
                self.controls.process(device_id, event);
            },
            _ => ()//eprintln!("{:?}", event),
        }
    }

    fn update_game(&mut self) -> Option<Instant> {
        if let Some(ref mut gd) = self.game_data {
            if gd.paused() {
                return None;
            }
            let mut next_update_time = Self::next_tick_time(
                gd.update_ref_time,
                gd.num_updates,
                gd.update_rate,
            );
            if Instant::now() >= next_update_time {
                let status = gd.game.update();
                if let GameStatus::Ended = status {
                    self.emit_event(Event::GameEnded);
                    self.game_data = None;
                } else {
                    gd.num_updates += 1;
                    if gd.paused() {
                        return None
                    }
                    next_update_time = Self::next_tick_time(
                        gd.update_ref_time,
                        gd.num_updates,
                        gd.update_rate,
                    );
                }
            }
            return Some(next_update_time);
        }
        None
    }

    fn render(&mut self) -> Instant {
        let mut next_render_time = Self::next_tick_time(
            self.render_ref_time + self.render_phase,
            self.num_renders,
            self.render_rate,
        );
        if Instant::now() >= next_render_time {
            let game_info = self.game_data.as_ref().map(|gd| gd.game_info());
            self.graphics.render(&self.application, game_info);
            self.num_renders += 1;
            // TODO adapt render_phase and render_rate
            next_render_time = Self::next_tick_time(
                self.render_ref_time + self.render_phase,
                self.num_renders,
                self.render_rate,
            );
        }
        next_render_time
    }

    fn next_tick_time(ref_time: Instant, num_ticks: u64, tick_rate: u32) -> Instant {
        ref_time + Duration::from_secs(num_ticks) / tick_rate
    }
}

pub fn run_application<A: Application + 'static>() -> ! {
    // creating structures
    let event_loop = EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(A::optimal_window_size())
        .with_title(A::title());
    let context = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &event_loop).unwrap();
    let mut binds = Vec::new();
    let mut graphics = Graphics::new(display, A::optimal_window_size());
    let mut controls = Controls::new();
    let application = A::new(graphics.object_creator(), &mut binds);
    binds.into_iter().for_each(|bind| controls.add_bind(bind));
    let mut app_wrapper = AppWrapper {
        application,
        controls,
        graphics,
        num_renders: 0,
        render_rate: 60,
        render_phase: Duration::from_secs(0),
        game_data: None,
        render_ref_time: Instant::now(),
        closing: false,
    };

    // main loop
    let mut event_buffer = VecDeque::new();
    event_loop.run(move |event, _, control_flow| {
        app_wrapper.handle_event(event);
        if app_wrapper.closing {
            *control_flow = ControlFlow::Exit;
            return;
        };

        // this must not be in the device event branch of handle_event,
        // because events may also be produced by binding/unbinding
        app_wrapper.controls.get_events(&mut event_buffer);
        for event in event_buffer.drain(..) {
            app_wrapper.emit_event(Event::ControlEvent(event));
        }

        // update the game
        let mut next_loop_time = app_wrapper.update_game();

        // render the screen
        let next_render_time = app_wrapper.render();
        match next_loop_time {
            None => next_loop_time = Some(next_render_time),
            Some(ref nlt) if *nlt > next_render_time => next_loop_time = Some(next_render_time),
            _ => (),
        }

        // schedule next loop
        if let Some(nlt) = next_loop_time {
            if Instant::now() < nlt {
                *control_flow = ControlFlow::WaitUntil(nlt);
            } else {
                // ControlFlow::Poll seems to skip fetching window events
                //*control_flow = ControlFlow::Poll;
                *control_flow = ControlFlow::WaitUntil(nlt);
            }
        }
    });
}


#[cfg(test)]
mod tests {
    //use std::time::Instant;
    //use std::time::Duration;
    use std::cell::Cell;
    use std::io::Cursor;

    use strum_macros::EnumString;
    use strum_macros::ToString;

    use crate::{Application, VirtualKeyCode, GameInfo};
    use crate::AppTransition;
    use crate::run_application;
    use crate::GameStatus;
    use crate::FireTrigger;
    use crate::HoldableTrigger;
    use crate::ValueTrigger;
    use crate::ControlBind;
    use crate::ControlEvent;
    use crate::Game;
    use crate::Event;
    use crate::LogicalSize;
    use crate::Color;
    use crate::SceneObjectCreator;
    use crate::SceneRenderer;
    use crate::TexturelessSceneObject;
    use crate::TexturedSceneObject;
    use crate::cgmath::Vector3;
    use crate::cgmath::Vector2;
    use crate::cgmath::Matrix4;
    use crate::cgmath::Rad;
    use crate::ValueTargetTrait;
    use crate::Texture2d;
    use crate::TEXT_NUM_LINES;

    const NUM_TICKS: u64 = 131;
    //const TICK_RATE: u32 = 50;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, ToString, EnumString)]
    enum FireTarget {
        StartGame,
        EndGame,
        ToggleGamePause,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, ToString, EnumString)]
    enum SwitchTarget {
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, ToString, EnumString)]
    enum ValueTarget {
    }

    impl ValueTargetTrait for ValueTarget {
        fn base_factor(&self) -> f64 {
            1.0
        }
    }

    #[derive(Clone)]
    struct TestGame {
        cube_rotation: f32,
        num_updates: u64,
    }

    impl Game for TestGame {
        fn update(&mut self) -> GameStatus {
            self.cube_rotation += 0.05;
            self.num_updates += 1;
            GameStatus::Running
        }
    }

    struct TestApplication {
        textured_cube: TexturedSceneObject,
        textureless_cube: TexturelessSceneObject,
        cube_texture: Texture2d,
        num_renders: Cell<u64>,
    }

    impl Application for TestApplication {
        type FireTarget = FireTarget;
        type SwitchTarget = SwitchTarget;
        type ValueTarget = ValueTarget;
        type G = TestGame;

        fn title() -> &'static str {
            "Test Game"
        }

        fn optimal_window_size() -> LogicalSize {
            LogicalSize::new(50.0, 50.0)
        }

        fn new(
            mut scene_object_creator: SceneObjectCreator,
            binds: &mut Vec<ControlBind<FireTarget, SwitchTarget, ValueTarget>>,
        ) -> Self {

            binds.push(ControlBind::Fire(FireTrigger::Holdable(HoldableTrigger::Button(1)), FireTarget::StartGame));
            binds.push(ControlBind::Fire(FireTrigger::Holdable(HoldableTrigger::Button(3)), FireTarget::EndGame));
            binds.push(ControlBind::Fire(FireTrigger::Holdable(HoldableTrigger::KeyCode(VirtualKeyCode::Space)), FireTarget::ToggleGamePause));

            let textured_vertices = [
                (Vector3::new(-0.5, -0.5,  0.5), Vector2::new(0.0, 0.0)),
                (Vector3::new( 0.5, -0.5,  0.5), Vector2::new(1.0, 0.0)),
                (Vector3::new(-0.5,  0.5,  0.5), Vector2::new(0.0, 1.0)),
                (Vector3::new( 0.5,  0.5,  0.5), Vector2::new(1.0, 1.0)),

                (Vector3::new( 0.5, -0.5, -0.5), Vector2::new(0.0, 0.0)),
                (Vector3::new(-0.5, -0.5, -0.5), Vector2::new(1.0, 0.0)),
                (Vector3::new( 0.5,  0.5, -0.5), Vector2::new(0.0, 1.0)),
                (Vector3::new(-0.5,  0.5, -0.5), Vector2::new(1.0, 1.0)),

                (Vector3::new(-0.5, -0.5, -0.5), Vector2::new(0.0, 0.0)),
                (Vector3::new( 0.5, -0.5, -0.5), Vector2::new(1.0, 0.0)),
                (Vector3::new(-0.5, -0.5,  0.5), Vector2::new(0.0, 1.0)),
                (Vector3::new( 0.5, -0.5,  0.5), Vector2::new(1.0, 1.0)),

                (Vector3::new( 0.5,  0.5, -0.5), Vector2::new(0.0, 0.0)),
                (Vector3::new(-0.5,  0.5, -0.5), Vector2::new(1.0, 0.0)),
                (Vector3::new( 0.5,  0.5,  0.5), Vector2::new(0.0, 1.0)),
                (Vector3::new(-0.5,  0.5,  0.5), Vector2::new(1.0, 1.0)),

                (Vector3::new(-0.5,  0.5, -0.5), Vector2::new(0.0, 0.0)),
                (Vector3::new(-0.5, -0.5, -0.5), Vector2::new(1.0, 0.0)),
                (Vector3::new(-0.5,  0.5,  0.5), Vector2::new(0.0, 1.0)),
                (Vector3::new(-0.5, -0.5,  0.5), Vector2::new(1.0, 1.0)),

                (Vector3::new( 0.5, -0.5, -0.5), Vector2::new(0.0, 0.0)),
                (Vector3::new( 0.5,  0.5, -0.5), Vector2::new(1.0, 0.0)),
                (Vector3::new( 0.5, -0.5,  0.5), Vector2::new(0.0, 1.0)),
                (Vector3::new( 0.5,  0.5,  0.5), Vector2::new(1.0, 1.0)),
            ];

            let textureless_vertices: Vec<Vector3<f32>>
                = textured_vertices.iter().map(|&(p, _)| p).collect();

            let indices = [
                 0,  1,  3,   0,  3,  2,
                 4,  5,  7,   4,  7,  6,
                 8,  9, 11,   8, 11, 10,
                12, 13, 15,  12, 15, 14,
                16, 17, 19,  16, 19, 18,
                20, 21, 23,  20, 23, 22,
            ];

            let cube_texture = scene_object_creator.create_texture(image::load(
                Cursor::new(&include_bytes!("../images/test_image.png")[..]),
                image::PNG,
            ).unwrap());

            let textured_cube = scene_object_creator.create_textured(
                &textured_vertices,
                &indices
            );
            let textureless_cube = scene_object_creator.create_textureless(
                textureless_vertices.as_ref(),
                &indices
            );

            TestApplication {
                textured_cube,
                textureless_cube,
                cube_texture,
                num_renders: Cell::new(0),
            }
        }

        fn handle_event(
            &mut self,
            event: Event<FireTarget, SwitchTarget, ValueTarget>,
            game_info: Option<GameInfo<Self::G>>
        ) -> AppTransition<TestGame> {
            match event {
                Event::ControlEvent(ce) => match ce {
                    ControlEvent::Fire(FireTarget::StartGame) => {
                        if game_info.is_none() {
                            AppTransition::NewSinglePlayer(TestGame {
                                cube_rotation: 0.0,
                                num_updates: 0,
                            })
                        } else {
                            AppTransition::None
                        }
                    },
                    ControlEvent::Fire(FireTarget::EndGame) => {
                        if game_info.is_some() {
                            AppTransition::EndSinglePlayer
                        } else {
                            AppTransition::None
                        }
                    },
                    ControlEvent::Fire(FireTarget::ToggleGamePause) => {
                        if let Some(GameInfo { game: _, paused }) = game_info {
                            if paused {
                                AppTransition::ResumeSinglePlayer
                            } else {
                                AppTransition::PauseSinglePlayer
                            }
                        } else {
                            AppTransition::None
                        }
                    },
                    ControlEvent::Switch { .. } => AppTransition::None,
                    ControlEvent::Value { .. } => AppTransition::None,
                },
                Event::GameEnded | Event::WindowFocusChanged(_) => AppTransition::None,
                Event::CloseRequested => AppTransition::CloseApplication,
            }
        }

        fn render(&self, game_info: Option<GameInfo<TestGame>>, renderer: SceneRenderer) {
            let mut renderer = renderer.start_object_rendering(&Default::default());
            if let Some(GameInfo { game, paused: _ }) = game_info {
                renderer.draw_textured(
                    &self.textured_cube,
                    &self.cube_texture,
                    &Matrix4::from_angle_z(Rad(game.cube_rotation)),
                );
                let x_cube = Matrix4::from_translation(Vector3::unit_x()) * Matrix4::from_scale(0.05);
                let z_cube = Matrix4::from_translation(Vector3::unit_y()) * Matrix4::from_scale(0.2);
                let y_cube = Matrix4::from_translation(Vector3::unit_z()) * Matrix4::from_scale(0.5);
                renderer.draw_textureless(&self.textureless_cube, Color::red(), &x_cube);
                renderer.draw_textureless(&self.textureless_cube, Color::green(), &y_cube);
                renderer.draw_textureless(&self.textureless_cube, Color::blue(), &z_cube);
                let mut renderer = renderer.start_text_rendering();
                for i in 0..TEXT_NUM_LINES {
                    renderer.draw_text(i, &format!("line {}", i));
                }
                self.num_renders.set(self.num_renders.get() + 1);
            }
        }
    }

    #[test]
    fn test_all() {
        // TODO move the checks into run
        //let start_time = Instant::now();
        run_application::<TestApplication>();
        /*let duration = Instant::now() - start_time;

        assert_eq!(app.game.num_ticks, NUM_TICKS);
        assert!(
            app.game.num_renders.get() >= NUM_TICKS,
            "left: {}, right: {}",
            app.game.num_renders.get(),
            NUM_TICKS
        );

        const NUM_MILLIS: u64 = (NUM_TICKS - 1) * 1000 / TICK_RATE as u64;
        const TARGET_DURATION: Duration = Duration::from_millis(NUM_MILLIS);
        let diff = if duration > TARGET_DURATION {
            duration - TARGET_DURATION
        } else {
            TARGET_DURATION - duration
        };
        assert!(
            diff < Duration::from_millis(20),
            "run duration differs: {:?} != {:?}",
            duration,
            TARGET_DURATION
        );*/
    }
}
