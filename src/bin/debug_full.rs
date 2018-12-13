use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use std::thread;

use glutin_window::GlutinWindow as Window;
use graphics::{Image, ImageSize};
use image;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;

use rqrr;
use std::ops::RangeInclusive;

pub struct State {
    overlay: Option<Texture>,
    cursor: Option<(usize, usize)>,
    caps: Vec<rqrr::identify::CapStone>,
    grid: Option<(usize, rqrr::identify::Perspective)>,
}


pub struct App {
    gl: GlGraphics,
    // OpenGL drawing backend.
    tex: Texture,
    img: Image,
    internal: State,
    recv: Receiver<Update>,
    running: bool,
}

impl App {
    pub fn new(opengl: OpenGL, rgba: &image::RgbaImage, recv: Receiver<Update>) -> Self {
        let gl = GlGraphics::new(opengl);
        let img = Image::new();
        let tex = Texture::from_image(rgba, &TextureSettings::new());
        App {
            gl,
            tex,
            img,
            recv,
            internal: State {
                overlay: None,
                cursor: None,
                caps: Vec::new(),
                grid: None,
            },
            running: false,
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let w = args.width;
        let tex = &self.tex;
        let img = &self.img;
        let overlay = &self.internal.overlay;
        let cursor = &self.internal.cursor;
        let caps = &self.internal.caps;
        let grid = &self.internal.grid;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            img.draw(tex, &c.draw_state, c.transform, gl);

            if let &Some(ref ov) = overlay {
                img.draw(ov, &c.draw_state, c.transform, gl);
            }

            if let &Some((ref x, ref y)) = cursor {
                let line = line::Line::new([0.0, 1.0, 0.0, 1.0], 2.0);
                line.draw([0.0, *y as f64, w, *y as f64], &c.draw_state, c.transform, gl);
            }

            for cap in caps {
                let lines = line_from_pers(&cap.c);
                let l = line::Line::new([1.0, 1.0, 0.0, 1.0], 1.0);
                l.draw(lines[0], &c.draw_state, c.transform, gl);
                l.draw(lines[1], &c.draw_state, c.transform, gl);
            }

            if let &Some((ref u, ref p)) = grid {
                let lines = lines_for_grid(*u, p);
                let d = line::Line::new([0.2, 0.2, 1.0, 1.0], 1.0);
                for l in lines {
                    d.draw(l, &c.draw_state, c.transform, gl);
                }
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        if !self.running {
            return;
        }

        match self.recv.try_recv() {
            Ok(s) => {
                match s {
                    Update::CodeImage(w, h, px) => {
                        let tex = to_texture(w, h, &px);
                        self.internal.overlay = Some(tex);
                    }
                    Update::Scan(img, x, y) => {
                        let tex = to_texture(img.w, img.h, &img.pixels);
                        self.internal.overlay = Some(tex);
                        self.internal.cursor = Some((*x.start(), y));
                    }
                    Update::Cap(cap) => {
                        self.internal.caps.push(cap)
                    },
                    Update::Stop => {
                        self.running = false;
                    },
                    Update::ClearScan => {
                        self.internal.cursor = None;
                    },
                    Update::Grid(size, p) => {
                        self.internal.grid = Some((size, p));
                    }
                }
            }
            Err(_) => (),
        };
    }
}

fn lines_for_grid(size: usize, pers: &rqrr::identify::Perspective) -> (Vec<[f64; 4]>) {
    let mut lines = Vec::new();
    for i in 0..=size {
        let start_hor = pers.map(0.0, i as f64);
        let end_hor = pers.map(size as f64, i as f64);
        let start_ver = pers.map(i as f64, 0.0);
        let end_ver = pers.map(i as f64, size as f64);

        lines.push([start_hor.x as f64, start_hor.y as f64, end_hor.x as f64, end_hor.y as f64]);
        lines.push([start_ver.x as f64, start_ver.y as f64, end_ver.x as f64, end_ver.y as f64]);
    }

    lines
}

fn line_from_pers(pers: &rqrr::identify::Perspective) -> [[f64; 4]; 2] {
    let start = pers.map(0.0, 0.0);
    let hor_end = pers.map(7.0, 0.0);
    let ver_end = pers.map(0.0, 7.0);
    let hor = [
        start.x as f64,
        start.y as f64,
        hor_end.x as f64,
        hor_end.y as f64,
    ];
    let ver = [
        start.x as f64,
        start.y as f64,
        ver_end.x as f64,
        ver_end.y as f64,
    ];

    [hor, ver]
}

fn to_texture(w: usize, h: usize, buf: &[rqrr::identify::PixelColor]) -> Texture {
    let img: image::RgbaImage = image::ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
        let p = buf[y as usize * w + x as usize];
        use rqrr::identify::PixelColor::*;
        let col = match p {
            White
            | TimingWhite => {
                [255, 255, 255, 255]
            }
            Black
            | TimingBlack => {
                [0, 0, 0, 255]
            }
            CheckCapstone => [128, 0, 0, 255],
            CapStone => [255, 0, 0, 255],
            _ => {
                [0, 255, 0, 255]
            }
        };
        image::Rgba(col)
    });
    Texture::from_image(&img, &TextureSettings::new())
}

