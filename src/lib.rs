extern crate nanovg;
extern crate exgui;

use std::path::Path;
use nanovg::{
    Context, ContextBuilder, Font as NanovgFont, Frame, Color as NanovgColor, Gradient as NanovgGradient,
    Paint as NanovgPaint, StrokeOptions, PathOptions, TextOptions, Alignment,
    LineCap as NanovgLineCap, LineJoin as NanovgLineJoin, Transform as NanovgTransform,
};
use exgui::{
    Node, ModelComponent, Drawable, Shape, Paint, Color, Gradient, Stroke, Font, AlignHor, AlignVer,
    Transform, LineCap, LineJoin
};

struct ToNanovgPaint(Paint);

impl ToNanovgPaint {
    fn to_nanovg_color(color: Color) -> NanovgColor {
        let [r, g, b, a] = color.as_arr();
        NanovgColor::new(r, g, b, a)
    }

    fn to_nanovg_gradient(gradient: Gradient) -> NanovgGradient {
        match gradient {
            Gradient::Linear { start, end, start_color, end_color } =>
                NanovgGradient::Linear {
                    start, end,
                    start_color: Self::to_nanovg_color(start_color),
                    end_color: Self::to_nanovg_color(end_color),
                },
            Gradient::Box { position, size, radius, feather, start_color, end_color } =>
                NanovgGradient::Box {
                    position, size, radius, feather,
                    start_color: Self::to_nanovg_color(start_color),
                    end_color: Self::to_nanovg_color(end_color),
                },
            Gradient::Radial { center, inner_radius, outer_radius, start_color, end_color } =>
                NanovgGradient::Radial {
                    center, inner_radius, outer_radius,
                    start_color: Self::to_nanovg_color(start_color),
                    end_color: Self::to_nanovg_color(end_color),
                },
        }
    }
}

impl NanovgPaint for ToNanovgPaint {
    fn fill(&self, context: &Context) {
        match self.0 {
            Paint::Color(ref color) => Self::to_nanovg_color(*color).fill(context),
            Paint::Gradient(ref gradient) => Self::to_nanovg_gradient(*gradient).fill(context),
        }
    }

    fn stroke(&self, context: &Context) {
        match self.0 {
            Paint::Color(ref color) => Self::to_nanovg_color(*color).stroke(context),
            Paint::Gradient(ref gradient) => Self::to_nanovg_gradient(*gradient).stroke(context),
        }
    }
}

pub struct Renderer {
    pub context: Context,
    pub width: f32,
    pub height: f32,
    pub device_pixel_ratio: f32,
}

impl Renderer {
    pub fn new() -> Self {
        let context = ContextBuilder::new()
            .stencil_strokes()
            .build()
            .expect("Initialization of NanoVG context failed!");

        let renderer = Self::new_with_context(context);
        renderer
    }

