use macroquad::prelude::*;

const PLAYER_SIZE: Vec2 = const_vec2!([150f32, 40f32]);
const PLAYER_SPEED: f32 = 700f32;
const BLOCK_SIZE: Vec2 = const_vec2!([100f32, 40f32]);
const BLOCK_PADDING: f32 = 15f32;
const BLOCK_BOX: (i32, i32) = (6, 6);
const BALL_SIZE: f32 = 35f32;
const BALL_SPEED: f32 = 400f32;
pub enum GameState {
    Menu,
    Game,
    LevelCompleted,
    Dead,
}

struct Player {
    rect: Rect,
}

impl Player {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(
                screen_width() * 0.5f32 - PLAYER_SIZE.x * 0.5f32,
                screen_height() - 100f32,
                PLAYER_SIZE.x,
                PLAYER_SIZE.y,
            ),
        }
    }

    pub fn update(&mut self, dt: f32) {
        let x_move = match (is_key_down(KeyCode::Left), is_key_down(KeyCode::Right)) {
            (true, false) => -1f32,
            (false, true) => 1f32,
            _ => 0f32,
        };
        self.rect.x += x_move * dt * PLAYER_SPEED;

        if self.rect.x < 0f32 {
            self.rect.x = 0f32;
        }
        if self.rect.x > screen_width() - self.rect.w {
            self.rect.x = screen_width() - self.rect.w;
        }
    }

    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, BLUE);
    }
}

struct Block {
    rect: Rect,
    lives: i32,
}

impl Block {
    pub fn new(pos: Vec2) -> Self {
        Self {
            rect: Rect::new(
                pos.x,
                pos.y,
                BLOCK_SIZE.x - BLOCK_PADDING,
                BLOCK_SIZE.y - BLOCK_PADDING,
            ),
            lives: 2,
        }
    }

    pub fn draw(&self) {
        let color: Color = match self.lives {
            1 => ORANGE,
            _ => RED,
        };

        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, color);
    }

    pub fn init() -> Vec<Self> {
        let mut blocks: Vec<Block> = Vec::new();

        let border_start_pos: Vec2 = vec2(
            (screen_width() - (BLOCK_SIZE.x * BLOCK_BOX.0 as f32)) * 0.5f32,
            50f32,
        );

        for i in 0..BLOCK_BOX.0 * BLOCK_BOX.1 {
            let block_x = (i % BLOCK_BOX.0) as f32 * BLOCK_SIZE.x;
            let block_y = (i / BLOCK_BOX.0) as f32 * BLOCK_SIZE.y;
            blocks.push(Block::new(border_start_pos + vec2(block_x, block_y)));
        }
        blocks
    }
}

struct Ball {
    shape: Rect,
    vel: Vec2,
}

