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

struct Paddle {
    pos: nalgebra::Point2<f32>,
}

impl Paddle {
    pub fn new(pos: nalgebra::Point2<f32>) -> Self {
        Self { pos }
    }

    pub fn draw(&mut self, ctx: &mut Context, param: &mut graphics::DrawParam) -> GameResult {
        let rect = graphics::Rect::new(-PADDLE_HALF_WIDTH, -PADDLE_HALF_HEIGHT, PADDLE_WIDTH, PADDLE_HEIGHT);
        let mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, graphics::WHITE)?;

        param.dest = self.pos.into();
        graphics::draw(ctx, &mesh, *param)?;

        Ok(())
    }
}

struct Ball {
    pos: nalgebra::Point2<f32>,
    velocity: nalgebra::Vector2<f32>,
}

impl Ball {
    pub fn new(pos: nalgebra::Point2<f32>, velocity: nalgebra::Vector2<f32>) -> Self {
        Ball { pos, velocity }
    }

    pub fn normalize_velocity(&mut self) {
        self.velocity = self.velocity.normalize();
    }

    pub fn draw(&mut self, ctx: &mut Context, param: &mut graphics::DrawParam) -> GameResult {
        let rect = graphics::Rect::new(-BALL_HALF_SIZE, -BALL_HALF_SIZE, BALL_SIZE, BALL_SIZE);
        let mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, graphics::WHITE)?;

        param.dest = self.pos.into();
        graphics::draw(ctx, &mesh, *param)?;

        // KLUDGE: Ball pos
        let ball_pos = graphics::Text::new(format!("{}, {}", self.pos.x, self.pos.y));

        param.dest = nalgebra::Point2::new(0.0, 0.0).into();
        graphics::draw(ctx, &ball_pos, *param)?;

        Ok(())
    }
}

struct Pong {
    p1: Paddle,
    p2: Paddle,
    ball: Ball,
    p1_score: u32,
    p2_score: u32,
    screen_borders: (f32, f32),
    rng: ThreadRng,
}

impl Pong {
    pub fn new(ctx: &mut Context) -> Self {
        let (screen_w, screen_h) = graphics::drawable_size(ctx);

        Pong {
            p1: Paddle::new(nalgebra::Point2::new(PADDLE_HALF_WIDTH, screen_h * 0.5)),
            p2: Paddle::new(nalgebra::Point2::new(screen_w - PADDLE_HALF_WIDTH, screen_h * 0.5)),
            ball: Ball::new(nalgebra::Point2::new(screen_w * 0.5, screen_h * 0.5), nalgebra::Vector2::new(1.0, 1.0)),
            p1_score: 0,
            p2_score: 0,
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

        self.ball.velocity.x = theta.cos() * self.ball.velocity.x - theta.sin() * self.ball.velocity.y;
        self.ball.velocity.y = theta.sin() * self.ball.velocity.x + theta.cos() * self.ball.velocity.y;

        self.ball.normalize_velocity();

        self.ball.pos.x = self.rng.gen_range((self.screen_borders.0 * 0.3)..(self.screen_borders.0 * 0.6));
        self.ball.pos.y = self.rng.gen_range((self.screen_borders.1 * 0.3)..(self.screen_borders.1 * 0.6));
    }

    fn handle_input(&mut self, ctx: &mut Context, delta: f32) -> GameResult {
        // TEMP: Reset the game
        if keyboard::is_key_pressed(ctx, KeyCode::R) {
            self.reset_ball();
        }

        if keyboard::is_key_pressed(ctx, KeyCode::W) {
           self.p1.pos.y -= delta * PLAYER_SPEED;
        }

        if keyboard::is_key_pressed(ctx, KeyCode::S) {
           self.p1.pos.y += delta * PLAYER_SPEED;
        }

        Pong::clamp(&mut self.p1.pos.y, self.screen_borders.1 - PADDLE_HALF_HEIGHT, PADDLE_HALF_HEIGHT);
        Pong::clamp(&mut self.p2.pos.y, self.screen_borders.1 - PADDLE_HALF_HEIGHT, PADDLE_HALF_HEIGHT);

        Ok(())
    }

    fn check_winner(&mut self) -> GameResult {
        if self.ball.pos.x <= 0.0 {
            self.reset_ball();
            self.p2_score += 1;
        }

        if self.ball.pos.x >= self.screen_borders.0 {
            self.reset_ball();
            self.p1_score += 1;
        }

        Ok(())
    }

    fn move_ball(&mut self, delta: f32) -> GameResult {
        self.ball.pos += self.ball.velocity * BALL_SPEED * delta;

        if self.ball.pos.y >= self.screen_borders.1 - BALL_HALF_SIZE || self.ball.pos.y <= 0.0 + BALL_HALF_SIZE {
            self.ball.velocity.y *= -1.0;
        }

        if self.ball.pos.x < self.p1.pos.x + PADDLE_HALF_WIDTH && self.ball.pos.x > self.p1.pos.x - PADDLE_HALF_WIDTH
            || self.ball.pos.x > self.p2.pos.x + PADDLE_HALF_WIDTH && self.ball.pos.x < self.p2.pos.x - PADDLE_HALF_WIDTH
        {
            self.ball.velocity.x *= -1.0;
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

        let mut draw_param = graphics::DrawParam::default();

        self.p1.draw(ctx, &mut draw_param)?;
        self.p2.draw(ctx, &mut draw_param)?;

        self.ball.draw(ctx, &mut draw_param)?;

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
