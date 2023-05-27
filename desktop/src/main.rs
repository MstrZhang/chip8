use chip8_core::*;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use std::env;
use std::fs::File;
use std::io::Read;

// arbitrary value; scale factor
const SCALE: u32 = 15;

const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

// arbitrary value; CHIP-8 spec doesn't say anything about how fast clock speed should be
const TICKS_PER_FRAME: usize = 10;

fn main() {
    let args: Vec<_> = env::args().collect();

    // accept only path to game otherwise exit with error
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    // setup SDL window
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    // poll for events every game loop
    let mut event_pump = sdl_context.event_pump().unwrap();

    // instantiate emulation object
    let mut chip8 = Emu::new();

    // args[0] is the name of the file
    let mut rom = File::open(&args[1]).expect("unable to open file");
    let mut buffer = Vec::new();

    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

    'gameloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'gameloop,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, false);
                    }
                }
                _ => (),
            }
        }

        // clock speed is 10 ticks per frame (arbitrary value)
        for _ in 0..TICKS_PER_FRAME {
            chip8.tick();
        }

        // timers tick once per frame
        chip8.tick_timers();

        // game draws at 60 Hz
        draw_screen(&chip8, &mut canvas);
    }
}

// draw loop
fn draw_screen(emu: &Emu, canvas: &mut Canvas<Window>) {
    // clear the canvas as black
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = emu.get_display();
    // set draw color to white, iterate through each point and see if it should be drawn
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            // convert 1D array's index into a 2D (x, y) position
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            // draw rectangle at (x, y) scaled up by `SCALE` factor
            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }

    canvas.present();
}

// key mapper
fn key2btn(key: Keycode) -> Option<usize> {
    //  keyboard             CHIP-8
    //  +---+---+---+---+    +---+---+---+---+
    //  | 1 | 2 | 3 | 4 |    | 1 | 2 | 3 | C |
    //  +---+---+---+---+    +---+---+---+---+
    //  | Q | W | E | R |    | 4 | 5 | 6 | D |
    //  +---+---+---+---+    +---+---+---+---+
    //  | A | S | D | F |    | 7 | 8 | 9 | E |
    //  +---+---+---+---+    +---+---+---+---+
    //  | Z | X | C | V |    | A | 0 | B | F |
    //  +---+---+---+---+    +---+---+---+---+

    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0x4),
        Keycode::Q => Some(0x5),
        Keycode::W => Some(0x6),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xB),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
