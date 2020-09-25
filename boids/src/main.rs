extern crate rustbatch;

use rustbatch::{sdl2, image, gl, rand, Mat};
use rustbatch::debug::FPS;
use rustbatch::{Window, Texture, Sprite, Batch};
use rustbatch::WHITE;
use rustbatch::math::vect::Vect;
use rustbatch::entity::scanner::Scanner;
use rustbatch::rand::Rng;
use rustbatch::math::rect::{Rect, Sides};
use std::mem::replace;

struct Bee {
    pos: Vect,
    vel: Vect,
}

struct Engine {
    e: Vec<Option<Bee>>,
    col: Vec<Vect>,
    col1: Vec<Vect>,
    bounds: Rect,
    map: Scanner<usize>,
    col2: Vec<usize>,
}

impl Engine {
    pub fn update(&mut self, b: &mut Batch, s: &mut Sprite, delta: f32) {
        for i in 0..self.e.len() {
            let mut bee = replace(&mut self.e[i], None).unwrap();
            let mut avoidance = Vect::ZERO;
            self.map.query_point(&bee.pos, &mut self.col2);
            for b in self.col2.iter() {
                let b = &self.e[*b as usize];
                if b.is_none() { continue }
                let b = b.as_ref().unwrap();
                let dif = b.pos - bee.pos;
                let len = dif.len();
                if len > 20f32 { continue }
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

            bee.vel += avoidance * 7.5f32  + alignment * 0.045f32 + cohesion * 0.03f32;
            bee.vel = bee.vel.clamped(15f32, 150f32);
            let prev = bee.pos;

            bee.pos += bee.vel * delta;
            match self.bounds.respective(&bee.pos) {
                Sides::Left => bee.pos.x = self.bounds.max.x,
                Sides::Right => bee.pos.x = self.bounds.min.x,
                Sides::Top => bee.pos.y = self.bounds.min.y,
                Sides::Bottom => bee.pos.y = self.bounds.max.y,
                _ => {},
            }
            self.map.update(&(prev), &(bee.pos), i);
            s.draw(b, &Mat::IM.scaled(Vect::ZERO, 0.5f32).rotated(Vect::ZERO, bee.vel.ang()).moved(bee.pos), &WHITE);
            self.e[i] = Some(bee);
        }
    }
}


fn main() {
    // creating window to draw to and event pump to read input
    let (mut window, mut event_pump, _gl) = Window::new(|sys| {
        sys.window("heureka", 1000, 600)
            .opengl()
            .resizable()
            .build()
            .unwrap()
    });

    window.set_background_color(&[0.5f32, 0.5f32, 0.5f32, 1f32]); //gray background
    window.set_camera(Mat::IM.moved(Vect::i64(-1000, -600)).scaled(Vect::ZERO, 0.5f32));

    // use of image crate to load image
    let img = image::open("C:/Users/jakub/Documents/programming/rust/src/rustbatch_examples/boids/src/bullets.png").unwrap();

    // This is wrapped opengl texture object
    let texture = Texture::from_img(
        &img,
        gl::NEAREST, // So the pixels are drawn as they are
        gl::RGBA // Color structure, you would use gl::RGB if your texture does not have alpha channel
    );

    // Creating sprite. Notice that sprite is just collection of points and it cannot be directly
    // drawn to window
    let mut sprite = Sprite::new(texture.frame());

    // On the other hand batch owns texture witch can be drawn to window
    let mut batch = Batch::new(texture);

    // this is just a little helper
    let mut fps = FPS::new(1f32);


    let mut engine = Engine{
        e: vec![],
        col: vec![],
        col1: vec![],
        bounds: window.get_viewport_rect(),
        map: Scanner::new(200, 120, Vect::i32(15, 15)),
        col2: vec![]
    };

    let mut rand = rand::thread_rng();

    for i in 0..10000 {
        let bee = Bee{
            pos: Vect::new(rand.gen::<f32>() * 1000f32, rand.gen::<f32>() * 1000f32),
            vel: Vect::new(rand.gen::<f32>() * 100f32, rand.gen::<f32>() * 100f32),
        };
        engine.map.insert(&bee.pos, i);
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

        // i hope you know how to get delta on your own but fps returns is as bonus if you supply
        // 0f32 as delta
        let delta = fps.increase(0f32);

        // clearing window
        window.clear();

        engine.update(&mut batch, &mut sprite, delta);




        window.draw(&batch);

        // Don't forget to clear batch if you not planning to use it as canvas
        // after all drawing sprites to batch takes some time
        batch.clear();

        // finishing with window update so you can se it changing
        window.update();
    }
}