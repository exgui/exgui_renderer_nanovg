extern crate glutin;
extern crate gl;
#[macro_use]
extern crate exgui;
extern crate exgui_renderer_nanovg as renderer;

use glutin::{GlContext, ElementState, MouseButton};
use renderer::Renderer;
use exgui::{ModelComponent, Viewable, Node, Color, controller::MouseInput};

struct SmileModel {
    normal_face: bool,
}

pub enum Msg {
    ToggleFace,
    Nope,
}

impl ModelComponent for SmileModel {
    type Message = Msg;
    type Properties = ();

    fn update(&mut self, msg: <Self as ModelComponent>::Message) -> bool {
        match msg {
            Msg::ToggleFace => {
                self.normal_face = !self.normal_face;
                true
            }
            Msg::Nope => false,
        }
    }
}

impl Viewable<SmileModel> for SmileModel {
    fn view(&self) -> Node<Self> {
        egml! {
            <group translate = Some((50.0, 50.0).into()), >
                <rect x = 0.0, y = 0.0, width = 300.0, height = 300.0,
                        fill = None, stroke = Some((Color::Black, 2.0, 0.5).into()), >
                    <group fill = Some((Color::Black, 0.6).into()), stroke = Some((Color::Black, 5.0).into()), >
                        <circle cx = 150.0, cy = 150.0, r = 100.0,
                            fill = Some(if self.normal_face { Color::Yellow } else { Color::Red }.into()),
                            onclick = |_| Msg::ToggleFace, />
                        <circle cx = 110.0, cy = 130.0, r = 15.0, />
                        <circle cx = 190.0, cy = 130.0, r = 15.0, />
                    </group>
                </rect>
            </group>
        }
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let mut mouse_controller = MouseInput::new();
    let window = glutin::WindowBuilder::new()
        .with_title("ExGUI app")
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

    let mut smile = SmileModel {
        normal_face: true,
    };
    let mut smile_node = smile.view();
    smile_node.resolve(None);

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
        render.render(&smile_node);

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
                        if mouse_controller.left_pressed(&mut smile, &smile_node) {
                            smile_node = smile.view();
                            smile_node.resolve(None);
                        }
                    },
                    _ => {}
                }
            }
            _ => {}
        });
    }
}
