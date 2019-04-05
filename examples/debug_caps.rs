use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use std::thread;

use glutin_window::GlutinWindow as Window;
use graphics::{Image};
use image;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings, Filter};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;

use rqrr;
use rqrr::identify::Point;
use std::ops::RangeInclusive;


pub struct State {
    running: bool,
    overlay: Option<Texture>,
    cursor: Option<(RangeInclusive<usize>, usize)>,
    corners: Option<Corners>,
    perspective: Option<rqrr::identify::Perspective>,
}


pub struct App {
    gl: GlGraphics,
    img: Image,
    internal: State,
    recv: Receiver<Update>,
}


const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const CURSOR: [f32; 4] = [0.2, 0.8, 0.2, 1.0];
const LINE: [f32; 4] = [0.8, 0.2, 0.2, 1.0];
const SCALE: f64 = 40.0;

impl App {
    fn new(opengl: OpenGL, recv: Receiver<Update>) -> Self {
        let gl = GlGraphics::new(opengl);
        let img = Image::new();
        App {
            gl,
            recv,
            img,
            internal: State {
                running: false,
                overlay: None,
                cursor: None,
                corners: None,
                perspective: None,
            },
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let w = args.width;
        let overlay = &self.internal.overlay;
        let cursor = &self.internal.cursor;
        let corners = &self.internal.corners;
        let pers = &self.internal.perspective;
        let img = &self.img;
        self.gl.viewport(0, 0, 18, 18);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(WHITE, gl);
            let trans = c.transform.scale(SCALE, SCALE);

            if let &Some(ref ov) = overlay {
                img.draw(ov, &c.draw_state, trans, gl);
            }

            if let &Some((ref x, ref y)) = cursor {
                let w = x.end() - x.start() + 1;
                let col = if w > 1 {
                    LINE
                } else {
                    CURSOR
                };
                let rect = rectangle::Rectangle::new(col);
                rect.draw([*x.start() as f64, *y as f64, w as f64, 1.0], &c.draw_state, trans, gl);
            }

            if let &Some(ref co) = corners {
                let rect = rectangle::Rectangle::new([0.3, 0.3, 1.0, 1.0]);
                let ref ref_0 = co.0;
                rect.draw([ref_0.x as f64, ref_0.y as f64, 1.0, 1.0], &c.draw_state, trans, gl);

                let rect = rectangle::Rectangle::new([0.5, 0.0, 0.5, 1.0]);
                let ref first = co.1;
                rect.draw([first.x as f64, first.y as f64, 1.0, 1.0], &c.draw_state, trans, gl);

                if let Some(ref o) = co.2 {
                    for p in o {
                        rect.draw([p.x as f64, p.y as f64, 1.0, 1.0], &c.draw_state, trans, gl);
                    }
                }
            }

            if let &Some(ref p) = pers {
                let trans = trans.trans(0.5, 0.5);

                let line = line::Line::new([0.0, 0.0, 0.0, 1.0], 2.0/40.0);
                let start = p.map(0.0, 0.0);
                let end_hor = p.map(7.0, 0.0);
                let end_ver = p.map(0.0, 7.0);
                let end_end = p.map(7.0, 7.0);

                line.draw([start.x as f64, start.y as f64, end_hor.x as f64, end_hor.y as f64], &c.draw_state, trans, gl);
                line.draw([start.x as f64, start.y as f64, end_ver.x as f64, end_ver.y as f64], &c.draw_state, trans, gl);
                line.draw([end_hor.x as f64, end_hor.y as f64, end_end.x as f64, end_end.y as f64], &c.draw_state, trans, gl);
                line.draw([end_ver.x as f64, end_ver.y as f64, end_end.x as f64, end_end.y as f64], &c.draw_state, trans, gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        if !self.internal.running {
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
                        self.internal.cursor = Some((x, y));
                    }
                    Update::Reset => {
                        self.internal.running = false;
                        self.internal.cursor = None;
                        self.internal.overlay = None;
                        self.internal.corners = None;
                        self.internal.perspective = None;
                    }
                    Update::ScanDone => {
                        self.internal.cursor = None;
                    },
                    Update::Perspective(p) => {
                        self.internal.perspective = Some(p);
                    }
                    Update::Corners(x, y, c) => {
                        self.internal.cursor = Some((x, y));
                        self.internal.corners = Some(c);
                    }
                    Update::Stop => {
                        self.internal.running = false;
                    }
                    Update::Start => {
                        self.internal.running = true;
                    }
                }
            }
            Err(_) => (),
        };
    }
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
    let mut set = TextureSettings::new();
    set.set_filter(Filter::Nearest);

    Texture::from_image(&img, &set)
}

fn main() {
    use piston::window::AdvancedWindow;
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    let img = image::open("tests/data/cap/cap.png").unwrap();
    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
        "rqrr",
        [720, 720],
    )
        .opengl(opengl)
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

    let (send, recv) = sync_channel(1);
    let grey = img.to_luma();

    let decode_send = send.clone();
    let _hdl = thread::spawn(move || {
        decode(grey, decode_send)
    });

    // Create a new game and run it.
    let mut app = App::new(opengl, recv);
    window.set_size((720, 720));

    let mut events = Events::new(EventSettings::new().ups(2));
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
                    app.internal.running = false;
                }
                Button::Keyboard(Key::C) => {
                    app.internal.running = true;
                }
                Button::Keyboard(Key::Right) => {
                    let old = events.get_event_settings().ups;
                    events.set_ups(old * 2);
                }
                Button::Keyboard(Key::Left) => {
                    let old = events.get_event_settings().ups;
                    let old = ::std::cmp::max(old, 1);
                    events.set_ups(old / 2);
                }
                _ => ()
            }
        }
    }
}

struct Corners(Point, Point, Option<[Point; 3]>);

enum Update {
    CodeImage(usize, usize, Vec<rqrr::identify::PixelColor>),
    Scan(rqrr::identify::Image, RangeInclusive<usize>, usize),
    Corners(RangeInclusive<usize>, usize, Corners),
    Perspective(rqrr::identify::Perspective),
    ScanDone,
    Reset,
    Stop,
    Start,
}

fn decode(img: image::GrayImage, update: SyncSender<Update>) {
    let w = img.width() as usize;
    let h = img.height() as usize;

    let img_lock = update.clone();
    let mut code_img = rqrr::identify::Image::from_greyscale_debug(w, h, |x, y| {
        img.get_pixel(x as u32, y as u32).data[0]
    }, |_, _, _| ());

    update.send(Update::CodeImage(w, h, code_img.pixels.to_vec())).unwrap();

    let mut start_corners = false;
    let mut start_all_corners = false;

    let caps = rqrr::identify::capstones_from_image(&mut code_img);
    update.send(Update::ScanDone).unwrap();

    if caps.len() > 0 {
        update.send(Update::Perspective(caps[0].c.clone())).unwrap();
    }
}
