use glow::HasContext;
use glutin::event::ElementState;
use glutin::event::MouseButton;
use instrument::InstrumentParams;
use instrument::UIThreadContext;
use instrument::initialize_audio;
use minvect::*;
extern crate glow_mesh;
use glow_mesh::xyzrgba::*;
use glow_mesh::xyzrgba_build2d::*;
use glutin::event::{Event, WindowEvent};
use xypanel::XYPanel;

mod xypanel;
mod instrument;


pub struct CrazySynth {
    xres: i32,
    yres: i32,
    window: glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>,
    gl: glow::Context,

    prog: ProgramXYZRGBA,

    mouse_pos: Vec2,
    mouse_lmb_held: bool,

    top_left: XYPanel,
    top_right: XYPanel,
    bot: XYPanel,

    cx: UIThreadContext,
}

impl CrazySynth {
    pub fn new(event_loop: &glutin::event_loop::EventLoop<()>) -> Self {
        let xres = 800;
        let yres = 512;
    
        unsafe {
            let window_builder = glutin::window::WindowBuilder::new()
                .with_title("crazy synth")
                .with_inner_size(glutin::dpi::PhysicalSize::new(xres, yres));
            let window = glutin::ContextBuilder::new()
                .with_vsync(true)
                .build_windowed(window_builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap();
    
            let gl = glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
    
            let prog = ProgramXYZRGBA::default(&gl);
            prog.bind(&gl);
            let mat4_ident = [1.0f32, 0., 0., 0., 0., -1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1. ];
            prog.set_proj(&mat4_ident, &gl);

            let p_default = InstrumentParams {
                a: 0.0,
                b: 0.0,
                c: 0.0,
                d: 0.0,
                e: 0.0,
                f: 0.0,
            };

            CrazySynth {
                xres,
                yres,
                window,
                gl,
                prog,
                mouse_lmb_held: false,
                mouse_pos: vec2(0.0, 0.0),
                top_left: XYPanel::new([
                        0.45, 0.0, -0.5,
                        0.0, 0.45, -0.5,
                        0.0, 0.0, 1.0],
                        vec2(p_default.a, p_default.b)
                ),
                top_right: XYPanel::new([
                        0.45, 0.0, 0.5,
                        0.0, 0.45, -0.5,
                        0.0, 0.0, 1.0],
                        vec2(p_default.c, p_default.d)
                ),
                bot: XYPanel::new([
                        0.95, 0.0, 0.0,
                        0.0, 0.45, 0.5,
                        0.0, 0.0, 1.0],
                        vec2(p_default.e, p_default.f)
                ),
                cx: initialize_audio(p_default),

                
                // top_right: XYPanel::new(vec2(-0.95, -0.95), vec2(0.95, 0.95), vec2(0.5, 0.5)),
                // bot: XYPanel::new(vec2(-0.095, 0.05), vec2(0.95, 0.95), vec2(0.5, 0.5)),
            }
        }
    }

    pub fn handle_event(&mut self, event: glutin::event::Event<()>) {
        unsafe {
            match event {
                Event::LoopDestroyed |
                Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
                    std::process::exit(0);
                },

                Event::WindowEvent {event, .. } => {
                    match event {
                        WindowEvent::Resized(size) => {
                            self.xres = size.width as i32;
                            self.yres = size.height as i32;
                            self.window.resize(size);
                            self.gl.viewport(0, 0, size.width as i32, size.height as i32);
                        },
                        WindowEvent::MouseInput{device_id, state, button, modifiers} => {
                            if button == MouseButton::Left {
                                if state == ElementState::Pressed {
                                    self.mouse_lmb_held = true;
                                } else {
                                    self.mouse_lmb_held = false;
                                }
                            }
                        },
                        WindowEvent::CursorMoved { device_id, position, modifiers } => {
                            let x = (position.x as f32 / self.xres as f32) * 2.0 - 1.0;
                            let y = (position.y as f32 / self.yres as f32) * 2.0 - 1.0;
                            self.mouse_pos = vec2(x, y);
                        }
                        _ => {},
                    }
                },
                Event::MainEventsCleared => {
                    self.gl.clear_color(0.5, 0.5, 0.5, 1.0);
                    self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                    let mut update = false; 
                    if self.mouse_lmb_held {
                        update |= self.top_left.update(self.mouse_pos);
                        update |= self.top_right.update(self.mouse_pos);
                        update |= self.bot.update(self.mouse_pos);
                    }
                    if update {
                        self.cx.send_struct(InstrumentParams { 
                            a: self.top_left.p.x,
                            b: self.top_left.p.y,
                            c: self.top_right.p.x,
                            d: self.top_right.p.y,
                            e: self.bot.p.x,
                            f: self.bot.p.y,
                        });
                        println!("{:.1} {:.1}|{:.3} {:.3}|{:.1} {:.1}", 
                            self.top_left.p.x,
                            self.top_left.p.y,
                            self.top_right.p.x,
                            self.top_right.p.y,
                            self.bot.p.x,
                            self.bot.p.y,
                        );
                    }
                    let depth = 0.0;
                    let mut buf = vec![];
                    self.top_left.push_geometry(&mut buf, depth);
                    self.top_right.push_geometry(&mut buf, depth);
                    self.bot.push_geometry(&mut buf, depth);
                    let h = upload_xyzrgba_mesh(&buf, &self.gl);
                    h.render(&self.gl);
                    self.window.swap_buffers().unwrap();
                },
                _ => {},
            }
        }
    }
}

// pub fn put rect vec2 min vec2 max..... for sure
// w h calculations

pub fn main() {
        let event_loop = glutin::event_loop::EventLoop::new();
        let mut demo = CrazySynth::new(&event_loop);
        event_loop.run(move |event, _, _| demo.handle_event(event));
}

pub fn put_rect(buf: &mut Vec<XYZRGBA>, min: Vec2, max: Vec2, col: Vec4, depth: f32) {
    let a = min;
    let b = vec2(max.x, min.y);
    let c = max;
    let d = vec2(min.x, max.y);

    put_quad(buf, a, b, c, d, col, depth);
}