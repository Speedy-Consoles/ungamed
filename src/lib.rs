mod graphics;

use std::time::Instant;
use std::time::Duration;
use std::hash::Hash;
use std::str::FromStr;
use std::collections::vec_deque::VecDeque;

use glium::glutin::event::Event as WinitEvent;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::event::WindowEvent;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::window::Window;
use glium::glutin::dpi::LogicalPosition;
use glium::Display;

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
pub use self::graphics::TexturelessSceneObject3d;
pub use self::graphics::TexturedSceneObject3d;
pub use self::graphics::TexturelessSceneObject2d;
pub use self::graphics::TexturedSceneObject2d;
pub use self::graphics::create::SceneObjectCreator;
pub use self::graphics::render::SceneSettings;
pub use self::graphics::render::SceneRenderer;
pub use self::graphics::render::SceneObjectRenderer;
pub use self::graphics::render::Projection;
pub use self::graphics::render::Camera;
pub use self::graphics::render::TEXT_NUM_LINES;
pub use self::graphics::render::OverlayAlignment;

#[derive(Debug)]
pub enum Event<FireTarget, SwitchTarget, ValueTarget> {
    ControlEvent(ControlEvent<FireTarget, SwitchTarget, ValueTarget>),
    WindowFocusChanged(bool),
    CloseRequested,
    GameUpdated,
    CursorMoved,
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
        controller: ApplicationController<Self>,
    );

    fn render(
        &self,
        game_info: Option<GameInfo<Self::G>>,
        graphics_info: GraphicsInfo,
        renderer: SceneRenderer
    );
}

#[derive(Copy, Clone)]
pub enum GameStatus {
    Running,
    Ended,
}

pub trait Game {
    fn update(&mut self) -> GameStatus;
}

pub struct ApplicationController<'a, A: ?Sized + Application> {
    pub game_controller: GameController<'a, A::G>,
    pub cursor_controller: CursorController<'a>,
    closing: &'a mut bool,
}

impl<'a, A: Application> ApplicationController<'a, A> {
    pub fn close(self) {
        *self.closing = true;
    }
}

pub struct GameInfo<'a, G: Game> {
    pub game: &'a G,
    pub paused: bool,
    pub ended: bool,
}

pub struct RunningGameController<'a, G: Game> {
    game_data: &'a mut Option<GameData<G>>,
}

impl<'a, G: Game> RunningGameController<'a, G> {
    pub fn pause(self) -> PausedGameController<'a, G> {
        self.game_data.as_mut().unwrap().pause_start = Some(Instant::now());
        PausedGameController { game_data: self.game_data }
    }

    pub fn close(self) -> ClosedGameController<'a, G> {
        *self.game_data = None;
        ClosedGameController { game_data: self.game_data }
    }
}

pub struct PausedGameController<'a, G: Game> {
    game_data: &'a mut Option<GameData<G>>,
}

impl<'a, G: Game> PausedGameController<'a, G> {
    pub fn resume(self) -> RunningGameController<'a, G> {
        let gd = self.game_data.as_mut().unwrap();
        gd.update_ref_time += Instant::now() - gd.pause_start.unwrap();
        gd.pause_start = None;
        RunningGameController { game_data: self.game_data }
    }

    pub fn close(self) -> ClosedGameController<'a, G> {
        *self.game_data = None;
        ClosedGameController { game_data: self.game_data }
    }
}

pub struct EndedGameController<'a, G: Game> {
    game_data: &'a mut Option<GameData<G>>,
}

impl<'a, G: Game> EndedGameController<'a, G> {
    pub fn close(self) -> ClosedGameController<'a, G> {
        *self.game_data = None;
        ClosedGameController { game_data: self.game_data }
    }
}

pub struct ClosedGameController<'a, G: Game> {
    game_data: &'a mut Option<GameData<G>>,
}

impl<'a, G: Game> ClosedGameController<'a, G> {
    pub fn start_new(self, game: G) -> RunningGameController<'a, G> {
        *self.game_data = Some(GameData {
            game,
            pause_start: None,
            ended: false,
            update_ref_time: Instant::now(),
            num_updates: 0,
            update_rate: 50,
        });
        RunningGameController { game_data: self.game_data }
    }
}

