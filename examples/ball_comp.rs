extern crate glutin;
extern crate gl;
#[macro_use]
extern crate exgui;
extern crate exgui_renderer_nanovg as renderer;

use glutin::{GlContext, ElementState, MouseButton};
use renderer::Renderer;
use exgui::{
    Component, Viewable, Shapeable, ChangeView, Node, Comp, Finger, GetError, Color,
    controller::MouseInput, Pct, Real, RealValue
};

struct Model {
    normal: bool,
}

#[derive(Copy, Clone)]
enum ModelMsg {
    Ball(BallMsg),
}

impl Component for Model {
    type Message = ModelMsg;
    type Properties = ();

    fn create(_props: &Self::Properties) -> Self {
        Model {
            normal: true,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            ModelMsg::Ball(BallMsg::Toggle) => {
                self.normal = !self.normal;
                ChangeView::Modify
            },
            _ => ChangeView::None,
        }
    }
}

impl Viewable<Model> for Model {
    fn view(&self) -> Node<Self> {
        egml! {
            <rect x = Pct(5), y = Pct(5), width = Pct(90), height = Pct(90),
                    fill = (Color::Yellow, 0.5), stroke = (Color::Blue, 3, 0.5),
                    modifier = |this, model: Model| {
                        this.stroke.as_mut()
                            .map(|s| s.paint = (
                                if model.normal { Color::Blue } else { Color::Red },
                                0.5
                            ).into());
                        this.fill.as_mut()
                            .map(|f| f.paint = (
                                if model.normal { Color::Yellow } else { Color::Green },
                                0.5
                            ).into());
                    }, >
                <Ball: id = "ball", pass_up = |msg| ModelMsg::Ball(msg), />
            </rect>
        }
    }
}

#[derive(Debug)]
struct Ball {
    orientation: BallOrientation,
    normal: bool,
    dir: i32,
    point_pct: Pct<Real>,
    old_pos_px: Option<Real>,
    radius: Real,
}

#[derive(Copy, Clone)]
enum BallMsg {
    Toggle,
    PosUpdate(RealValue, RealValue),
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq)]
enum BallOrientation {
    Horizontal,
    Vertical,
}

impl Default for BallOrientation {
    fn default() -> Self {
        BallOrientation::Vertical
    }
}

#[derive(Default, Clone, PartialEq)]
struct BallProps {
    orientation: BallOrientation,
}

impl Component for Ball {
    type Message = BallMsg;
    type Properties = BallProps;

    fn create(props: &Self::Properties) -> Self {
        Ball {
            orientation: props.orientation,
            normal: true,
            dir: 1,
            point_pct: 50.into(),
            old_pos_px: None,
            radius: 20.0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            BallMsg::Toggle => {
                self.normal = !self.normal;
                ChangeView::None
            },
            BallMsg::PosUpdate(last_x, last_y) => {
                let last_pos = match self.orientation {
                    BallOrientation::Horizontal => last_x.0,
                    BallOrientation::Vertical => last_y.0,
                };
                let step_pct = 0.5;

                if let Some(old_pos_px) = self.old_pos_px {
                    let radius_pct = self.radius / ((old_pos_px - last_pos).abs() / step_pct);

                    if self.point_pct <= radius_pct.into() && self.dir < 0 {
                        self.dir = 1;
                    } else if self.point_pct >= (100.0 - radius_pct).into() && self.dir > 0 {
                        self.dir = -1;
                    };
                }
                self.old_pos_px = Some(last_pos);

                self.point_pct += Pct(self.dir as Real * step_pct);
                ChangeView::Modify
            },
        }
    }
}

impl Viewable<Ball> for Ball {
    fn view(&self) -> Node<Self> {
        egml! {
            <circle cx = self.point_pct, cy = self.point_pct, r = self.radius,
                    fill = if self.normal { Color::Blue } else { Color::Red },
                    modifier = |this, model: Ball| {
                        match model.orientation {
                            BallOrientation::Horizontal => this.cx = model.point_pct.into(),
                            BallOrientation::Vertical => this.cy = model.point_pct.into(),
                        }
                    },
                    onclick = |_| BallMsg::Toggle, />
        }
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let mut mouse_controller = MouseInput::new();
    let window = glutin::WindowBuilder::new()
        .with_title("ExGUI ball comp")
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

        let circle = comp.get_comp(Finger::Id("ball"))
            .and_then(|ball| ball.get_prim::<Ball>(Finger::Root))
            .and_then(|prim| prim.circle().ok_or(GetError::NotFound))
            .expect("Can't get circle shape in Ball");
        let last_pos = (circle.cx, circle.cy);
        comp.send::<Model, Ball>(Finger::Id("ball"), BallMsg::PosUpdate(last_pos.0, last_pos.1));

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
