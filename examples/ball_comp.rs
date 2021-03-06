use exgui_renderer_nanovg::NanovgRenderer;
use exgui_controller_glutin::{App, AppState, glutin};
use exgui::{
    egml, Component, Shapeable, ChangeView, Node, Comp, Color,
    Pct, Real, SystemMessage
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
    penult_pos_px: Option<Real>,
    radius: Real,
}

#[derive(Copy, Clone)]
enum BallMsg {
    Toggle,
    PosUpdate,
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
            penult_pos_px: None,
            radius: 20.0,
        }
    }

    #[allow(irrefutable_let_patterns)]
    fn system_update(&mut self, msg: SystemMessage) -> Option<Self::Message> {
        if let SystemMessage::FrameChange = msg {
            Some(BallMsg::PosUpdate)
        } else {
            None
        }
    }

    fn update_with_view(&mut self, view: Option<&Node<Self>>, msg: Self::Message) -> ChangeView {
        match msg {
            BallMsg::Toggle => {
                self.normal = !self.normal;
                ChangeView::None
            },
            BallMsg::PosUpdate => {
                if let Some(circle) = view
                    .and_then(|view| view.prim())
                    .and_then(|prim| prim.circle())
                {
                    let last_pos = match self.orientation {
                        BallOrientation::Horizontal => circle.cx.0,
                        BallOrientation::Vertical => circle.cy.0,
                    };
                    let step_pct = 0.5;

                    if let Some(penult_pos_px) = self.penult_pos_px {
                        let radius_pct = self.radius / ((penult_pos_px - last_pos).abs() / step_pct);

                        if self.point_pct <= radius_pct.into() && self.dir < 0 {
                            self.dir = 1;
                        } else if self.point_pct >= (100.0 - radius_pct).into() && self.dir > 0 {
                            self.dir = -1;
                        };
                    }
                    self.penult_pos_px = Some(last_pos);

                    self.point_pct += Pct(self.dir as Real * step_pct);
                    ChangeView::Modify
                } else {
                    ChangeView::None
                }
            },
        }
    }

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
    let mut app = App::new(
        glutin::WindowBuilder::new()
            .with_title("ExGUI ball comp")
            .with_dimensions(480, 480),
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4)
            .with_srgb(true),
        NanovgRenderer::default(),
    ).unwrap();

    app.init().unwrap();

    let mut comp = Comp::new::<Model>(());
    comp.resolve(None);

    app.run_proc(&mut comp, |app, _comp| {
        let (dims, hdpi) = (app.dimensions(), app.window().hidpi_factor());
        app.renderer_mut().set_dimensions(dims, hdpi);

        AppState::Continue
    }).unwrap();
}
