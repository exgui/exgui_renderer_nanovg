use exgui_renderer_nanovg::NanovgRenderer;
use exgui_controller_glutin::{App, AppState, glutin};
use exgui::{egml, Component, Viewable, ChangeView, Node, Comp, PathCommand::*, Color, Stroke, LineJoin};

struct SmileModel {
    normal_face: bool,
}

#[derive(Clone)]
pub enum Msg {
    ToggleFace,
    Nope,
}

impl Component for SmileModel {
    type Message = Msg;
    type Properties = ();

    fn create(_props: &Self::Properties) -> Self {
        SmileModel {
            normal_face: true,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            Msg::ToggleFace => {
                self.normal_face = !self.normal_face;
                ChangeView::Rebuild
            },
            Msg::Nope => ChangeView::None,
        }
    }
}

impl Viewable<SmileModel> for SmileModel {
    fn view(&self) -> Node<Self> {
        egml! {
            <group translate = (50, 50), >
                <rect x = 0, y = 0, width = 300, height = 300,
                        fill = None, stroke = (Color::Black, 2, 0.5), >
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
                </rect>
            </group>
        }
    }
}

impl SmileModel {
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

fn main() {
    let mut app = App::new(
        glutin::WindowBuilder::new()
            .with_title("ExGUI smile")
            .with_dimensions(480, 480),
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(8)
            .with_srgb(true),
        NanovgRenderer::default()
    ).unwrap();

    app.init().unwrap();

    let mut comp = Comp::new::<SmileModel>(());
    comp.resolve(None);

    app.run_proc(&mut comp, |app, _| {
        let (dims, hdpi) = (app.dimensions(), app.window().hidpi_factor());
        app.renderer_mut().set_dimensions(dims, hdpi);
        AppState::Continue
    }).unwrap();
}
