use exgui_renderer_nanovg::NanovgRenderer;
use exgui_controller_glutin::{App, AppState, glutin};
use exgui::{egml, Component, ChangeView, Node, Comp, Color, AlignHor::*, AlignVer::*};

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

    fn view(&self) -> Node<Self> {
        egml! {
            <rect stroke = (Color::Blue, 2), >
                <circle stroke = (Color::Yellow, 2), > // { paint: Color::Yellow.with_alpha(0.5), width: 2 }
                    <text x = 50, y = 100, font_name = "Roboto", font_size = 24,
                            align = (Left, Baseline), fill = Color::Red, >
                        { "Some text in circle" }
                    </text>
                </circle>
            </rect>
        }
    }
}

fn main() {
    let mut app = App::new(
        glutin::WindowBuilder::new()
            .with_title("ExGUI auto size")
            .with_dimensions(480, 480),
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(8)
            .with_srgb(true),
        NanovgRenderer::default()
    ).unwrap();

    app.init().unwrap();
    app.renderer_mut().load_font("Roboto", "resources/Roboto-Regular.ttf").unwrap();

    let mut comp = Comp::new::<Model>(());
    comp.resolve(None);

    app.run_proc(&mut comp, |app, _| {
        let (dims, hdpi) = (app.dimensions(), app.window().hidpi_factor());
        app.renderer_mut().set_dimensions(dims, hdpi);
        AppState::Continue
    }).unwrap();
}