use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler, KeyCode};
use ggez::timer;
use ggez::input::keyboard;
use ggez::graphics;
use ggez::nalgebra;
use rand::prelude::*;

const PADDLE_WIDTH: f32 = 20.0;
const PADDLE_HEIGHT: f32 = 100.0;
const PADDLE_HALF_WIDTH: f32 = PADDLE_WIDTH * 0.5;
const PADDLE_HALF_HEIGHT: f32 = PADDLE_HEIGHT * 0.5;
const BALL_SIZE: f32 = 15.0;
const BALL_HALF_SIZE: f32 = BALL_SIZE * 0.5;

const PLAYER_SPEED: f32 = 500.0;
const BALL_SPEED: f32 = 250.0;

struct Pong {
    p1_pos: nalgebra::Point2<f32>,
    p1_score: u32,
    p2_pos: nalgebra::Point2<f32>,
    p2_score: u32,
    ball_pos: nalgebra::Point2<f32>,
    ball_dir: nalgebra::Vector2<f32>,
    screen_borders: (f32, f32),
    rng: ThreadRng,
}

impl Pong {
    pub fn new(ctx: &mut Context) -> Self {
        let (screen_w, screen_h) = graphics::drawable_size(ctx);

        Pong {
            p1_pos: nalgebra::Point2::new(PADDLE_HALF_WIDTH, screen_h * 0.5),
            p1_score: 0,
            p2_pos: nalgebra::Point2::new(screen_w - PADDLE_HALF_WIDTH, screen_h * 0.5),
            p2_score: 0,
            ball_pos: nalgebra::Point2::new(screen_w * 0.5, screen_h * 0.5),
            ball_dir: nalgebra::Vector2::new(1.0, 1.0),
            screen_borders: (screen_w, screen_h),
            rng: rand::thread_rng(),
        }
    }

    fn clamp(value: &mut f32, hi: f32, lo: f32) {
        if *value < lo {
            *value = lo;
        } else if *value > hi {
            *value = hi;
        }
    }

    fn reset_ball(&mut self) {
        let theta: f32 = self.rng.gen_range(0.0..(2.0 * std::f32::consts::PI));

        self.ball_dir.x = theta.cos() * self.ball_dir.x - theta.sin() * self.ball_dir.y;
        self.ball_dir.y = theta.sin() * self.ball_dir.x + theta.cos() * self.ball_dir.y;

        self.ball_dir = self.ball_dir.normalize();

        self.ball_pos.x = self.rng.gen_range((self.screen_borders.0 * 0.3)..(self.screen_borders.0 * 0.6));
        self.ball_pos.y = self.rng.gen_range((self.screen_borders.1 * 0.3)..(self.screen_borders.1 * 0.6));
    }

    fn handle_input(&mut self, ctx: &mut Context, delta: f32) -> GameResult {
        if keyboard::is_key_pressed(ctx, KeyCode::W) {
           self.p1_pos.y -= delta * PLAYER_SPEED;
        }

        if keyboard::is_key_pressed(ctx, KeyCode::S) {
           self.p1_pos.y += delta * PLAYER_SPEED;
        }

        Pong::clamp(&mut self.p1_pos.y, self.screen_borders.1 - PADDLE_HALF_HEIGHT, PADDLE_HALF_HEIGHT);
        Pong::clamp(&mut self.p2_pos.y, self.screen_borders.1 - PADDLE_HALF_HEIGHT, PADDLE_HALF_HEIGHT);

        Ok(())
    }

    fn check_winner(&mut self) -> GameResult {
        if self.ball_pos.x <= 0.0 {
            self.reset_ball();
            self.p2_score += 1;
        }

        if self.ball_pos.x >= self.screen_borders.0 {
            self.reset_ball();
            self.p1_score += 1;
        }

        Ok(())
    }

    fn move_ball(&mut self, delta: f32) -> GameResult {
        self.ball_pos.x += delta * BALL_SPEED * self.ball_dir.x;
        self.ball_pos.y += delta * BALL_SPEED * self.ball_dir.y;

        if self.ball_pos.y >= self.screen_borders.1 || self.ball_pos.y <= 0.0 {
            self.ball_dir.y *= -1.0;
        }

        self.check_winner()?;

        Ok(())
    }
}

impl EventHandler for Pong {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let delta = timer::delta(ctx).as_secs_f32();

        self.handle_input(ctx, delta)?;
        self.move_ball(delta)?;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);

        let paddle = graphics::Rect::new(-PADDLE_HALF_WIDTH, -PADDLE_HALF_HEIGHT, PADDLE_WIDTH, PADDLE_HEIGHT);
        let paddle_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), paddle, graphics::WHITE)?;

        let mut draw_param = graphics::DrawParam::default();

        // P1 Paddle
        draw_param.dest = self.p1_pos.into();
        graphics::draw(ctx, &paddle_mesh, draw_param)?;

        // P2 Paddle
        draw_param.dest = self.p2_pos.into();
        graphics::draw(ctx, &paddle_mesh, draw_param)?;

        // Ball
        let ball = graphics::Rect::new(-BALL_HALF_SIZE, -BALL_HALF_SIZE, BALL_SIZE, BALL_SIZE);
        let ball_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), ball, graphics::WHITE)?;

        draw_param.dest = self.ball_pos.into();
        graphics::draw(ctx, &ball_mesh, draw_param)?;

        // Score
        let score = graphics::Text::new(format!("P1: {}  --  P2: {}", self.p1_score, self.p2_score));
        let (score_x, _) = score.dimensions(ctx);

        draw_param.dest = nalgebra::Point2::new(self.screen_borders.0 * 0.5 - score_x as f32 * 0.5, 5.0).into();
        graphics::draw(ctx, &score, draw_param)?;

        graphics::present(ctx)?;
        Ok(())
    }
}


fn main() -> GameResult {
    let (mut ctx, mut event_loop) = ContextBuilder::new("Pong", "Alex Sun").build()?;

    graphics::set_window_title(&ctx, "Pong");

    let mut pong = Pong::new(&mut ctx);
    event::run(&mut ctx, &mut event_loop, &mut pong)?;

    Ok(())
}