pub enum GameController<'a, G: Game> {
    Running(RunningGameController<'a, G>),
    Paused(PausedGameController<'a, G>),
    Ended(EndedGameController<'a, G>),
    Closed(ClosedGameController<'a, G>),
}

pub struct FreeCursorController<'a> {
    cursor_data: &'a mut CursorData,
    window: &'a Window,
}

impl<'a> FreeCursorController<'a> {
    pub fn position(&self) -> (f64, f64) { // TODO should this be relative to window size?
        (self.cursor_data.pos.x, self.cursor_data.pos.y)
    }

    pub fn capture(self) -> CapturedCursorController<'a> {
        self.window.set_cursor_grab(true).ok(); // TODO what to do on error?
        self.cursor_data.mode = CursorMode::Captured;
        CapturedCursorController {
            cursor_data: self.cursor_data,
            window: self.window,
        }
    }

    pub fn hide(self) -> HiddenCursorController<'a> {
        self.window.set_cursor_visible(false);
        self.window.set_cursor_grab(true).ok(); // TODO what to do on error?
        self.cursor_data.mode = CursorMode::Hidden;
        HiddenCursorController {
            cursor_data: self.cursor_data,
            window: self.window,
        }
    }
}

pub struct CapturedCursorController<'a> {
    cursor_data: &'a mut CursorData,
    window: &'a Window,
}

impl<'a> CapturedCursorController<'a> {
    pub fn position(&self) -> (f64, f64) { // TODO should this be relative to window size?
        (self.cursor_data.pos.x, self.cursor_data.pos.y)
    }

    pub fn free(self) -> FreeCursorController<'a> {
        self.window.set_cursor_visible(true);
        self.window.set_cursor_grab(false).ok(); // TODO what to do on error?
        self.cursor_data.mode = CursorMode::Normal;
        FreeCursorController {
            cursor_data: self.cursor_data,
            window: self.window,
        }
    }

    pub fn hide(self) -> HiddenCursorController<'a> {
        self.window.set_cursor_visible(false);
        self.window.set_cursor_grab(true).ok(); // TODO what to do on error?
        self.cursor_data.mode = CursorMode::Hidden;
        HiddenCursorController {
            cursor_data: self.cursor_data,
            window: self.window,
        }
    }
}

pub struct HiddenCursorController<'a> {
    cursor_data: &'a mut CursorData,
    window: &'a Window,
}

impl<'a> HiddenCursorController<'a> {
    pub fn show(self) -> FreeCursorController<'a> {
        self.window.set_cursor_visible(true);
        self.window.set_cursor_grab(false).ok(); // TODO what to do on error?
        self.window.set_cursor_position(self.cursor_data.pos).ok(); // TODO what to do on error?
        self.cursor_data.mode = CursorMode::Normal;
        FreeCursorController {
            cursor_data: self.cursor_data,
            window: self.window,
        }
    }

    pub fn show_captured(self) -> CapturedCursorController<'a> {
        self.window.set_cursor_visible(true);
        self.window.set_cursor_grab(true).ok(); // TODO what to do on error?
        self.window.set_cursor_position(self.cursor_data.pos).ok(); // TODO what to do on error?
        self.cursor_data.mode = CursorMode::Captured;
        CapturedCursorController {
            cursor_data: self.cursor_data,
            window: self.window,
        }
    }
}

