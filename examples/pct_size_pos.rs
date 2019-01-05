extern crate glutin;
extern crate gl;
#[macro_use]
extern crate exgui;
extern crate exgui_renderer_nanovg as renderer;

use glutin::{GlContext, ElementState, MouseButton};
use renderer::Renderer;
use exgui::{Pct, Component, Viewable, ChangeView, Node, Comp, Color, AlignHor::*, AlignVer::*, controller::MouseInput};

#[derive(Debug)]
struct Model;

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: &Self::Properties) -> Self {
        Model
    }

    fn update(&mut self, _msg: Self::Message) -> ChangeView {
        ChangeView::None
    }
}

impl Viewable<Model> for Model {
    fn view(&self) -> Node<Self> {
        egml! {
            <rect x = Pct(5), y = Pct(5), width = Pct(90), height = Pct(90), stroke = (Color::Blue, 5), >
                <circle cx = Pct(50), cy = Pct(50), stroke = (Color::Yellow, 3), >
                    <text x = Pct(50), y = Pct(50), font_name = "Roboto", font_size = 24,
                            align = (Center, Middle), fill = Color::Red, >
                        { "Some text in circle" }
                    </text>
                </circle>
            </rect>
        }
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let mut mouse_controller = MouseInput::new();
    let window = glutin::WindowBuilder::new()
        .with_title("ExGUI pct size and pos")
        .with_dimensions(480, 480);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(8)
        .with_srgb(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.8, 0.8, 0.8, 1.0);
    }

    let mut comp = Comp::new::<Model>(());
    comp.resolve(None);

    let mut render = Renderer::new();
    render.load_font("Roboto", "resources/Roboto-Regular.ttf");

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
        render.render(comp.view_node_mut::<Model>());

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
