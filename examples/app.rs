use exgui_renderer_nanovg::NanovgRenderer;
use exgui_controller_glutin::{App, AppState, glutin};
use exgui::{egml, Component, ChangeView, Node, Comp, Color};

struct Model;

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: &Self::Properties) -> Self {
        Model
    }

    fn update(&mut self, _msg: Self::Message) -> ChangeView {
        unimplemented!()
    }

    fn view(&self) -> Node<Self> {
        let rect_fill = Color::Yellow;
        egml! {
            <rect x = 40, y = 40, width = 50, height = 80,
                    fill = rect_fill, stroke = (Color::Red, 2), >
                <group>
                    <circle cx = 120, cy = 120, r = 40, fill = Color::White, />
                </group>
            </rect>
        }
    }
}

fn main() {
    let mut app = App::new(
        glutin::WindowBuilder::new()
            .with_title("ExGUI app")
            .with_dimensions(480, 480),
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4)
            .with_srgb(true),
        NanovgRenderer::default()
    ).unwrap();

    app.init().unwrap();

    let mut comp = Comp::new::<Model>(());
    comp.resolve(None);

    app.run_proc(&mut comp, |app, _| {
        let (dims, hdpi) = (app.dimensions(), app.window().hidpi_factor());
        app.renderer_mut().set_dimensions(dims, hdpi);
        AppState::Continue
    }).unwrap();
}