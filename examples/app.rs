use exgui_renderer_nanovg::Renderer;
use exgui_controller_glutin::{App, AppState, glutin};
use exgui::{egml, Component, Viewable, ChangeView, Node, Comp, Color};

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
}

impl Viewable<Model> for Model {
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
            .with_srgb(true)
    ).unwrap();

    app.init().unwrap();

    let mut comp = Comp::new::<Model>(());
    comp.resolve(None);
    let mut render = Renderer::new();

    app.run_proc(&mut comp, |app, comp| {
        let (width, height) = app.dimensions();
        render.width = width as f32;
        render.height = height as f32;
        render.device_pixel_ratio = app.window().hidpi_factor();
        render.render(comp.view_node_mut::<Model>());

        AppState::Continue
    }).unwrap();
}