    pub fn new_with_context(context: Context) -> Self {
        Renderer {
            context,
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

    pub fn load_font<S: Into<String>, P: AsRef<Path>>(&mut self, name: S, path: P) {
        let name = name.into();
        let display_path = path.as_ref().display();
        NanovgFont::from_file(&self.context, name.as_str(), path.as_ref())
            .expect(&format!("Failed to load font '{}'", display_path));
    }

    pub fn render<MC: ModelComponent>(&self, node: &Node<MC>) {
        self.context.frame(
            (self.width, self.height),
            self.device_pixel_ratio,
            move |frame| {
                let mut font_stack = Vec::new();
                Self::render_draw(&frame, node as &dyn Drawable, &mut font_stack);
            }
        );
    }

    fn render_draw<'a>(frame: &Frame, draw: &'a dyn Drawable, font_stack: &mut Vec<&'a Font>) {
        if let Some(shape) = draw.shape() {
            match shape {
                Shape::Rect(ref r) => {
                    frame.path(
                        |path| {
                            path.rect((r.x, r.y), (r.width, r.height));
                            if let Some(fill) = r.fill {
                                path.fill(ToNanovgPaint(fill.paint), Default::default());
                            };
                            if let Some(stroke) = r.stroke {
                                path.stroke(
                                    ToNanovgPaint(stroke.paint),
                                    Self::stroke_option(&stroke)
                                );
                            }
                        },
                        Self::path_options(r.transform.as_ref()),
                    );
                },
                Shape::Circle(ref c) => {
                    frame.path(
                        |path| {
                            path.circle((c.cx, c.cy), c.r);
                            if let Some(fill) = c.fill {
                                path.fill(ToNanovgPaint(fill.paint), Default::default());
                            };
                            if let Some(stroke) = c.stroke {
                                path.stroke(
                                    ToNanovgPaint(stroke.paint),
                                    Self::stroke_option(&stroke)
                                );
                            }
                        },
                        Self::path_options(c.transform.as_ref()),
                    );
                },
                Shape::Path(ref p) => {
                    frame.path(
                        |path| {
                            use exgui::PathCommand::*;

                            let mut last_xy = [0.0_f32, 0.0];
                            let mut bez_ctrls = [(0.0_f32, 0.0), (0.0_f32, 0.0)];

                            for cmd in p.cmd.iter() {
                                match cmd {
                                    Move(ref xy) => {
                                        last_xy = *xy;
                                        path.move_to((last_xy[0], last_xy[1]));
                                    },
                                    MoveRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        path.move_to((last_xy[0], last_xy[1]));
                                    },
                                    Line(ref xy) => {
                                        last_xy = *xy;
                                        path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineAlonX(ref x) => {
                                        last_xy[0] = *x;
                                        path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineAlonXRel(ref x) => {
                                        last_xy[0] += *x;
                                        path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineAlonY(ref y) => {
                                        last_xy[1] = *y;
                                        path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineAlonYRel(ref y) => {
                                        last_xy[1] += *y;
                                        path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    Close => path.close(),
                                    BezCtrl(ref xy) => {
                                        bez_ctrls = [bez_ctrls[1], (xy[0], xy[1])];
                                    },
                                    BezCtrlRel(ref xy) => {
                                        bez_ctrls = [bez_ctrls[1], (last_xy[0] + xy[0], last_xy[1] + xy[1])];
                                    },
                                    QuadBezTo(ref xy) => {
                                        last_xy = *xy;
                                        path.quad_bezier_to((last_xy[0], last_xy[1]), bez_ctrls[1]);
                                    },
                                    QuadBezToRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        path.quad_bezier_to((last_xy[0], last_xy[1]), bez_ctrls[1]);
                                    },
                                    CubBezTo(ref xy) => {
                                        last_xy = *xy;
                                        path.cubic_bezier_to((last_xy[0], last_xy[1]), bez_ctrls[0], bez_ctrls[1]);
                                    },
                                    CubBezToRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        path.cubic_bezier_to((last_xy[0], last_xy[1]), bez_ctrls[0], bez_ctrls[1]);
                                    },
                                    _ => panic!("Not impl rendering cmd {:?}", cmd), // TODO: need refl impl
                                }
                            }
                            if let Some(fill) = p.fill {
                                path.fill(ToNanovgPaint(fill.paint), Default::default());
                            };
                            if let Some(stroke) = p.stroke {
                                path.stroke(
                                    ToNanovgPaint(stroke.paint),
                                    Self::stroke_option(&stroke)
                                );
                            }
                        },
                        Self::path_options(p.transform.as_ref()),
                    );
                },
                Shape::Group(ref _g) => {},
                Shape::Font(ref f) => {
                    font_stack.push(f);
                    if let Some(childs) = draw.childs() {
                        for child in childs {
                            Self::render_draw(frame, child, font_stack);
                        }
                    }
                    font_stack.pop();
                    return;
                },
                Shape::Text(ref t) => {
                    if let Some(f) = font_stack.last() {
                        let nanovg_font = NanovgFont::find(frame.context(), f.name.as_str())
                            .expect(&format!("Font '{}' not found", f.name));
                        let color = ToNanovgPaint::to_nanovg_color(
                            f.fill.and_then(|fill| if let Paint::Color(color) = fill.paint {
                                Some(color)
                            } else {
                                None
                            }).unwrap_or_default()
                        );
                        let mut align = Alignment::new();
                        align = match f.align.0 {
                            AlignHor::Left => align.left(),
                            AlignHor::Right => align.right(),
                            AlignHor::Center => align.center(),
                        };
                        align = match f.align.1 {
                            AlignVer::Bottom => align.bottom(),
                            AlignVer::Middle => align.middle(),
                            AlignVer::Baseline => align.baseline(),
                            AlignVer::Top => align.top(),
                        };

                        frame.text(
                            nanovg_font,
                            (f.x, f.y),
                            t,
                            TextOptions {
                                color,
                                size: f.size,
                                align,
                                transform: Self::to_nanovg_transform(f.transform.as_ref()),
                                ..Default::default()
                            },
                        );
                    }
                },
            }
        }
        if let Some(childs) = draw.childs() {
            for child in childs {
                Self::render_draw(frame, child, font_stack);
            }
        }
    }

    fn to_nanovg_transform(transform: Option<&Transform>) -> Option<NanovgTransform> {
        transform.map(|transform| {
            let mut nanovg_transform = NanovgTransform::new();
            if transform.absolute {
                nanovg_transform.absolute();
            }
            nanovg_transform.matrix = transform.matrix;
            nanovg_transform
        })
    }

    fn path_options(transform: Option<&Transform>) -> PathOptions {
        if let Some(transform) = transform {
            let mut nanovg_transform = NanovgTransform::new();
            if transform.absolute {
                nanovg_transform.absolute();
            }
            nanovg_transform.matrix = transform.matrix;
            PathOptions {
                transform: Some(nanovg_transform),
                ..Default::default()
            }
        } else {
            PathOptions::default()
        }
    }

    fn stroke_option(stroke: &Stroke) -> StrokeOptions {
        let line_cap = match stroke.line_cap {
            LineCap::Butt => NanovgLineCap::Butt,
            LineCap::Round => NanovgLineCap::Round,
            LineCap::Square => NanovgLineCap::Square,
        };
        let line_join = match stroke.line_join {
            LineJoin::Miter => NanovgLineJoin::Miter,
            LineJoin::Round => NanovgLineJoin::Round,
            LineJoin::Bevel => NanovgLineJoin::Bevel,
        };
        StrokeOptions {
            width: stroke.width,
            line_cap,
            line_join,
            miter_limit: stroke.miter_limit,
            ..Default::default()
        }
    }
}
