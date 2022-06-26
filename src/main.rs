#![warn(clippy::pedantic)]

use bracket_lib::prelude::*;

const SCREEN_WIDTH : i32 = 40;
const SCREEN_HEIGHT : i32 = 25;
const FRAME_DURATION : f32 = 60.0;
const GRAVITY : f32 = 0.4;
const MAX_GRAVITY : f32 = 2.0;
const JUMP_FORCE : f32 = 2.5;
const HORIZONTAL_VELOCITY : i32 = 1;
const PLAYER_FRAMES : [usize; 6] = [ 64, 1, 2, 3, 2, 1 ];


enum GameMode {
    Menu,
    Playing,
    End,
}

struct Player {
    x: i32,
    y: f32,
    velocity: f32,
    spin: Radians,
    scale: PointF,
    frame: usize
}

impl Player {
    fn new(x: i32, y: f32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
            spin: Radians::new(0.0),
            scale: PointF { x: 2.0, y: 2.0},
            frame: 0
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_fancy(
            PointF { x: 0.0, y: self.y},
            1,
            self.spin,
            self.scale,
            YELLOW,
            NAVY,
            PLAYER_FRAMES[self.frame]
        );
        ctx.set_active_console(0);
    }

    fn gravity_and_move(&mut self) {
        if self.velocity <= MAX_GRAVITY {
            self.velocity += GRAVITY;
        }
        self.y += self.velocity;
        self.x += HORIZONTAL_VELOCITY;
        self.frame += 1;
        self.frame = self.frame % PLAYER_FRAMES.len();
        if self.y < 0.0 {
            self.y = 0.0;
        };
    }

    fn flap(&mut self) {
        self.velocity = -JUMP_FORCE;
    }
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, SCREEN_HEIGHT - 10),
            size: i32::max(2, 20 - score)
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player: &Player) {
        let screen_x = self.x - player.x;
        let half_size = self.size / 2;

        for y in 0..(self.gap_y - half_size) {
            ctx.set(
                screen_x,
                y,
                GREY,
                BLACK,
                179
            );
        };

        for y in (self.gap_y + half_size)..SCREEN_HEIGHT {
            ctx.set(
                screen_x,
                y,
                GREY,
                BLACK,
                179
            );
        }
    }

    fn hit_obstacle(&mut self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = self.x == player.x;
        let player_hit_top = (player.y as i32) < (self.gap_y - half_size);
        let player_hit_bottom = (player.y as i32) > (self.gap_y + half_size);

        does_x_match && (player_hit_top || player_hit_bottom)
    }
}

struct State {
    mode: GameMode,
    frame_time: f32,
    player: Player,
    obstacle: Obstacle,
    score: i32
}

impl State {
    fn new() -> Self {
        State {
            mode: GameMode::Menu,
            player: Player::new(5, (SCREEN_HEIGHT / 2) as f32),
            frame_time: 0.0,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            score: 0
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
        }
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }
        self.player.render(ctx);
        self.obstacle.render(ctx, &self.player);
        ctx.print(0, 0, "Press SPACE to flap");
        ctx.print(0, 1, &format!("Score: {}", self.score));

        for floor_x in 0..SCREEN_WIDTH {
            ctx.set(floor_x, SCREEN_HEIGHT - 1, GREY, NAVY, 35);
        };

        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(SCREEN_WIDTH + self.player.x, self.score);
        }
        if self.player.y as i32 > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_color_centered(5, CYAN, BLACK, "Welcome to Flappy Rust");
        ctx.print_color_centered(8, GREEN, BLACK, "(P) Play Game");
        ctx.print_color_centered(9, GREEN, BLACK, "(Q) Quit Game");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_color_centered(5, RED, BLACK, "You are dead!");
        ctx.print_color_centered(6, GREEN, BLACK, &format!("Final score: {}", self.score));
        ctx.print_color_centered(8, GREEN, BLACK, "(P) Play Again");
        ctx.print_color_centered(9, GREEN, BLACK, "(Q) Quit Game ");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(5, (SCREEN_HEIGHT / 2) as f32);
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.dead(ctx),
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::new()
        .with_font("../resources/flappy32.png", 32, 32)
        .with_title("Flappy Rust")
        .with_tile_dimensions(16, 16)
        .with_vsync(false)
        .with_simple_console(SCREEN_WIDTH, SCREEN_HEIGHT, "../resources/flappy32.png")
        .with_fancy_console(SCREEN_WIDTH, SCREEN_HEIGHT, "../resources/flappy32.png")
        .build()?;
    main_loop(context, State::new())
}
