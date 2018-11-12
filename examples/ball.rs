extern crate glutin;
extern crate gl;
#[macro_use]
extern crate exgui;
extern crate exgui_renderer_nanovg as renderer;

use glutin::{GlContext, ElementState, MouseButton};
use renderer::Renderer;
use exgui::{ModelComponent, Viewable, Node, Comp, Color, controller::MouseInput};

#[derive(Debug)]
struct Ball {
    normal: bool,
    dir: i32,
    cy: f32,
}

pub enum Msg {
    Toggle,
    Nope,
    PosUpdate,
}

impl ModelComponent for Ball {
    type Message = Msg;
    type Properties = ();

    fn create(_props: &<Self as ModelComponent>::Properties) -> Self {
        Ball {
            normal: true,
            dir: 1,
            cy: 50.0,
        }
    }

    fn update(&mut self, msg: <Self as ModelComponent>::Message) -> bool {
        match msg {
            Msg::Toggle => {
                self.normal = !self.normal;
                true
            }
            Msg::PosUpdate => {
                self.dir = if self.cy <= 70.0 && self.dir < 0 {
                    1
                } else if self.cy >= 330.0 && self.dir > 0 {
                    -1
                } else {
                    self.dir
                };
                self.cy += (self.dir * 2) as f32;
                false
            },
            Msg::Nope => false,
        }
    }
}

impl Viewable<Ball> for Ball {
    fn view(&self) -> Node<Self> {
        egml! {
            <group translate = Some((50.0, 50.0).into()), >
                <rect x = 0.0, y = 0.0, width = 300.0, height = 300.0,
                        fill = None, stroke = Some((Color::Black, 2.0, 0.5).into()), >
                    <circle cx = 150.0, cy = self.cy, r = 20.0,
                            fill = Some(if self.normal { Color::Blue } else { Color::Red }.into()),
                            modifier = |this, model: Ball| {
                                this.cy = model.cy;
                            },
                            onclick = |_| Msg::Toggle, />
                </rect>
            </group>
        }
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let mut mouse_controller = MouseInput::new();
    let window = glutin::WindowBuilder::new()
        .with_title("ExGUI ball")
        .with_dimensions(480, 480);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4)
        .with_srgb(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.8, 0.8, 0.8, 1.0);
    }

    let mut comp = Comp::new::<Ball>(());
    comp.resolve(None);

    let mut render = Renderer::new();

    let mut running = true;
    while running {
        let (width, height) = gl_window.get_inner_size().unwrap();
        let (width, height) = (width as i32, height as i32);
        unsafe {
            gl::Viewport(0, 0, width, height);
            gl::Clear(
                gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT,
            );
        }

        comp.send::<Ball>(Msg::PosUpdate);
        comp.modify();

        render.width = width as f32;
        render.height = height as f32;
        render.device_pixel_ratio = gl_window.hidpi_factor();
        render.render(comp.view_node::<Ball>());

        gl_window.swap_buffers().unwrap();

        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => {
                match event {
                    glutin::WindowEvent::Closed => running = false,
                    glutin::WindowEvent::Resized(w, h) => gl_window.resize(w, h),
                    glutin::WindowEvent::CursorMoved { position: (x_pos, y_pos), .. } => {
                        mouse_controller.update_pos(x_pos, y_pos);
                    },
                    glutin::WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                        mouse_controller.left_pressed_comp(&mut comp);
                    },
                    _ => (),
                }
            }
            _ => (),
        });
    }
}
