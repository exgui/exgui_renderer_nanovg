use exgui_renderer_nanovg::NanovgRenderer;
use exgui_controller_glutin::{App, AppState, glutin};
use exgui::{egml, Component, Viewable, ChangeView, Node, Comp, Color};

#[derive(Debug)]
struct Ball {
    normal: bool,
    dir: i32,
    cy: f32,
}

#[derive(Clone)]
pub enum Msg {
    Toggle,
    PosUpdate,
}

impl Component for Ball {
    type Message = Msg;
    type Properties = ();

    fn create(_props: &Self::Properties) -> Self {
        Ball {
            normal: true,
            dir: 1,
            cy: 50.0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            Msg::Toggle => {
                self.normal = !self.normal;
                ChangeView::Rebuild
            },
            Msg::PosUpdate => {
                self.dir = if self.cy <= 20.0 && self.dir < 0 {
                    1
                } else if self.cy >= 280.0 && self.dir > 0 {
                    -1
                } else {
                    self.dir
                };
                self.cy += (self.dir * 2) as f32;
                ChangeView::Modify
            },
        }
    }
}

impl Viewable<Ball> for Ball {
    fn view(&self) -> Node<Self> {
        egml! {
            <group translate = (50, 50), >
                <rect x = 0, y = 0, width = 300, height = 300,
                        fill = None, stroke = (Color::Black, 2, 0.5), >
                    <circle cx = 150, cy = self.cy, r = 20,
                            fill = if self.normal { Color::Blue } else { Color::Red },
                            modifier = |this, model: Ball| {
                                this.cy = model.cy.into();
                            },
                            onclick = |_| Msg::Toggle, />
                </rect>
            </group>
        }
    }
}

fn main() {
    let mut app = App::new(
        glutin::WindowBuilder::new()
            .with_title("ExGUI ball")
            .with_dimensions(480, 480),
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4)
            .with_srgb(true),
        NanovgRenderer::default()
    ).unwrap();

    app.init().unwrap();

    let mut comp = Comp::new::<Ball>(());
    comp.resolve(None);

    app.run_proc(&mut comp, |app, comp| {
        comp.send_self(Msg::PosUpdate);

        let (dims, hdpi) = (app.dimensions(), app.window().hidpi_factor());
        app.renderer_mut().set_dimensions(dims, hdpi);

        AppState::Continue
    }).unwrap();
}