fn main() {
    use piston::window::AdvancedWindow;
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    let img = image::open("tests/data/code2.jpg").unwrap();
    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
        "rqrr",
        [1024, 768],
    )
        .opengl(opengl)
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

    let (send, recv) = sync_channel(1);
    let grey = img.to_luma();
    let hdl = thread::spawn(move || {
        decode(grey, send)
    });

    // Create a new game and run it.
    let mut app = App::new(opengl, &img.to_rgba(), recv);
    window.set_size(app.tex.get_size());

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }

        if let Some(u) = e.press_args() {
            match u {
                Button::Keyboard(Key::P) => {
                    app.running = false;
                },
                Button::Keyboard(Key::C) =>  {
                    app.running = true;
                }
                Button::Keyboard(Key::Right) => {
                    let old = events.get_event_settings().ups;
                    events.set_ups(old * 2);
                }
                Button::Keyboard(Key::Left) => {
                    let old = events.get_event_settings().ups;
                    events.set_ups(old / 2);
                }
                _ => ()
            }
        }
    }
}

pub enum Update {
    CodeImage(usize, usize, Vec<rqrr::identify::PixelColor>),
    Scan(rqrr::identify::Image, RangeInclusive<usize>, usize),
    Cap(rqrr::identify::CapStone),
    Grid(usize, rqrr::identify::Perspective),
    ClearScan,
    Stop,
}

fn decode(img: image::GrayImage, update: SyncSender<Update>) {
    use image::GenericImageView;
    let w = img.width() as usize;
    let h = img.height() as usize;

    let img_lock = update.clone();
    let mut code_img = rqrr::identify::Image::from_greyscale_debug(w, h, |x, y| {
        img.get_pixel(x as u32, y as u32).data[0]
    }, |w, h, px| update.send(Update::CodeImage(w, h, px.to_vec())).unwrap(),
    );

    update.send(Update::Stop).unwrap();

    let mut lasty = 0;

    let caps = rqrr::identify::capstones_from_image_with_debug(&mut code_img, |img, event, x, y| {
        if lasty <= y && event == "line done" {
            let px = img.clone();
            update.send(Update::Scan(px, x, y)).unwrap();
            lasty = y + 1;
        }
    },
    |_, _, _, _| {
    });
    update.send(Update::ClearScan).unwrap();
    update.send(Update::Stop).unwrap();

    let caps = rqrr::identify::find_groupings(caps);
    for group in caps {
        let grid = rqrr::identify::Grid::from_group_debug(&mut code_img, group, |img, pers| {
            update.send(Update::CodeImage(img.w, img.h, img.pixels.to_vec())).unwrap();

            if let Some((s, p)) = pers {
                update.send(Update::Grid(s, p.clone())).unwrap();
            }
        });
    }

}
