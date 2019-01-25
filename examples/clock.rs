use std::f32::consts::PI;
use exgui_renderer_nanovg::NanovgRenderer;
use exgui_controller_glutin::{App, AppState, glutin};
use exgui::{
    egml, Component, Viewable, ChangeView, Node, Comp, Color, Gradient, AlignHor::*, AlignVer::*,
    PathCommand::*, Transform
};
use chrono::{DateTime, Local, Timelike, Datelike};

const INIT_WINDOW_SIZE: (u32, u32) = (480, 480);
const TWO_PI: f32 = 2.0 * PI;

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
    day_changed: bool,

    hour_angle: f32,
    minute_angle: f32,
    second_angle: f32,
}

#[derive(Clone)]
pub enum Msg {
    ResizeWindow((u32, u32)),
    Tick,
}

impl Component for Clock {
    type Message = Msg;
    type Properties = ();

    fn create(_props: &Self::Properties) -> Self {
        let (width, height) = INIT_WINDOW_SIZE;
        let mut clock = Clock::default();
        clock.size_recalc(width, height);
        clock
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            Msg::ResizeWindow((w, h)) => {
                self.size_recalc(w, h)
            },
            Msg::Tick => {
                let dt: DateTime<Local> = Local::now(); // e.g. `2018-11-28T21:45:59.324310806+09:00`

                let prev_second = self.second;
                self.second = f64::from(dt.second()) as f32;

                if (self.second - prev_second).abs() >= 1. {
                    let hour = dt.hour();

                    self.am = hour < 12;
                    self.hour = f64::from(hour % 12) as f32;
                    self.minute = f64::from(dt.minute()) as f32;


                    self.year = dt.year();
                    self.month = dt.month();

                    let day = dt.day();
                    if self.day == day {
                        self.day_changed = false;
                    } else {
                        self.day = day;
                        self.day_changed = true;
                    }

                    let radians_per_sec = TWO_PI / 60.0;

                    self.hour_angle = (((self.hour * 60.0 + self.minute) / 60.0) / 12.0) * TWO_PI;
                    self.minute_angle = self.minute * radians_per_sec;
                    self.second_angle = self.second * radians_per_sec;

                    ChangeView::Modify
                } else {
                    ChangeView::None
                }
            },
        }
    }
}

impl Viewable<Clock> for Clock {
    fn view(&self) -> Node<Self> {
        let second_hand_len = self.dial_radius * 0.9;
        let second_hand_props = HandProperties {
            length: second_hand_len,
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
            <group translate = (self.dial_center.0, self.dial_center.1), >
                // Dial
                <circle cx = 0, cy = 0, r = self.dial_radius,
                    stroke = (silver, 3),
                    fill = Color::RGB(0.2, 0.0, 0.8), />

                // Hour/minute markers
                { for (1..=12).map(|n| self.view_num(n, second_hand_len, 24.0)) }

                // Tick markers
                { for (1..=60)
                        .filter(|m| m % 5 != 0)
                        .map(|m| self.view_tick(m as f32, 3.0, 1.0)) }

                // Date-string
                <text x = 0, y = self.dial_radius * 0.7, font_name = "Roboto", font_size = 24,
                        align = (Center, Baseline), fill = silver, >
                    { format!("{:4}-{:02}-{:02}", self.year, self.month, self.day) }
                        .word.modifier = |this, clock_model: Clock| {
                            if clock_model.day_changed {
                                this.content = format!(
                                    "{:4}-{:02}-{:02}", clock_model.year, clock_model.month, clock_model.day
                                );
                            }
                        },
                </text>

                // Second hand
                <Hand: with second_hand_props,
                    modifier = |this, clock_model: Clock| {
                        let hand_theta = this.model::<Hand>().theta;
                        if (hand_theta - clock_model.second_angle).abs() > 0.00001 {
                            this.send_self(HandMsg::ChangeTheta(clock_model.second_angle));
                        }
                    }, />

                // Minute hand
                <Hand: with minute_hand_props,
                    modifier = |this, clock_model: Clock| {
                        let hand_theta = this.model::<Hand>().theta;
                        if (hand_theta - clock_model.minute_angle).abs() > 0.00001 {
                            this.send_self(HandMsg::ChangeTheta(clock_model.minute_angle));
                        }
                    }, />

                // Hour hand
                <Hand: with hour_hand_props,
                    modifier = |this, clock_model: Clock| {
                        let hand_theta = this.model::<Hand>().theta;
                        if (hand_theta - clock_model.hour_angle).abs() > 0.00001 {
                            this.send_self(HandMsg::ChangeTheta(clock_model.hour_angle));
                        }
                    }, />

                // Boss
                <circle cx = 0, cy = 0, r = boss_rad,
                    stroke = darkgray,
                    fill = Gradient::Radial {
                        center: (0.0, 0.0),
                        inner_radius: 0.0,
                        outer_radius: boss_rad,
                        start_color: silver,
                        end_color: darksilver,
                    }, />
            </group>
        }
    }
}

