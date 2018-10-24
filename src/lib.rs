extern crate nanovg;
extern crate exgui;

use std::collections::HashMap;
use std::path::Path;
use nanovg::{Context, ContextBuilder, Font, Frame, Color as NanovgColor, StrokeOptions, PathOptions};
use exgui::{Node, ModelComponent, Shape, Color};

pub trait AsNanovgColor {
    fn as_nanovg_color(&self) -> NanovgColor;
}

impl AsNanovgColor for Color {
    fn as_nanovg_color(&self) -> NanovgColor {
        let [r, g, b, a] = self.as_arr();
        NanovgColor::new(r, g, b, a)
    }
}

pub struct Renderer<'a> {
    pub context: Context,
    pub fonts: HashMap<String, Font<'a>>,
    pub width: f32,
    pub height: f32,
    pub device_pixel_ratio: f32,
}

impl<'a> Renderer<'a> {
    pub fn new() -> Self {
        let context = ContextBuilder::new()
            .stencil_strokes()
            .build()
            .expect("Initialization of NanoVG context failed!");

        let renderer = Self::new_with_context(context);
        //renderer.load_font("Roboto", "resources/Roboto-Regular.ttf");
        renderer
    }

    pub fn new_with_context(context: Context) -> Self {
        Renderer {
            context,
            fonts: HashMap::new(),
            width: 0.0,
            height: 0.0,
            device_pixel_ratio: 0.0,
        }
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn with_device_pixel_ratio(mut self, device_pixel_ratio: f32) -> Self {
        self.device_pixel_ratio = device_pixel_ratio;
        self
    }

    pub fn load_font<S: Into<String>, P: AsRef<Path>>(&'a mut self, name: S, path: P) {
        let name = name.into();
        let display_path = path.as_ref().display();
        let font = Font::from_file(&self.context, name.as_str(), path.as_ref())
            .expect(&format!("Failed to load font '{}'", display_path));
        self.fonts.insert(name, font);
    }

    pub fn render<MC: ModelComponent>(&self, node: &Node<MC>) {
        self.context.frame(
            (self.width, self.height),
            self.device_pixel_ratio,
            Self::render_handler(node)
        );
    }

    pub fn render_handler<MC: ModelComponent>(node: &'a Node<MC>) -> impl FnOnce(Frame<'a>) {
        move |frame| {
            Self::render_node(&frame, node);
        }
    }

    fn render_node<MC: ModelComponent>(frame: &Frame<'a>, node: &Node<MC>) {
        match node {
            Node::Unit(ref unit) => {
                match unit.shape {
                    Shape::Rect(ref r) => {
                        frame.path(
                            |path| {
                                path.rect((r.x, r.y), (r.width, r.height));

                                let fill_color = if let Some(ref fill) = r.fill {
                                    fill.color
                                } else {
                                    Color::White
                                };
                                path.fill(fill_color.as_nanovg_color(), Default::default());

                                if let Some(stroke) = r.stroke {
                                    path.stroke(
                                        stroke.color.as_nanovg_color(),
                                        StrokeOptions {
                                            width: stroke.width,
                                            ..Default::default()
                                        }
                                    );
                                }
                            },
                            PathOptions::default(),
                        );
                    },
                    Shape::Circle(ref c) => {
                        frame.path(
                            |path| {
                                path.circle((c.cx, c.cy), c.r);

                                let fill_color = if let Some(ref fill) = c.fill {
                                    fill.color
                                } else {
                                    Color::White
                                };
                                path.fill(fill_color.as_nanovg_color(), Default::default());

                                if let Some(stroke) = c.stroke {
                                    path.stroke(
                                        stroke.color.as_nanovg_color(),
                                        StrokeOptions {
                                            width: stroke.width,
                                            ..Default::default()
                                        }
                                    );
                                }
                            },
                            PathOptions::default(),
                        );
                    },
                    Shape::Group(ref _g) => (),
                }
                for child in unit.childs.iter() {
                    Self::render_node(frame, child);
                }
            }
        }
    }
}
