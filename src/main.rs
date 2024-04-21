use rand::{Rng, thread_rng};
use rusty_engine::prelude::*;
use rand::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq)]
enum HammerState {
    Hold,
    Hit,
    Pressed,
}

#[derive(Debug)]
struct Hammer {
    position: Vec2,
    state: HammerState,
}

impl Hammer {
    fn new() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            state: HammerState::Hold,
        }
    }

    fn update_position(&mut self, position: Vec2) {
        self.position = position
    }
    fn update_state(&mut self, pressed: bool) {
        self.state = match (self.state, pressed) {
            (HammerState::Hold, true) => HammerState::Hit,
            (HammerState::Hit, false) => HammerState::Hold,
            (HammerState::Hit, true) => HammerState::Pressed,
            (HammerState::Pressed, false) => HammerState::Hold,
            (_, _) => self.state,
        }
    }
    fn init(&self, game: &mut Game<GameState>) {
        let sprite = game.add_sprite("player", "sprite/hammer.png");
        self._draw(sprite);
    }
    fn draw(&self, engine: &mut Engine) {
        let sprite = engine.sprites.get_mut("player").unwrap();
        self._draw(sprite);
    }

    fn _draw(&self, sprite: &mut Sprite) {
        sprite.layer = 3.0;
        sprite.scale = 0.6;
        sprite.translation.x = self.position.x;
        sprite.translation.y = self.position.y;
        sprite.rotation = match self.state {
            HammerState::Hold => EAST,
            _ => NORTH_EAST,
        };
        sprite.collision = match self.state {
            HammerState::Hit => true,
            _ => false
        };
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum EnemyState {
    Hide,
    Showing(Vec2),
}

struct Enemy {
    name: String,
    state: EnemyState,
    timer: Timer,
}

impl Enemy {
    fn new(name: String, timer_seconds: f32) -> Self {
        Self {
            name: name.into(),
            state: EnemyState::Hide,
            timer: Timer::from_seconds(timer_seconds, false),
        }
    }


    fn draw(&self, engine: &mut Engine) {
        match self.state {
            EnemyState::Hide => {
                engine.sprites.remove(self.name.as_str());
            }
            EnemyState::Showing(pos) => {
                let sprite = match engine.sprites.get_mut(self.name.as_str()) {
                    None => { engine.add_sprite(self.name.as_str(), "sprite/ninja.png") }
                    Some(sprite) => { sprite }
                };
                sprite.scale = 0.5;
                sprite.layer = 2.0;
                sprite.translation = pos;
                sprite.collision = true;
            }
        }
    }

    fn reset(&mut self, holes: &mut Holes) {
        match self.state {
            EnemyState::Hide => {
                let x = holes.occupy();
                self.state = EnemyState::Showing(x);
            }
            EnemyState::Showing(pos) => {
                holes.release(pos);
                self.state = EnemyState::Hide;
            }
        }
        self.timer = Timer::from_seconds(thread_rng().gen_range(1.0..3.0), false);
    }

    fn hit(&mut self, holes: &mut Holes) -> bool {
        if let EnemyState::Showing(pos) = self.state {
            holes.release(pos);
            self.state = EnemyState::Hide;
            return true;
        }
        false
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum HoleState {
    Free,
    Occupied,
}

struct Hole {
    id: usize,
    pos: Vec2,
    state: HoleState,
}

impl Hole {
    fn new(id: usize, pos: Vec2) -> Self {
        Self {
            id,
            pos,
            state: HoleState::Free,
        }
    }
    fn init(&self, game: &mut Game<GameState>) {
        let hole = game.add_sprite(format!("hole_{}", self.id), "sprite/hospital.png");
        hole.layer = 1.0;
        hole.scale = 0.5;
        hole.collision = true;
        hole.translation = self.pos.clone();
    }
    fn draw(&self, engine: &mut Engine) {
        let hole = engine.sprites.get_mut(&format!("hole_{}", self.id)).unwrap();
        hole.collision = match self.state {
            HoleState::Free => { true }
            HoleState::Occupied => { false }
        }
    }
}

struct Holes(Vec<Hole>);

impl Holes {
    fn new(positions: Vec<Vec2>) -> Self {
        Self(positions.iter().enumerate().map(|(i, p)| Hole::new(i, p.clone())).collect())
    }

    fn occupy(&mut self) -> Vec2 {
        let hole = self.0.iter_mut()
            .filter(|o| { o.state == HoleState::Free })
            .choose(&mut thread_rng())
            .unwrap();

        hole.state = HoleState::Occupied;
        hole.pos
    }

    fn release(&mut self, pos: Vec2) {
        if let Some(hole) = self.0.iter_mut().find(|hole| hole.pos == pos) {
            hole.state = HoleState::Free;
        }
    }

    fn find(&self, name: &str) -> Option<&Hole> {
        self.0.iter().find(|h| name == format!("hole_{}", h.id))
    }

    fn init(&self, game: &mut Game<GameState>) {
        for hole in self.0.iter() {
            hole.init(game)
        }
    }

    fn draw(&self, engine: &mut Engine) {
        for hole in self.0.iter() {
            hole.draw(engine)
        }
    }

    fn hit(&self, name: &str) -> bool {
        if let Some(hole) = self.find(&name) {
            if hole.state == HoleState::Free {
                return true;
            }
        }
        false
    }
}

struct GameState {
    hammer: Hammer,
    holes: Holes,
    enemies: Vec<Enemy>,
    score: u32,
    lives: u32,
}

fn main() {
    let mut game = Game::new();

    // score
    let score_message = game.add_text("score", "Score: 0");
    score_message.translation = Vec2::new(550.0, 320.0);

    // score
    let score_message = game.add_text("lives", "Lives: 3");
    score_message.translation = Vec2::new(-550.0, 320.0);

    // player - hammer
    let hammer = Hammer::new();
    hammer.init(&mut game);

    // holes
    let holes_pos = vec![
        Vec2::new(-300.0, 200.),
        Vec2::new(0.0, 200.),
        Vec2::new(300.0, 200.),
        Vec2::new(-150.0, 0.0),
        Vec2::new(150.0, 0.0),
    ];

    let holes = Holes::new(holes_pos);
    holes.init(&mut game);

    // enemies
    let enemies = (0..4).map(|x| {
        Enemy::new(
            format!("enemy_{}", x),
            thread_rng().gen_range(1.0..3.0),
        )
    }).collect::<Vec<_>>();


    // game setup goes here
    game.add_logic(game_logic);
    game.run(GameState { hammer, holes, enemies, score: 0, lives: 3 });
}

fn game_logic(engine: &mut Engine, game_state: &mut GameState) {
    if game_state.lives == 0 {
        let game_over = engine.add_text("game_over", "GAME OVER: The UN condemn you");
        game_over.font_size = 90.0;
        game_over.translation.y = -200.0;
        return;
    }

    if let Some(location) = engine.mouse_state.location() {
        game_state.hammer.update_position(location);
    }

    game_state
        .hammer
        .update_state(engine.mouse_state.pressed(MouseButton::Left));

    for enemy in &mut game_state.enemies {
        if enemy.timer.tick(engine.delta).just_finished() {
            enemy.reset(&mut game_state.holes);
        }
    }

    game_state.enemies.iter().for_each(|e| e.draw(engine));
    game_state.holes.draw(engine);
    game_state.hammer.draw(engine);


    for event in engine.collision_events.drain(..) {
        if event.state.is_end() {
            continue;
        }
        if let Some(name) = player_hit(&event.pair) {
            if name.starts_with("enemy") {
                if let Some(enemy) = game_state.enemies.iter_mut().find(|enemy| enemy.name == name) {
                    if enemy.hit(&mut game_state.holes) {
                        engine.audio_manager.play_sfx(
                            SfxPreset::Impact1, 0.2,
                        );
                        game_state.score += 1;
                    }
                }
            } else if name.starts_with("hole") {
                if game_state.holes.hit(&name) {
                    engine.audio_manager.play_sfx(
                        SfxPreset::Forcefield1, 0.2,
                    );
                    game_state.lives -= 1;
                }
            }
        }
    }

    let score = engine.texts.get_mut("score").unwrap();
    score.value = format!("Score: {}", game_state.score);

    let lives = engine.texts.get_mut("lives").unwrap();
    lives.value = format!("Lives: {}", game_state.lives);
}

fn player_hit(pair: &CollisionPair) -> Option<String> {
    match pair.array() {
        ["player", other] => Some(other.to_string()),
        [other, "player"] => Some(other.to_string()),
        _ => None,
    }
}
