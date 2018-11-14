extern crate glutin;
extern crate gl;
#[macro_use]
extern crate exgui;
extern crate exgui_renderer_nanovg as renderer;
extern crate chrono;

use std::f32::consts::PI;
use glutin::{GlContext, ElementState, MouseButton};
use renderer::Renderer;
use exgui::{
    ModelComponent, Viewable, Node, Comp, Color, Gradient,
    PathCommand::*, Transform, controller::MouseInput
};
use chrono::{DateTime, Local, Timelike, Datelike};

const INIT_WINDOW_SIZE: (u32, u32) = (480, 480);

#[derive(Debug, Default)]
struct Clock {
    clock_size: i32,
    dial_radius: f32,
    dial_center: (f32, f32),

    am: bool,
    hour: f32,
    minute: f32,
    second: f32,

    year: i32,
    month: u32,
    day: u32,

    hour_angle: f32,
    minute_angle: f32,
    second_angle: f32,
}

pub enum Msg {
    ResizeWindow(i32, i32),
    Tick,
}

impl ModelComponent for Clock {
    type Message = Msg;
    type Properties = ();

    fn create(_props: &<Self as ModelComponent>::Properties) -> Self {
        let (width, height) = INIT_WINDOW_SIZE;
        let mut clock = Clock::default();
        clock.size_recalc(width as i32, height as i32);
        clock
    }

    fn update(&mut self, msg: <Self as ModelComponent>::Message) -> bool {
        match msg {
            Msg::ResizeWindow(w, h) => {
                self.size_recalc(w, h)
            }
            Msg::Tick => {
                let dt: DateTime<Local> = Local::now(); // e.g. `2018-11-28T21:45:59.324310806+09:00`
                let hour = dt.hour();

                self.am = hour < 12;
                self.hour = f64::from(hour % 12) as f32;
                self.minute = f64::from(dt.minute()) as f32;
                self.second = f64::from(dt.second()) as f32;

                self.year = dt.year();
                self.month = dt.month();
                self.day = dt.day();

                let two_pi = 2.0 * PI;
                let radians_per_sec = two_pi / 60.0;
//                let radians_per_hour = two_pi / 12.0;

                self.hour_angle = (((self.hour * 60.0 + self.minute) / 60.0) / 12.0) * two_pi;
                self.minute_angle = self.minute * radians_per_sec;
                self.second_angle = self.second * radians_per_sec;

                false
            },
        }
    }
}

impl Viewable<Clock> for Clock {
    fn view(&self) -> Node<Self> {
        let second_hand_props = HandProperties {
            length: self.dial_radius * 0.9,
            width: 1.0,
            theta: self.second_angle,
        };
        let minute_hand_props = HandProperties {
            length: self.dial_radius * 0.8,
            width: 3.0,
            theta: self.minute_angle,
        };
        let hour_hand_props = HandProperties {
            length: self.dial_radius * 0.6,
            width: 5.0,
            theta: self.hour_angle,
        };

        let silver = Color::RGB(196.0 / 255.0,199.0 / 255.0,206.0 / 255.0);
        let darksilver = Color::RGB(148.0 / 255.0, 152.0 / 255.0, 161.0 / 255.0);
        let darkgray = Color::RGB(169.0 / 255.0, 169.0 / 255.0, 169.0 / 255.0);
        let boss_rad = 6.0_f32;

        egml! {
            <group translate = Some((self.dial_center.0, self.dial_center.1).into()), >
                // Dial
                <circle cx = 0.0, cy = 0.0, r = self.dial_radius,
                    stroke = Some((silver, 3.0).into()),
                    fill = Some(Color::RGB(0.2, 0.0, 0.8).into()), />

                // Second hand
                <Hand: with second_hand_props,
                    modifier = |this, clock_model: Clock| {
                        let hand_theta = this.model::<Hand>().theta;
                        if (hand_theta - clock_model.second_angle).abs() > 0.00001 {
                            this.send::<Hand>(HandMsg::ChangeTheta(clock_model.second_angle));
                        }
                    }, />

                // Minute hand
                <Hand: with minute_hand_props,
                    modifier = |this, clock_model: Clock| {
                        let hand_theta = this.model::<Hand>().theta;
                        if (hand_theta - clock_model.minute_angle).abs() > 0.00001 {
                            this.send::<Hand>(HandMsg::ChangeTheta(clock_model.minute_angle));
                        }
                    }, />

                // Hour hand
                <Hand: with hour_hand_props,
                    modifier = |this, clock_model: Clock| {
                        let hand_theta = this.model::<Hand>().theta;
                        if (hand_theta - clock_model.hour_angle).abs() > 0.00001 {
                            this.send::<Hand>(HandMsg::ChangeTheta(clock_model.hour_angle));
                        }
                    }, />

                // Boss
                <circle cx = 0.0, cy = 0.0, r = boss_rad,
                    stroke = Some(darkgray.into()),
                    fill = Some(Gradient::Radial {
                        center: (0.0, 0.0),
                        inner_radius: 0.0,
                        outer_radius: boss_rad,
                        start_color: silver,
                        end_color: darksilver,
                    }.into()), />
            </group>
        }
    }
}

