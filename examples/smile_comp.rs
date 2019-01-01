extern crate glutin;
extern crate gl;
#[macro_use]
extern crate exgui;
extern crate exgui_renderer_nanovg as renderer;

use glutin::{GlContext, ElementState, MouseButton};
use renderer::Renderer;
use exgui::{Component, Viewable, ChangeView, Node, Comp, Color, Stroke, LineJoin, PathCommand::*, controller::MouseInput};

#[derive(Debug)]
struct Smile {
    normal_face: bool,
}

pub enum Msg {
    ToggleFace,
}

impl Component for Smile {
    type Message = Msg;
    type Properties = ();

    fn create(_props: &Self::Properties) -> Self {
        Smile {
            normal_face: true,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            Msg::ToggleFace => {
                self.normal_face = !self.normal_face;
                ChangeView::Rebuild
            }
        }
    }
}

impl Viewable<Smile> for Smile {
    fn view(&self) -> Node<Self> {
        egml! {
            <group stroke = (Color::Black, 5), >
                <circle cx = 150, cy = 150, r = 100,
                    fill = if self.normal_face { Color::Yellow } else { Color::Red },
                    onclick = |_| Msg::ToggleFace, />
                <group fill = if self.normal_face { Color::Black } else { Color::White }, >
                    <circle cx = 110, cy = 130, r = 15, />
                    <circle cx = 190, cy = 130, r = 15, />
                    { self.view_mouth() }
                </group>
            </group>
        }
    }
}

impl Smile {
    fn view_mouth(&self) -> Node<Self> {
        if self.normal_face {
            egml! {
                <path cmd = vec![Move([100.0, 180.0]), BezCtrl([150.0, 230.0]), QuadBezTo([200.0, 180.0]), BezCtrl([150.0, 210.0]), QuadBezTo([100.0, 180.0])],
                        stroke = Stroke { width: 5.0, line_join: LineJoin::Round, ..Default::default() }, />
            }
        } else {
            egml! {
                <path cmd = vec![Move([100.0, 205.0]), BezCtrl([150.0, 155.0]), QuadBezTo([200.0, 205.0]), BezCtrl([150.0, 175.0]), QuadBezTo([100.0, 205.0])],
                        stroke = Stroke { width: 5.0, line_join: LineJoin::Round, ..Default::default() }, />
            }
        }
    }
}

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
            <group translate = (50, 50), >
                <rect x = 0, y = 0, width = 300, height = 300,
                        fill = None, stroke = (Color::Black, 2, 0.5), >
                    <Smile: />
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

    let mut comp = Comp::new::<Model>(());
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
