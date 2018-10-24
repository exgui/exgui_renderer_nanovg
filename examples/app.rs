extern crate glutin;
extern crate gl;
#[macro_use]
extern crate exgui;
extern crate exgui_renderer_nanovg as renderer;

use glutin::GlContext;
use renderer::Renderer;
use exgui::{ModelComponent, Viewable, Node, Color, Fill, Stroke};

struct Model;

impl ModelComponent for Model {
    type Message = ();
    type Properties = ();

    fn update(&mut self, _msg: <Self as ModelComponent>::Message) -> bool {
        unimplemented!()
    }
}

impl Viewable<Model> for Model {
    fn view(&self) -> Node<Self> {
        let rect_fill = Some(Fill { color: Color::Yellow, transparent: 0.0 });
        egml! {
            <rect x = 40.0, y = 40.0, width = 50.0, height = 80.0, fill = rect_fill,
                    stroke = Some(Stroke { color: Color::Red, width: 1.0, transparent: 0.0 }), >
                <group>
                    <circle cx = 120.0, cy = 120.0, r = 40.0, />
                </group>
            </rect>
        }
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("ExGUI app")
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

    let node = Model.view();
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

        render.width = width as f32;
        render.height = height as f32;
        render.device_pixel_ratio = gl_window.hidpi_factor();
        render.render(&node);

        gl_window.swap_buffers().unwrap();

        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => {
                match event {
                    glutin::WindowEvent::Closed => running = false,
                    glutin::WindowEvent::Resized(w, h) => gl_window.resize(w, h),
                    _ => {}
                }
            }
            _ => {}
        });
    }
}