pub enum CursorController<'a> {
    Free(FreeCursorController<'a>),
    Captured(CapturedCursorController<'a>),
    Hidden(HiddenCursorController<'a>),
}

pub struct GraphicsInfo {
    pub fps: f32,
}

struct GameData<G: Game> {
    game: G,
    pause_start: Option<Instant>,
    ended: bool,
    update_ref_time: Instant,
    num_updates: u64,
    update_rate: u32,
}

impl<G: Game> GameData<G> {
    fn maybe_update(&mut self) -> bool {
        match self.next_update_time() {
            Some(nut) if nut <= Instant::now() => {
                if let GameStatus::Ended = self.game.update() {
                    self.ended = true;
                }
                self.num_updates += 1;
                true
            },
            _ => false,
        }
    }

    fn game_info(&self) -> GameInfo<G> {
        GameInfo {
            game: &self.game,
            paused: self.paused(),
            ended: self.ended,
        }
    }

    fn paused(&self) -> bool {
        self.pause_start.is_some()
    }

    fn next_update_time(&self) -> Option<Instant> {
        if self.paused() || self.ended {
            return None;
        }
        Some(next_tick_time(self.update_ref_time, self.num_updates, self.update_rate))
    }
}

struct GraphicsData {
    graphics: Graphics,
    render_ref_time: Instant,
    num_renders: u64,
    render_rate: u32,
    render_phase: Duration,
    last_render: Instant,
    fps: f32,
}

impl GraphicsData {
    fn maybe_render<A: Application>(
        &mut self,
        application: &A,
        display: &Display,
        game_info: Option<GameInfo<A::G>>,
    ) -> bool {
        let next_render_time = self.next_render_time();
        let now = Instant::now();
        if now >= next_render_time {
            // TODO adapt render_phase and render_rate properly
            let start = Instant::now();
            self.graphics.render(application, display, game_info, self.graphics_info());
            let render_duration = Instant::now() - start;
            if render_duration > Duration::from_millis(10) {
                self.render_phase += render_duration - Duration::from_millis(3);
            }
            self.fps = self.fps * 0.95 + 0.05 / (now - self.last_render).as_secs_f32();
            self.last_render = now;
            self.num_renders += 1;
            return true;
        }
        return false;
    }

    fn next_render_time(&self) -> Instant {
        next_tick_time(
            self.render_ref_time + self.render_phase,
            self.num_renders,
            self.render_rate
        )
    }

    fn graphics_info(&self) -> GraphicsInfo {
        GraphicsInfo {
            fps: self.fps,
        }
    }
}

#[derive(Eq, PartialEq)]
enum CursorMode {
    Normal,
    Captured,
    Hidden,
}

struct CursorData {
    pos: LogicalPosition,
    mode: CursorMode,
}

struct Engine<A: Application> {
    application: A,
    display: Display,
    controls: Controls<A::FireTarget, A::SwitchTarget, A::ValueTarget>,
    game_data: Option<GameData<A::G>>,
    graphics_data: GraphicsData,
    cursor_data: CursorData,
    closing: bool,
}

impl<A: Application> Engine<A> {
    fn emit_event(
        &mut self,
        event: Event<A::FireTarget, A::SwitchTarget, A::ValueTarget>,
    ) {
        let game_controller = match self.game_data {
            Some(ref gd) => {
                if gd.ended {
                    GameController::Ended(EndedGameController { game_data: &mut self.game_data })
                } else if gd.paused() {
                    GameController::Paused(PausedGameController { game_data: &mut self.game_data })
                } else {
                    GameController::Running(RunningGameController { game_data: &mut self.game_data })
                }
            },
            None => GameController::Closed(ClosedGameController { game_data: &mut self.game_data })
        };
        let gl_window = self.display.gl_window();
        let cursor_controller = match self.cursor_data.mode {
            CursorMode::Normal => CursorController::Free(FreeCursorController {
                cursor_data: &mut self.cursor_data,
                window: gl_window.window(),
            }),
            CursorMode::Captured => CursorController::Captured(CapturedCursorController {
                cursor_data: &mut self.cursor_data,
                window: gl_window.window(),
            }),
            CursorMode::Hidden => CursorController::Hidden(HiddenCursorController {
                cursor_data: &mut self.cursor_data,
                window: gl_window.window(),
            }),
        };
        let application_controller = ApplicationController {
            game_controller,
            cursor_controller,
            closing: &mut self.closing,
        };
        self.application.handle_event(event, application_controller)
    }

    fn handle_event(&mut self, event: WinitEvent<()>) {
        let my_window_id = self.display.gl_window().window().id();
        match event {
            WinitEvent::WindowEvent { event: we, window_id } => {
                if window_id == my_window_id {
                    match we {
                        WindowEvent::Resized(size) => {
                            self.graphics_data.graphics.set_view_port_size(size);
                        },
                        WindowEvent::CloseRequested => {
                            self.emit_event(Event::CloseRequested);
                        },
                        WindowEvent::Focused(focused) => {
                            if focused {
                                self.controls.resume()
                            } else {
                                self.controls.pause()
                            }
                            self.emit_event(Event::WindowFocusChanged(focused));
                        },
                        WindowEvent::Moved(_) => (),
                        WindowEvent::AxisMotion { .. } => (),
                        WindowEvent::CursorMoved { position, .. } => {
                            if self.cursor_data.mode != CursorMode::Hidden {
                                let old_pos = self.cursor_data.pos;
                                self.cursor_data.pos = position;
                                if self.cursor_data.pos != old_pos {
                                    self.emit_event(Event::CursorMoved);
                                }
                            }
                        },
                        WindowEvent::CursorEntered { .. } => (),
                        WindowEvent::CursorLeft { .. } => (),
                        _ => eprintln!("{:?}", we), // TODO
                    }
                } else {
                    eprintln!("wrong window ({:?}): {:?}", window_id, we);
                }
            },
            WinitEvent::DeviceEvent { event, device_id } => {
                self.controls.process(device_id, event);
            },
            WinitEvent::UserEvent(_) => (), // TODO
            WinitEvent::NewEvents(_) => (),
            WinitEvent::EventsCleared => (),
            WinitEvent::LoopDestroyed => (),
            WinitEvent::Suspended => (),
            WinitEvent::Resumed => (),
        }
    }

    fn maybe_update_game(&mut self) -> Option<Instant> {
        if let Some(ref mut gd) = self.game_data {
            if gd.maybe_update() {
                let next_update_time = gd.next_update_time();
                self.emit_event(Event::GameUpdated);
                return next_update_time;
            }
            return gd.next_update_time();
        }
        return None;
    }

    fn maybe_render(&mut self) -> Instant {
        self.graphics_data.maybe_render(
            &self.application,
            &self.display,
            self.game_data.as_ref().map(|gd| gd.game_info())
        );
        return self.graphics_data.next_render_time();
    }
}

fn next_tick_time(ref_time: Instant, num_ticks: u64, tick_rate: u32) -> Instant {
    ref_time + Duration::from_secs(num_ticks) / tick_rate
}

pub fn run_application<A: Application + 'static>() -> ! {
    // creating structures
    let event_loop = EventLoop::new();
    let window_builder = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(A::optimal_window_size())
        .with_title(A::title());
    let context = glium::glutin::ContextBuilder::new();
    let display = Display::new(window_builder, context, &event_loop).unwrap(); // TODO maybe not unwrap
    let mut binds = Vec::new();
    let mut graphics = Graphics::new(&display, A::optimal_window_size());
    let mut controls = Controls::new();
    let application = A::new(graphics.object_creator(&display), &mut binds);
    binds.into_iter().for_each(|bind| controls.add_bind(bind));
    let render_rate = 60;
    let mut engine = Engine {
        application,
        display,
        controls,
        game_data: None,
        graphics_data: GraphicsData {
            graphics,
            render_ref_time: Instant::now(),
            num_renders: 0,
            render_rate,
            render_phase: Duration::from_secs(0),
            last_render: Instant::now() - Duration::from_secs(1) / render_rate,
            fps: 0.0
        },
        cursor_data: CursorData {
            pos: LogicalPosition::new(0.0, 0.0),
            mode: CursorMode::Normal,
        },
        closing: false,
    };

    // main loop
    let mut event_buffer = VecDeque::new();
    event_loop.run(move |event, _, control_flow| {
        engine.handle_event(event);

        // this must not be in the device event branch of handle_event,
        // because events may also be produced by binding/unbinding
        engine.controls.get_events(&mut event_buffer);
        for event in event_buffer.drain(..) {
            engine.emit_event(Event::ControlEvent(event));
        }

        // update the game
        let next_tick_time = engine.maybe_update_game();

        // render the screen
        let next_render_time = engine.maybe_render();

        // close application
        if engine.closing {
            *control_flow = ControlFlow::Exit;
            return;
        };

        // schedule next loop
        let next_loop_time = next_tick_time.map_or(next_render_time, |x| x.min(next_render_time));
        if Instant::now() < next_loop_time {
            *control_flow = ControlFlow::WaitUntil(next_loop_time);
        } else {
            // ControlFlow::Poll seems to skip fetching window events
            //*control_flow = ControlFlow::Poll;
            *control_flow = ControlFlow::WaitUntil(next_loop_time);
            println!("polling");
        }
    });
}


#[cfg(test)]
mod tests {
    //use std::time::Instant;
    //use std::time::Duration;
    use std::cell::Cell;
    use std::io::Cursor;
    use std::rc::Rc;

    use strum_macros::EnumString;
    use strum_macros::ToString;

    use crate::Application;
    use crate::CursorController;
    use crate::ApplicationController;
    use crate::VirtualKeyCode;
    use crate::GameController;
    use crate::GraphicsInfo;
    use crate::run_application;
    use crate::GameStatus;
    use crate::GameInfo;
    use crate::FireTrigger;
    use crate::HoldableTrigger;
    use crate::ControlBind;
    use crate::ControlEvent;
    use crate::Game;
    use crate::Event;
    use crate::LogicalSize;
    use crate::Color;
    use crate::SceneObjectCreator;
    use crate::SceneRenderer;
    use crate::TexturelessSceneObject2d;
    use crate::TexturedSceneObject2d;
    use crate::TexturelessSceneObject3d;
    use crate::TexturedSceneObject3d;
    use crate::cgmath::Vector2;
    use crate::cgmath::Vector3;
    use crate::cgmath::Matrix2;
    use crate::cgmath::Matrix3;
    use crate::cgmath::Matrix4;
    use crate::cgmath::Rad;
    use crate::ValueTargetTrait;
    use crate::Texture2d;
    use crate::TEXT_NUM_LINES;
    use crate::OverlayAlignment;

    const NUM_TICKS: u64 = 131;
    //const TICK_RATE: u32 = 50;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, ToString, EnumString)]
    enum FireTarget {
        StartGame,
        EndGame,
        ToggleGamePause,
        FreeCursor,
        CaptureCursor,
        HideCursor,
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
            //if self.num_updates >= NUM_TICKS {
            //    GameStatus::Ended
            //} else {
                GameStatus::Running
            //}
        }
    }

    struct TestApplication {
        textured_cube: TexturedSceneObject3d<Rc<Texture2d>>,
        textureless_cube: TexturelessSceneObject3d,
        textured_square: TexturedSceneObject2d<Rc<Texture2d>>,
        textureless_square: TexturelessSceneObject2d,
        rectangle: TexturelessSceneObject2d,
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
            LogicalSize::new(160.0, 90.0)
        }

        fn new(
            mut scene_object_creator: SceneObjectCreator,
            binds: &mut Vec<ControlBind<FireTarget, SwitchTarget, ValueTarget>>,
        ) -> Self {

            binds.push(ControlBind::Fire(FireTrigger::Holdable(HoldableTrigger::KeyCode(VirtualKeyCode::Return)), FireTarget::StartGame));
            binds.push(ControlBind::Fire(FireTrigger::Holdable(HoldableTrigger::KeyCode(VirtualKeyCode::Back)), FireTarget::EndGame));
            binds.push(ControlBind::Fire(FireTrigger::Holdable(HoldableTrigger::KeyCode(VirtualKeyCode::P)), FireTarget::ToggleGamePause));
            binds.push(ControlBind::Fire(FireTrigger::Holdable(HoldableTrigger::KeyCode(VirtualKeyCode::F)), FireTarget::FreeCursor));
            binds.push(ControlBind::Fire(FireTrigger::Holdable(HoldableTrigger::KeyCode(VirtualKeyCode::C)), FireTarget::CaptureCursor));
            binds.push(ControlBind::Fire(FireTrigger::Holdable(HoldableTrigger::KeyCode(VirtualKeyCode::H)), FireTarget::HideCursor));

            let textured_cube_vertices = [
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

            let textureless_cube_vertices: Vec<Vector3<f32>>
                = textured_cube_vertices.iter().map(|p| p.0).collect();

            let cube_indices = [
                 0,  1,  3,   0,  3,  2,
                 4,  5,  7,   4,  7,  6,
                 8,  9, 11,   8, 11, 10,
                12, 13, 15,  12, 15, 14,
                16, 17, 19,  16, 19, 18,
                20, 21, 23,  20, 23, 22,
            ];

            let texture = Rc::new(scene_object_creator.create_texture(image::load(
                Cursor::new(&include_bytes!("../images/test_image.png")[..]),
                image::PNG,
            ).unwrap()));

            let textured_cube = scene_object_creator.create_textured3d(
                &textured_cube_vertices,
                &cube_indices,
                texture.clone(),
            );
            let textureless_cube = scene_object_creator.create_textureless3d(
                textureless_cube_vertices.as_ref(),
                &cube_indices
            );

            let textured_square_vertices = [
                (Vector2::new(-0.5, -0.5), Vector2::new(0.0, 0.0)),
                (Vector2::new( 0.5, -0.5), Vector2::new(1.0, 0.0)),
                (Vector2::new(-0.5,  0.5), Vector2::new(0.0, 1.0)),
                (Vector2::new( 0.5,  0.5), Vector2::new(1.0, 1.0)),
            ];

            let textureless_square_vertices: Vec<Vector2<f32>>
                = textured_square_vertices.iter().map(|p| p.0).collect();

            let square_indices = [
                0, 1, 3,
                0, 3, 2,
            ];

            let textured_square = scene_object_creator.create_textured2d(
                &textured_square_vertices,
                &square_indices,
                texture,
            );
            let textureless_square = scene_object_creator.create_textureless2d(
                textureless_square_vertices.as_ref(),
                &square_indices
            );

            let rectangle_vertices = [
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 1.0),
            ];
            let rectangle_indices = [
                0, 1, 3,
                0, 3, 2,
            ];

            let rectangle = scene_object_creator.create_textureless2d(
                &rectangle_vertices,
                &rectangle_indices,
            );

            TestApplication {
                textured_cube,
                textureless_cube,
                textured_square,
                textureless_square,
                rectangle,
                num_renders: Cell::new(0),
            }
        }

        fn handle_event(
            &mut self,
            event: Event<FireTarget, SwitchTarget, ValueTarget>,
            controller: ApplicationController<Self>,
        ) {
            match event {
                Event::ControlEvent(ce) => match ce {
                    ControlEvent::Fire(FireTarget::StartGame) => {
                        if let GameController::Closed(c) = controller.game_controller {
                            c.start_new(TestGame {
                                cube_rotation: 0.0,
                                num_updates: 0,
                            });
                        }
                    },
                    ControlEvent::Fire(FireTarget::EndGame) => {
                        if let GameController::Running(c) = controller.game_controller {
                            c.close();
                        } else if let GameController::Paused(c) = controller.game_controller {
                            c.close();
                        }
                    },
                    ControlEvent::Fire(FireTarget::ToggleGamePause) => {
                        if let GameController::Running(c) = controller.game_controller {
                            c.pause();
                        } else if let GameController::Paused(c) = controller.game_controller {
                            c.resume();
                        }
                    },
                    ControlEvent::Fire(FireTarget::FreeCursor) => {
                        match controller.cursor_controller {
                            CursorController::Captured(cc) => { cc.free(); },
                            CursorController::Hidden(cc) => { cc.show(); },
                            _ => (),
                        }
                    },
                    ControlEvent::Fire(FireTarget::CaptureCursor) => {
                        match controller.cursor_controller {
                            CursorController::Free(cc) => { cc.capture(); },
                            CursorController::Hidden(cc) => { cc.show_captured(); },
                            _ => (),
                        }
                    },
                    ControlEvent::Fire(FireTarget::HideCursor) => {
                        match controller.cursor_controller {
                            CursorController::Free(cc) => { cc.hide(); },
                            CursorController::Captured(cc) => { cc.hide(); },
                            _ => (),
                        }
                    },
                    ControlEvent::Switch { .. } => (),
                    ControlEvent::Value { .. } => (),
                },
                Event::GameUpdated => (),
                Event::CursorMoved => {
                    let position = match controller.cursor_controller {
                        CursorController::Free(cc) => Some(cc.position()),
                        CursorController::Captured(cc) => Some(cc.position()),
                        CursorController::Hidden(_) => None,
                    };
                    if let Some(p) = position {
                        eprintln!("cursor moved: {:.1}, {:.1}", p.0, p.1);
                    }
                },
                Event::WindowFocusChanged(focus) => {
                    eprintln!("focus: {}", focus);
                },
                Event::CloseRequested => controller.close(),
            }
        }

        fn render(
            &self,
            game_info: Option<GameInfo<TestGame>>,
            graphics_info: GraphicsInfo,
            renderer: SceneRenderer
        ) {
            let mut object_renderer = renderer.start_object_rendering(&Default::default());
            let mut overlay_renderer;
            if let Some(GameInfo { game, paused: _, ended: false }) = game_info {
                object_renderer.draw_textured(
                    &self.textured_cube,
                    &Matrix4::from_angle_z(Rad(game.cube_rotation)),
                );
                let x_cube = Matrix4::from_translation(Vector3::unit_x()) * Matrix4::from_scale(0.05);
                let z_cube = Matrix4::from_translation(Vector3::unit_y()) * Matrix4::from_scale(0.2);
                let y_cube = Matrix4::from_translation(Vector3::unit_z()) * Matrix4::from_scale(0.5);
                object_renderer.draw_textureless(&self.textureless_cube, Color::red(), &x_cube);
                object_renderer.draw_textureless(&self.textureless_cube, Color::green(), &y_cube);
                object_renderer.draw_textureless(&self.textureless_cube, Color::blue(), &z_cube);

                overlay_renderer = object_renderer.start_overlay_rendering();
                let rotation: Matrix3<f32> = Matrix2::from_angle(Rad(game.cube_rotation)).into();
                let square1 = Matrix3::new(
                    4.0, 0.0, 0.0,
                    0.0, 4.0, 0.0,
                    5.0, 5.0, 1.0f32,
                ) * rotation;
                let square2 = Matrix3::new(
                      4.0, 0.0, 0.0,
                      0.0, 4.0, 0.0,
                    155.0, 5.0, 1.0f32,
                ) * rotation;
                let square3 = Matrix3::new(
                    4.0,  0.0, 0.0,
                    0.0,  4.0, 0.0,
                    5.0, 85.0, 1.0f32,
                ) * rotation;
                let square4 = Matrix3::new(
                      4.0,  0.0, 0.0,
                      0.0,  4.0, 0.0,
                    155.0, 85.0, 1.0f32,
                ) * rotation;
                overlay_renderer.draw_textureless(
                    &self.textureless_square,
                    Color::black(),
                    &square1,
                    OverlayAlignment::BottomLeft,
                );
                overlay_renderer.draw_textured(
                    &self.textured_square,
                    &square2,
                    OverlayAlignment::BottomRight
                );
                overlay_renderer.draw_textured(
                    &self.textured_square,
                    &square3,
                    OverlayAlignment::TopLeft
                );
                overlay_renderer.draw_textureless(
                    &self.textureless_square,
                    Color::black(),
                    &square4,
                    OverlayAlignment::TopRight,
                );

                let alignments = [
                    OverlayAlignment::BottomLeft,
                    OverlayAlignment::Bottom,
                    OverlayAlignment::BottomRight,
                    OverlayAlignment::Left,
                    OverlayAlignment::Center,
                    OverlayAlignment::Right,
                    OverlayAlignment::TopLeft,
                    OverlayAlignment::Top,
                    OverlayAlignment::TopRight,
                ];
                let widths = [40.0, 80.0, 40.0];
                let heights = [25.0, 40.0, 25.0];
                for i in 0..9 {
                    let ix = i % 3;
                    let iy = i / 3;
                    let width = widths[ix];
                    let height = heights[iy];
                    let x = widths[..ix].iter().sum();
                    let y = heights[..iy].iter().sum();
                    let mat = Matrix3::new(
                        width,  0.0,    0.0,
                        0.0,    height, 0.0,
                        x,      y,      1.0,
                    );
                    overlay_renderer.draw_textureless(
                        &self.rectangle,
                        Color::blue(),
                        &mat,
                        alignments[i],
                    );
                }
            } else {
                overlay_renderer = object_renderer.start_overlay_rendering();
            }
            overlay_renderer.draw_text(0, &format!("FPS: {:.0}", graphics_info.fps));
            for i in 1..TEXT_NUM_LINES {
                overlay_renderer.draw_text(i, &format!("line {}", i));
            }
            self.num_renders.set(self.num_renders.get() + 1);
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