impl Ball {
    pub fn new(pos: Vec2) -> Ball {
        Ball {
            shape: Rect::new(pos.x, pos.y, BALL_SIZE, BALL_SIZE),
            vel: vec2(rand::gen_range(-1f32, 1f32), 1f32).normalize(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.shape.x += self.vel.x * dt * BALL_SPEED;
        self.shape.y += self.vel.y * dt * BALL_SPEED;
        if self.shape.x < 0f32 {
            self.vel.x = 1f32;
        }
        if self.shape.x > screen_width() - self.shape.w {
            self.vel.x = -1f32;
        }
        if self.shape.y < 0f32 {
            self.vel.y = 1f32;
        }
    }

    pub fn draw(&self) {
        draw_rectangle(self.shape.x, self.shape.y, self.shape.w, self.shape.h, RED);
    }

    pub fn reset(&mut self, pos: Option<Vec2>, vel: Option<Vec2>) {
        let position = match pos {
            Some(x) => x,
            None => vec2(screen_width() * 0.5f32, screen_height() * 0.5f32),
        };
        self.shape.x = position.x;
        self.shape.y = position.y;

        let velocity: Vec2 = match vel {
            Some(x) => x,
            None => vec2(rand::gen_range(-1f32, 1f32), 1f32),
        };

        self.vel = velocity.normalize();
    }
}

fn resolve_collision(a: &mut Rect, vel: &mut Vec2, b: &Rect) -> bool {
    let intersection = match a.intersect(*b) {
        Some(intersection) => intersection,
        None => return false,
    };
    let a_center = a.point() + a.size() * 0.5f32;
    let b_center = b.point() + b.size() * 0.5f32;
    let to = b_center - a_center;
    let to_signum = to.signum();
    match intersection.w > intersection.h {
        true => {
            // bounce on y
            a.y -= to_signum.y * intersection.h;
            vel.y = -to_signum.y * vel.y.abs();
        }
        false => {
            // bounce on x
            a.x -= to_signum.x * intersection.w;
            vel.x = -to_signum.x * vel.x.abs();
        }
    }
    true
}

fn lose_statement(ball: &mut Ball, lives: &mut i32, player_shape: Rect, state: &mut GameState) {
    if ball.shape.y < screen_height() {
        return;
    }
    *lives -= 1;
    ball.reset(
        Some(vec2(
            player_shape.x + (player_shape.w * 0.5),
            player_shape.y - (player_shape.h * 0.5),
        )),
        Some(vec2(rand::gen_range(-1f32, 1f32), -1f32)),
    );
    if *lives <= 0 {
        *state = GameState::Dead;
    }
}

fn reset(
    lives: &mut i32,
    score: &mut i32,
    ball: &mut Ball,
    player: &mut Player,
    blocks: &mut Vec<Block>,
) {
    *player = Player::new();
    *score = 0;
    *lives = 3;
    ball.reset(None, None);
    *blocks = Block::init();
}

fn draw_title_text(text: &str) {
    let dims = measure_text(text, None, 50u16, 1.0f32);
    draw_text(
        text,
        screen_width() * 0.5f32 - dims.width * 0.5f32,
        screen_height() * 0.5f32 - dims.height * 0.5f32,
        50f32,
        BLACK,
    );
}

#[macroquad::main("breakout")]
async fn main() {
    let mut score = 0;
    let mut lives = 3;
    let mut game_state = GameState::Menu;

    let mut player: Player = Player::new();
    let mut blocks: Vec<Block> = Block::init();
    let mut ball = Ball::new(vec2(screen_width() * 0.5f32, screen_height() * 0.5f32));

    loop {
        clear_background(WHITE);
        player.draw();
        for block in blocks.iter() {
            block.draw();
        }

        match game_state {
            GameState::Menu => {
                draw_title_text("Press SPACE to start !");
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Game;
                }
            }
            GameState::Game => {
                ball.draw();
                ball.update(get_frame_time());
                player.update(get_frame_time());

                lose_statement(&mut ball, &mut lives, player.rect.clone(), &mut game_state);

                resolve_collision(&mut ball.shape, &mut ball.vel, &player.rect);
                for block in blocks.iter_mut() {
                    if resolve_collision(&mut ball.shape, &mut ball.vel, &block.rect) {
                        block.lives -= 1;
                        score += 10;
                    }
                }
                blocks.retain(|block| block.lives > 0);
                if blocks.is_empty() {
                    game_state = GameState::LevelCompleted;
                }
            }
            GameState::LevelCompleted => {
                draw_title_text("You Win! Press SPACE to restart");
                if is_key_pressed(KeyCode::Space) {
                    reset(&mut lives, &mut score, &mut ball, &mut player, &mut blocks);
                    game_state = GameState::Game;
                }
            }
            GameState::Dead => {
                draw_title_text("You Lose! Press SPACE to restart");
                if is_key_pressed(KeyCode::Space) {
                    reset(&mut lives, &mut score, &mut ball, &mut player, &mut blocks);
                    game_state = GameState::Game;
                }
            }
        }
        draw_text(&format!("score {}", score), 100f32, 40.0, 30f32, BLACK);

        let lives_text = format!("lives: {}", lives);
        let lives_text_dim = measure_text(&lives_text, None, 30u16, 1.0);

        draw_text(
            &lives_text,
            screen_width() - (100f32 + lives_text_dim.width),
            40.0,
            30f32,
            BLACK,
        );
        next_frame().await
    }
}
