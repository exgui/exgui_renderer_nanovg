extern crate nanovg;
extern crate exgui;

use std::path::Path;
use nanovg::{
    Context, ContextBuilder, Font as NanovgFont, Frame, Color as NanovgColor, Gradient as NanovgGradient,
    Paint as NanovgPaint, StrokeOptions, PathOptions, TextOptions, Alignment,
    LineCap as NanovgLineCap, LineJoin as NanovgLineJoin, Transform as NanovgTransform,
};
use exgui::{
    Real, Node, Component, Drawable, Shape, Paint, Color, Gradient, Stroke,
    Text, AlignHor, AlignVer, Transform, LineCap, LineJoin
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

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min_x: Real,
    pub min_y: Real,
    pub max_x: Real,
    pub max_y: Real,
}

impl BoundingBox {
    pub fn width(&self) -> Real {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> Real {
        self.max_y - self.min_y
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

    pub fn render<M: Component>(&self, node: &mut Node<M>) {
        self.context.frame(
            (self.width, self.height),
            self.device_pixel_ratio,
            move |frame| {
                let bound = BoundingBox {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: self.width,
                    max_y: self.height,
                };
                Self::render_recalc(&frame, node as &mut dyn Drawable, bound, None);
                Self::render_draw(&frame, node as &dyn Drawable, None);
            }
        );
    }

    pub fn render_recalc(frame: &Frame,
                         draw: &mut dyn Drawable,
                         parent_bound: BoundingBox,
                         text: Option<&Text>) -> BoundingBox
    {
        let mut bound = parent_bound;

        if let Some(shape) = draw.shape_mut() {
            match shape {
                Shape::Rect(ref mut r) => {
                    if r.x.set_by_pct(parent_bound.width()) {
                        r.x.0 += parent_bound.min_x;
                    }
                    if r.y.set_by_pct(parent_bound.height()) {
                        r.y.0 += parent_bound.min_y;
                    }
                    r.width.set_by_pct(parent_bound.width());
                    r.height.set_by_pct(parent_bound.height());

                    bound = BoundingBox {
                        min_x: r.x.val(),
                        min_y: r.y.val(),
                        max_x: r.x.val() + r.width.val(),
                        max_y: r.y.val() + r.height.val(),
                    };
                },
                Shape::Circle(ref mut c) => {
                    if c.cx.set_by_pct(parent_bound.width()) {
                        c.cx.0 += parent_bound.min_x;
                    }
                    if c.cy.set_by_pct(parent_bound.height()) {
                        c.cy.0 += parent_bound.min_y;
                    }
                    c.r.set_by_pct(parent_bound.width().min(parent_bound.height()));

                    let (cx, cy, r) = (c.cx.val(), c.cy.val(), c.r.val());
                    bound = BoundingBox {
                        min_x: cx - r,
                        min_y: cy - r,
                        max_x: cx + r,
                        max_y: cy + r,
                    };
                },
                Shape::Text(ref mut t) => {
                    if t.x.set_by_pct(parent_bound.width()) {
                        t.x.0 += parent_bound.min_x;
                    }
                    if t.y.set_by_pct(parent_bound.height()) {
                        t.y.0 += parent_bound.min_y;
                    }

                    let text = t.clone();
                    return Self::calc_inner_bound(frame, draw, bound, Some(&text));
                },
                Shape::Word(ref w) => {
                    if let Some(text) = text {
                        let nanovg_font = NanovgFont::find(frame.context(), text.font_name.as_str())
                            .expect(&format!("Font '{}' not found", text.font_name));

                        let text_options = if let AlignHor::Center = text.align.0 {
                            // Fix nanovg text_bounds bug for centered text
                            let mut text = text.clone();
                            text.align.0 = AlignHor::Left;
                            Self::text_options(&text)
                        } else {
                            Self::text_options(text)
                        };

                        let mut text_bounds = frame.text_box_bounds(
                            nanovg_font,
                            (text.x.val(), text.y.val()),
                            w,
                            text_options,
                        );

                        // Fix nanovg text_bounds bug for centered text
                        if let AlignHor::Center = text.align.0 {
                            let half_width = (text_bounds.max_x - text_bounds.min_x) / 2.0;
                            text_bounds.min_x -= half_width;
                            text_bounds.max_x -= half_width;
                        }

                        bound = BoundingBox {
                            min_x: text_bounds.min_x,
                            min_y: text_bounds.min_y,
                            max_x: text_bounds.max_x,
                            max_y: text_bounds.max_y,
                        };
                    }
                },
                _ => (),
            }
        }

        let inner_bound = Self::calc_inner_bound(frame, draw, bound, text);

        if let Some(shape) = draw.shape_mut() {
            match shape {
                Shape::Rect(ref mut r) => {
                    r.x.set_by_auto(inner_bound.min_x);
                    r.y.set_by_auto(inner_bound.min_y);
                    r.width.set_by_auto(inner_bound.width());
                    r.height.set_by_auto(inner_bound.height());

                    bound = BoundingBox {
                        min_x: r.x.val(),
                        min_y: r.y.val(),
                        max_x: r.x.val() + r.width.val(),
                        max_y: r.y.val() + r.height.val(),
                    };
                },
                Shape::Circle(ref mut c) => {
                    c.cx.set_by_auto(inner_bound.min_x + inner_bound.width() / 2.0);
                    c.cy.set_by_auto(inner_bound.min_y + inner_bound.height() / 2.0);
                    c.r.set_by_auto(inner_bound.width().max(inner_bound.height()) / 2.0);

                    let (cx, cy, r) = (c.cx.val(), c.cy.val(), c.r.val());
                    bound = BoundingBox {
                        min_x: cx - r,
                        min_y: cy - r,
                        max_x: cx + r,
                        max_y: cy + r,
                    };
                },
                _ => (),
            }
        }
        bound
    }

    fn calc_inner_bound(frame: &Frame,
                        draw: &mut dyn Drawable,
                        bound: BoundingBox,
                        text: Option<&Text>) -> BoundingBox
    {
        let mut child_bounds = Vec::new();
        if let Some(childs) = draw.childs_mut() {
            for child in childs {
                child_bounds.push(
                    Self::render_recalc(frame, child, bound, text)
                );
            }
        }

        if child_bounds.is_empty() {
            BoundingBox::default()
        } else {
            let mut inner_bound = child_bounds[0];
            for bound in &child_bounds[1..] {
                if bound.min_x < inner_bound.min_x {
                    inner_bound.min_x = bound.min_x;
                }
                if bound.min_y < inner_bound.min_y {
                    inner_bound.min_y = bound.min_y;
                }
                if bound.max_x > inner_bound.max_x {
                    inner_bound.max_x = bound.max_x;
                }
                if bound.max_y > inner_bound.max_y {
                    inner_bound.max_y = bound.max_y;
                }
            }
            inner_bound
        }
    }

    fn render_draw<'a>(frame: &Frame, draw: &'a dyn Drawable, mut text: Option<&'a Text>) {
        if let Some(shape) = draw.shape() {
            match shape {
                Shape::Rect(ref r) => {
                    frame.path(
                        |path| {
                            path.rect((r.x.val(), r.y.val()), (r.width.val(), r.height.val()));
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
                            path.circle((c.cx.val(), c.cy.val()), c.r.val());
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
                Shape::Text(ref t) => {
                    text = Some(t);
                },
                Shape::Word(ref w) => {
                    if let Some(text) = text {
                        let nanovg_font = NanovgFont::find(frame.context(), text.font_name.as_str())
                            .expect(&format!("Font '{}' not found", text.font_name));
                        let text_options = Self::text_options(text);

                        frame.text(
                            nanovg_font,
                            (text.x.val(), text.y.val()),
                            w,
                            text_options,
                        );
                    }
                },
                Shape::Group(ref _g) => {},
            }
        }
        if let Some(childs) = draw.childs() {
            for child in childs {
                Self::render_draw(frame, child, text);
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

    fn text_options(text: &Text) -> TextOptions {
        let color = ToNanovgPaint::to_nanovg_color(
            text.fill.and_then(|fill| if let Paint::Color(color) = fill.paint {
                Some(color)
            } else {
                None
            }).unwrap_or_default()
        );
        let mut align = Alignment::new();
        align = match text.align.0 {
            AlignHor::Left => align.left(),
            AlignHor::Right => align.right(),
            AlignHor::Center => align.center(),
        };
        align = match text.align.1 {
            AlignVer::Bottom => align.bottom(),
            AlignVer::Middle => align.middle(),
            AlignVer::Baseline => align.baseline(),
            AlignVer::Top => align.top(),
        };

        TextOptions {
            color,
            size: text.font_size.val(),
            align,
            transform: Self::to_nanovg_transform(text.transform.as_ref()),
            ..Default::default()
        }
    }
}
