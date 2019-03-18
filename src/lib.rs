extern crate glium;

use std::time::Instant;
use std::time::Duration;

use glium::glutin::dpi::LogicalSize;
use glium::glutin::Event;
use glium::Frame;
use glium::Display;
use glium::glutin::EventsLoop;

pub trait Game {
    fn title() -> &'static str;
    fn initial_window_size() -> LogicalSize;
    fn new() -> Self;
    fn handle_event(&mut self, event: Event);
    fn tick(&mut self);
    fn render(&mut self, frame: &mut Frame);
    fn finished(&self) -> bool;
}

pub struct Application<G: Game> {
    game: G,
    display: Display,
    events_loop: EventsLoop,
}

impl<G: Game> Application<G> {
    pub fn new() -> Self {
        let events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_dimensions(G::initial_window_size())
            .with_title(G::title());
        let context = glium::glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();
        Self {
            game: G::new(),
            display,
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
            let now = Instant::now();
            if now >= next_tick_time {
                let game = &mut self.game;
                self.events_loop.poll_events(|event| game.handle_event(event));
                self.game.tick();
                num_ticks += 1;
                next_tick_time = start_time + Duration::from_secs(num_ticks) / tick_rate;
            }
            if now >= next_render_time {
                let mut frame = self.display.draw();
                self.game.render(&mut frame);
                frame.finish().unwrap(); // TODO maybe not unwrap?
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

    use crate::Game;
    use crate::Application;
    use crate::LogicalSize;
    use crate::Event;
    use crate::Frame;

    const NUM_TICKS: u64 = 81;
    const TICK_RATE: u32 = 50;

    struct TestGame {
        num_ticks: u64,
        num_renders: u64,
    }

    impl Game for TestGame {
        fn title() -> &'static str {
            "Test Game"
        }

        fn initial_window_size() -> LogicalSize {
            LogicalSize::new(50.0, 50.0)
        }

        fn new() -> Self {
            TestGame {
                num_ticks: 0,
                num_renders: 0,
            }
        }

        fn handle_event(&mut self, event: Event) {
            eprintln!("{:?}", event);
        }

        fn tick(&mut self) {
            self.num_ticks += 1;
        }

        fn render(&mut self, _frame: &mut Frame) {
            self.num_renders += 1;
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
            app.game.num_renders >= NUM_TICKS,
            "left: {}, right: {}",
            app.game.num_renders,
            NUM_TICKS
        );

        const NUM_MILLIS: u64 = (NUM_TICKS - 1) * 1000 / TICK_RATE as u64;
        const TARGET_DURATION: Duration = Duration::from_millis(NUM_MILLIS);
        if duration > TARGET_DURATION {
            assert!(
                duration - TARGET_DURATION < Duration::from_millis(4),
                "left: {:?}, right: {:?}",
                duration,
                TARGET_DURATION
            );
        } else {
            assert!(
                TARGET_DURATION - duration < Duration::from_millis(4),
                "left: {:?}, right: {:?}",
                duration,
                TARGET_DURATION
            );
        }
    }
}