impl Clock {
    fn size_recalc(&mut self, width: i32, height: i32) -> bool {
        let clock_size = width.min(height) - 2;
        if self.clock_size != clock_size {
            self.clock_size = clock_size;
            self.dial_radius = (self.clock_size as f64 / 2.0) as f32;
            self.dial_center = ((width as f64 / 2.0) as f32, (height as f64 / 2.0) as f32);
            true
        } else {
            false
        }
    }
}

struct Hand {
    props: HandProperties,
    theta: f32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
struct HandProperties {
    length: f32,
    width: f32,
    theta: f32,
}

enum HandMsg {
    ChangeTheta(f32),
}

impl ModelComponent for Hand {
    type Message = HandMsg;
    type Properties = HandProperties;

    fn create(props: &<Self as ModelComponent>::Properties) -> Self {
        Hand {
            props: *props,
            theta: props.theta,
        }
    }

    fn update(&mut self, msg: <Self as ModelComponent>::Message) -> bool {
        match msg {
            HandMsg::ChangeTheta(theta) => {
                self.theta = theta;
                false
            }
        }
    }
}

impl Viewable<Hand> for Hand {
    fn view(&self) -> Node<Hand> {
        egml! {
            <path cmd = vec![Move([0.0, 0.0]), Line([0.0, -self.props.length]), Close],
                fill = Some(Color::White.into()), stroke = Some((Color::White, self.props.width).into()),
                transform = Some(Transform::new().with_rotation(self.theta)),
                modifier = |this, model: Hand| { this.transform.as_mut().map(|t| t.rotate(model.theta)); }, />
        }
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let mut mouse_controller = MouseInput::new();
    let window = glutin::WindowBuilder::new()
        .with_title("ExGUI clock")
        .with_dimensions(INIT_WINDOW_SIZE.0, INIT_WINDOW_SIZE.1);
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

    let mut clock = Comp::new::<Clock>(());
    clock.resolve(None);

    let mut render = Renderer::new();

    let mut prev_second = -1.0;
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
        clock.send::<Clock>(Msg::ResizeWindow(width, height));

        {
            clock.send::<Clock>(Msg::Tick);
            let second = clock.model::<Clock>().second;
            if prev_second != second {
                clock.modify(None);
                prev_second = second;
            }
        }

        render.width = width as f32;
        render.height = height as f32;
        render.device_pixel_ratio = gl_window.hidpi_factor();
        render.render(clock.view_node::<Clock>());

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
                        mouse_controller.left_pressed_comp(&mut clock);
                    },
                    _ => (),
                }
            }
            _ => (),
        });
    }
}