impl Clock {
    fn size_recalc(&mut self, width: u32, height: u32) -> ChangeView {
        let clock_size = width.min(height) as i32 - 2;
        if self.clock_size != clock_size {
            self.clock_size = clock_size;
            self.dial_radius = (self.clock_size as f64 / 2.0) as f32;
            self.dial_center = ((width as f64 / 2.0) as f32, (height as f64 / 2.0) as f32);
            ChangeView::Rebuild
        } else {
            ChangeView::None
        }
    }

    fn view_num(&self, n: i32, len: f32, font_size: f32) -> Node<Clock> {
        let radians_per_hour = TWO_PI / 12.0;
        let x = len * (n as f32 * radians_per_hour).sin();
        let y = - len * (n as f32 * radians_per_hour).cos();
        let silver = Color::RGB(196.0 / 255.0,199.0 / 255.0,206.0 / 255.0);

        egml! {
            <text x = x, y = y, font_name = "Roboto", font_size = font_size,
                    align = (Center, Middle), fill = silver, >
                { format!("{}", n) }
            </text>
        }
    }

    fn view_tick(&self, m: f32, len: f32, width: f32) -> Node<Clock> {
        let radians_per_sec = TWO_PI / 60.0;
        let ticks_radius = self.dial_radius * 0.925;
        egml! {
            <path cmd = vec![Move([0.0, -ticks_radius]), Line([0.0, -ticks_radius - len]), Close],
                fill = Color::White, stroke = (Color::White, width),
                transform = Transform::new().with_rotation(m * radians_per_sec), />
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

#[derive(Clone)]
enum HandMsg {
    ChangeTheta(f32),
}

impl Component for Hand {
    type Message = HandMsg;
    type Properties = HandProperties;

    fn create(props: &Self::Properties) -> Self {
        Hand {
            props: *props,
            theta: props.theta,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            HandMsg::ChangeTheta(theta) => {
                self.theta = theta;
                ChangeView::Modify
            }
        }
    }
}

impl Viewable<Hand> for Hand {
    fn view(&self) -> Node<Hand> {
        egml! {
            <path cmd = vec![Move([0.0, 0.0]), Line([0.0, -self.props.length]), Close],
                fill = Color::White, stroke = (Color::White, self.props.width),
                transform = Transform::new().with_rotation(self.theta),
                modifier = |this, model: Hand| { this.transform.as_mut().map(|t| t.rotate(model.theta)); }, />
        }
    }
}

fn main() {
    let mut app = App::new(
        glutin::WindowBuilder::new()
            .with_title("ExGUI clock")
            .with_dimensions(INIT_WINDOW_SIZE.0, INIT_WINDOW_SIZE.1),
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(8)
            .with_srgb(true),
        NanovgRenderer::default()
    ).unwrap();

    app.init().unwrap();
    app.renderer_mut().load_font("Roboto", "resources/Roboto-Regular.ttf").unwrap();

    let mut comp = Comp::new::<Clock>(());
    comp.resolve(None);

    app.run_proc(&mut comp, |app, clock| {
        clock.send_self(Msg::ResizeWindow(app.dimensions()));
        clock.send_self(Msg::Tick);

        let (dims, hdpi) = (app.dimensions(), app.window().hidpi_factor());
        app.renderer_mut().set_dimensions(dims, hdpi);
        AppState::Continue
    }).unwrap();
}