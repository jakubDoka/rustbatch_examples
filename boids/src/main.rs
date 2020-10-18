extern crate rustbatch;

use rustbatch::{sdl2, image, gl, rand, Mat};
use rustbatch::debug::FPS;
use rustbatch::{Window, Texture, Sprite, Batch};
use rustbatch::math::vect::Vect;
use rustbatch::rand::Rng;
use rustbatch::math::rect::{Rect, Sides};
use std::mem::replace;
use rustbatch::math::rgba::{BLACK, WHITE};
use rustbatch::vect;
use rustbatch::render::texture::TextureConfig;
use rustbatch::entities::detection::quadmap::{Address, QuadMap};

struct Bee {
    pos: Vect,
    vel: Vect,
    add: Address,
}

struct Engine {
    e: Vec<Option<Bee>>,
    col: Vec<Vect>,
    col1: Vec<Vect>,
    bounds: Rect,
    map: QuadMap<usize>,
    col2: Vec<usize>,
}

const REPEL_COF: f32 = 7.5f32;
const ALIGN_COF: f32 = 0.045f32;
const COHESION_COF: f32 = 0.03f32;
const SPEED_LIMITS: (f32, f32) = (25.0f32, 150.0f32);
const SIGHT: f32 = 20f32;

impl Engine {
    pub fn update(&mut self, b: &mut Batch, s: &mut Sprite, delta: f32) {
        let mut useless = 0;
        for i in 0..self.e.len() {
            let mut bee = replace(&mut self.e[i], None).unwrap();

            let mut avoidance = Vect::ZERO;

            self.map.query(Rect::cube(bee.pos, SIGHT), &mut self.col2);
            for b in self.col2.iter() {
                let b = match &self.e[*b as usize] {
                    Some(val) => val,
                    None => {
                        continue
                    }
                };

                let dif = b.pos - bee.pos;
                let len = dif.len();
                if len > SIGHT {
                    useless += 1;
                    continue;
                }

                let inv = 1f32 / len.powi(2);
                avoidance -= dif * inv;

                self.col.push(b.pos);
                self.col1.push(b.vel);
            }

            let alignment = Vect::average(&self.col1);
            let cohesion = Vect::average(&self.col) - bee.pos;

            self.col.clear();
            self.col1.clear();
            self.col2.clear();

            bee.vel += avoidance * REPEL_COF  + alignment * ALIGN_COF + cohesion * COHESION_COF;
            bee.vel = bee.vel.clamped(SPEED_LIMITS.0, SPEED_LIMITS.1);

            bee.pos += bee.vel * delta;
            match self.bounds.respective(bee.pos) {
                Sides::Left => bee.pos.x = self.bounds.max.x,
                Sides::Right => bee.pos.x = self.bounds.min.x,
                Sides::Top => bee.pos.y = self.bounds.min.y,
                Sides::Bottom => bee.pos.y = self.bounds.max.y,
                _ => {},
            }

            bee.add = self.map.update(Rect::cube(bee.pos, 0.0), i, &bee.add);
            s.draw(b, bee.pos, Vect::new(0.5f32, 0.5f32), bee.vel.ang(), &WHITE);
            self.e[i] = Some(bee);
        }
    }
}


fn main() {
    // creating window to draw to and event pump to read input
    let (mut window, mut event_pump, _gl, _s, _e) = Window::new(|sys| {
        sys.window("heureka", 1000, 600)
            .opengl()
            .resizable()
            .build()
            .unwrap()
    });

    window.set_background_color(&[0.5f32, 0.5f32, 0.5f32, 1f32]); //gray background
    window.canvas.set_camera(vect!(1000, 600), 0.5f32);

    // use of image crate to load image
    let img = image::open("C:/Users/jakub/Documents/programming/rust/src/rustbatch_examples/boids/src/bullets.png").unwrap();

    // This is wrapped opengl texture object
    let texture = Texture::from_img(
        &img,
        TextureConfig::DEFAULT,
    );

    let mut sprite = Sprite::new(texture.frame());

    let mut batch = Batch::new(texture);

    let mut fps = FPS::new(1f32);

    println!("{:?}", window.get_viewport_rect());
    let mut engine = Engine{
        e: vec![],
        col: vec![],
        col1: vec![],
        bounds: window.get_viewport_rect(),
        map: QuadMap::new(8 ,vect!(2000, 2000)),
        col2: vec![]
    };

    let mut rand = rand::thread_rng();

    for i in 0..10000{
        let mut bee = Bee{
            pos: Vect::new(rand.gen::<f32>() * 1000f32, rand.gen::<f32>() * 1000f32),
            vel: Vect::new(rand.gen::<f32>() * 100f32, rand.gen::<f32>() * 100f32),
            add: Address::ZERO,
        };
        bee.add = engine.map.insert(Rect::cube(bee.pos, 0.0), i);
        engine.e.push(Some(bee));
    }

    'main: loop {
        //polling events
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        let delta = fps.increase(0f32);

        window.clear();

        engine.update(&mut batch, &mut sprite, delta);

        batch.draw(&mut window.canvas);

        batch.clear();

        window.update();
    }
}

