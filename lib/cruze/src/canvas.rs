extern crate lyon;

use lyon::math::{
    point,
    Point,
    vector,
    Vector,
    Angle,
};

use lyon::path::{
    Path,
};

use lyon::tessellation::geometry_builder::{
    BuffersBuilder,
    VertexBuffers,
    VertexConstructor,
};

use lyon::tessellation::{
    FillVertex,
    FillOptions,
    FillTessellator,
    StrokeVertex,
    StrokeOptions,
    StrokeTessellator,
};

use super::font_manager::{
    FontManager,
    GlyphTexData
};

use super::window::Widget;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color {
            r,
            g,
            b,
            a
        }
    }

    pub fn to_vec(&self) -> cgmath::Vector4<f32> {
        cgmath::Vector4::new(self.r, self.g, self.b, self.a)
    }

    pub fn from_rgb(r: f32, g: f32, b: f32) -> Color {
        Color::new(r, g, b, 1.0)
    }

    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color::new(r, g, b, a)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub fn new(width: T, height: T) -> Size<T> {
        Size {
            width,
            height
        }
    }
}

/*
   pub struct Rectangle {
   center: Point2<f32>,
   size: Size<f32>,
   radius: f32,
   pub current_rotation: f32,
   pub current_scale: f32,
   pub current_translate: cgmath::Vector2<f32>,
   rotation_mat: cgmath::Matrix4<f32>,
   translation_mat: cgmath::Matrix4<f32>,
   scale_mat: cgmath::Matrix4<f32>,
   }

   impl Rectangle {
   pub fn new(center: Point2<f32>, size: Size<f32>, radius: f32) -> Rectangle {
   Rectangle {
   center,
   size,
   radius,
   current_scale: 1.0,
   current_rotation: 0.0,
   current_translate: cgmath::Vector2::new(0.0, 0.0),
   rotation_mat: cgmath::Matrix4::identity(),
   translation_mat: cgmath::Matrix4::identity(),
   scale_mat: cgmath::Matrix4::identity(),
   }
   }

   pub fn geometry(&self) ->  (Vec<f32>, Vec<u32>) {
   #[derive(Copy, Clone, Debug)]
   struct MyVertex {
   x: f32,
   y: f32,
   z: f32
   };

   let tl = Point2 {
   x: self.center.x - (self.size.width / 2.0),
   y: self.center.y - (self.size.height / 2.0)
   };

   println!("{:#?}", tl);

   let mut geometry: VertexBuffers<MyVertex, u16> =
   VertexBuffers::new();

   let options = FillOptions::tolerance(0.0001);

   let result = fill_rounded_rectangle(
   &rect(tl.x, tl.y, self.size.width, self.size.height),
   &BorderRadii {
   top_left: self.radius,
   top_right: self.radius,
   bottom_left: self.radius,
   bottom_right: self.radius
   },
   &options,
   &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
   MyVertex {
   x: vertex.position.x,
   y: vertex.position.y,
   z: 0.0
   }
   }),
   ).unwrap();

   let mut vertices: Vec<f32> = Vec::new();
   let mut indices: Vec<u32> = Vec::new();

   for vertex in geometry.vertices.iter() {
   vertices.push(vertex.x);
vertices.push(vertex.y);
vertices.push(vertex.z);
}

indices = geometry
    .indices
    .iter()
.map(|index| *index as u32)
    .collect();

(vertices, indices)
    }

pub fn rotate(&mut self, angle: f32) {
    self.current_rotation = angle;

    self.rotation_mat =
        cgmath::Matrix4::from_translation(
            cgmath::Vector3::new(self.center.x, self.center.y, 0.0)
        )
        * cgmath::Matrix4::from_angle_z(cgmath::Deg(angle)) *
        cgmath::Matrix4::from_translation(
            cgmath::Vector3::new(-self.center.x, -self.center.y, 0.0)
        );
}

pub fn scale(&mut self, factor: f32) {
    self.current_scale = factor;

    self.scale_mat = cgmath::Matrix4::from_scale(factor);
}

pub fn translate(&mut self, direction: cgmath::Vector2<f32>) {
    self.current_translate = direction;

    self.translation_mat = cgmath::Matrix4::from_translation(
        cgmath::Vector3::new(
            direction.x,
            direction.y,
            0.0
        )
    );
}

pub fn model(&self) -> cgmath::Matrix4<f32> {
    self.translation_mat
        * self.rotation_mat
        * self.scale_mat
}
}
*/

#[derive(Debug)]
pub struct CanvasData {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub glyph_vertices: Vec<f32>,
    pub glyph_tex_data: Vec<GlyphTexData>,
    pub primitives: Vec<Primitive>,
}

impl CanvasData {
    pub fn new() -> CanvasData {
        CanvasData {
            vertices: Vec::new(),
            indices: Vec::new(),
            glyph_vertices: Vec::new(),
            glyph_tex_data: Vec::new(),
            primitives: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Gradient {
    pub radius: f32,
    pub gradient_type: u32,
    pub start_pos: Vector,
    pub end_pos: Vector,
    pub first_color: Color,
    pub last_color: Color
}

impl Gradient {
    pub fn new() -> Gradient {
        Gradient {
            radius: 0.0,
            gradient_type: 0,
            start_pos: vector(0.5, 0.0),
            end_pos: vector(0.5, 1.0),
            first_color: Color::from_rgb(0.0, 0.0, 0.0),
            last_color: Color::from_rgb(0.0, 0.0, 0.0),
        }
    }

    pub fn from_values(
        radius: f32,
        gradient_type: u32,
        start_pos: Vector,
        end_pos: Vector,
        first_color: Color,
        last_color: Color)
        -> Gradient
    {
        Gradient {
            radius,
            gradient_type,
            start_pos,
            end_pos,
            first_color,
            last_color
        }
    }
}

#[derive(Debug)]
pub enum PrimitiveType {
    Text,
    Path,
}

#[derive(Debug)]
pub struct Primitive {
    pub kind: PrimitiveType,
    pub gradient: Gradient,
    pub num_vertices: u32,
    pub center: Point,
    pub model: cgmath::Matrix4<f32>,
    pub font: String,
    pub text: String,
    pub stroke_width: f32,
    pub bbox: cgmath::Vector4<f32>,
}

impl Primitive {
    pub fn new() -> Primitive {
        Primitive {
            font: String::from("dejavu"),
            kind: PrimitiveType::Path,
            text: String::new(),
            center: point(0.0, 0.0),
            model: cgmath::Transform::one(),
            gradient: Gradient::new(),
            stroke_width: 0.0,
            num_vertices: 0,
            bbox: cgmath::Vector4::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

#[derive(Clone, Debug, Copy)]
struct CtxVertex {
    position: Point
}

// Handle conversions to the gfx vertex format
impl VertexConstructor<FillVertex, CtxVertex> for CtxVertex {
    fn new_vertex(&mut self, vertex: FillVertex) -> CtxVertex {
        assert!(!vertex.position.x.is_nan());
        assert!(!vertex.position.y.is_nan());

        CtxVertex {
            position: vertex.position,
        }
    }
}

impl VertexConstructor<StrokeVertex, CtxVertex> for CtxVertex {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> CtxVertex {
        assert!(!vertex.position.x.is_nan());
        assert!(!vertex.position.y.is_nan());

        CtxVertex {
            position: vertex.position,
        }
    }
}

#[derive(Debug)]
enum CtxCommand {
    MoveTo(Point),
    LineTo(Point),
    Gradient(CtxDirection, Color, Color),
    StrokeWidth(f32),
    Arc(Point, Vector, Angle, Angle),
    Text(Point, String),
    Close,
}

#[derive(PartialEq, Debug)]
enum CtxDirection {
    CCW,
    CW,
    GradientX,
    GradientY,
    GradientRadial(Vector, f32),
}

struct Ctx {
    fill_tess: FillTessellator,
    stroke_tess: StrokeTessellator,
    mesh: VertexBuffers<CtxVertex, u32>,
    primitives: Vec<Primitive>,
    font_manager: FontManager,
    fonts: Vec<String>,
    prim_id: usize,
    path_direction: CtxDirection,
    gradient_direction: CtxDirection,
    commands: Vec<CtxCommand>,
}

impl Ctx {
    fn new() -> Ctx {
        Ctx {
            fill_tess: FillTessellator::new(),
            stroke_tess: StrokeTessellator::new(),
            mesh: VertexBuffers::new(),
            primitives: vec![],
            fonts: vec![],
            font_manager: FontManager::new(),
            gradient_direction: CtxDirection::GradientY,
            path_direction: CtxDirection::CW,
            prim_id: 0,
            commands: vec![],
        }
    }

    fn begin_mesh(&mut self) {
    }

    fn end_mesh(mut self) -> CanvasData {
        let mut vertices: Vec<f32> = Vec::new();
        let (glyph_vertices, glyph_tex_data) = self.font_manager.generate_glyph_vertices();

        for vertex in self.mesh.vertices.iter() {
            vertices.push(vertex.position.x);
            vertices.push(vertex.position.y);
            vertices.push(0.0);
        }

        let indices = self.mesh
            .indices
            .iter()
            .map(|index| *index as u32)
            .collect();

        CanvasData {
            indices,
            vertices,
            primitives: self.primitives,
            glyph_vertices,
            glyph_tex_data
        }
    }

    fn begin_primitive(&mut self) {
        self.commands.clear();
        self.path_direction = CtxDirection::CW;
        self.gradient_direction = CtxDirection::GradientY;
    }

    fn build_path(&mut self) -> (Path, Primitive) {
        let mut current_primitive = Primitive::new();

        let mut builder = Path::builder();

        for command in &self.commands {
            match command {
                CtxCommand::LineTo(p) => builder.line_to(*p),
                CtxCommand::MoveTo(p) => builder.move_to(*p),
                CtxCommand::Gradient(gradient_type, f_c, l_c) => {
                    match gradient_type {
                        CtxDirection::GradientY => {
                            current_primitive.gradient = Gradient {
                                radius: 0.0,
                                gradient_type: 0,
                                start_pos: vector(0.5, 1.0),
                                end_pos: vector(0.5, 0.0),
                                first_color: *f_c,
                                last_color: *l_c
                            };
                        },
                        CtxDirection::GradientX => {
                            current_primitive.gradient = Gradient {
                                radius: 0.0,
                                gradient_type: 0,
                                start_pos: vector(0.0, 0.5),
                                end_pos: vector(1.0, 0.5),
                                first_color: *f_c,
                                last_color: *l_c
                            };
                        },
                        CtxDirection::GradientRadial(c, r) => {
                            current_primitive.gradient = Gradient {
                                gradient_type: 1,
                                start_pos: *c,
                                end_pos: vector(0.0, 0.0),
                                radius: *r,
                                first_color: *f_c,
                                last_color: *l_c
                            };
                        }
                        _ => ()
                    }
                },
                CtxCommand::StrokeWidth(w) => {
                    current_primitive.stroke_width = *w;
                },
                CtxCommand::Arc(c, r, s, x) => {
                    builder.arc(*c, *r, *s, *x);
                },
                CtxCommand::Text(c, t) => {
                    current_primitive.center = *c;
                    current_primitive.font = "dejavu".to_string();
                    current_primitive.kind = PrimitiveType::Text;
                    current_primitive.text = t.to_string();
                },
                CtxCommand::Close => builder.close(),
            };
        }

        let path = builder.build();

        let bbox = lyon::algorithms::aabb::fast_bounding_rect(path.iter());

        // BBox is TOP, RIGHT, BOTTOM, LEFT coordinates
        // of the current path bounding box, calculated from
        // the center using width and height
        current_primitive.bbox = cgmath::Vector4::new(
            bbox.center().y + bbox.size.height / 2.0,
            bbox.center().x + bbox.size.width / 2.0,
            bbox.center().y - bbox.size.height / 2.0,
            bbox.center().x - bbox.size.width / 2.0,
        );

        self.fonts.push(current_primitive.font.clone());

        (path, current_primitive)
    }

    fn fill(&mut self) {
        // Ends the current primitive
        // and fills current path
        // with the fillPaint provided
        let (path, mut current_primitive) = self.build_path();

        let fill_options = FillOptions
            ::tolerance(0.01);

        let result = self.fill_tess.tessellate_path(
            &path,
            &fill_options,
            &mut BuffersBuilder::new(&mut self.mesh, |vertex : FillVertex| {
                CtxVertex {
                    position: vertex.position
                }
            }),
        );

        match result {
            Ok(result) => {
                current_primitive.num_vertices = result.indices;
            },
            Err(_) => {
                println!("Error during tesselletion");
            }
        }

        self.primitives.push(current_primitive);
    }

    fn stroke(&mut self) {
        // Ends the current primitive
        // and draws the stroke with the given
        // color and path
        let (path, mut current_primitive) = self.build_path();

        let stroke_options = StrokeOptions
            ::tolerance(0.01)
            .with_line_width(current_primitive.stroke_width);

        let result = self.stroke_tess.tessellate_path(
            &path,
            &stroke_options,
            &mut BuffersBuilder::new(&mut self.mesh, |vertex : StrokeVertex| {
                CtxVertex {
                    position: vertex.position
                }
            }),
        );

        match result {
            Ok(result) => {
                current_primitive.num_vertices = result.indices;
            },
            Err(_) => {
                println!("Error during tesselletion");
            }
        }

        self.primitives.push(current_primitive);
    }

    fn layout_text(&mut self) {
        // Ends the current text primitive
        // and lays text out with given
        // attributes from primitive

        // We don't use the path generated from Lyon
        // when dealing with text primitives
        let (_, mut current_primitive) = self.build_path();

        self.font_manager.position_glyphs(&mut current_primitive);

        current_primitive.model = cgmath::Matrix4::from_translation(
            cgmath::Vector3::new(
                current_primitive.center.x,
                current_primitive.center.y,
                0.0
            )
        );

        self.primitives.push(current_primitive);
    }

    fn text(&mut self, center: Point, chars: String) {
        self.commands.push(CtxCommand::Text(center, chars));

        self.layout_text();
    }

    fn rect(&mut self, top_left: Point, width: f32, height: f32) {
        let l = top_left.x;
        let r = top_left.x + width;
        let b = top_left.y;
        let t = top_left.y + height;

        if self.path_direction == CtxDirection::CCW {
            // Start from BottomLeft and go CCW to TopLeft
            self.move_to(point(l, b));
            self.line_to(point(r, b));
            self.line_to(point(r, t));
            self.line_to(point(l, t));
        } else {
            // Start from TopLeft and go CW to BottomLeft
            self.move_to(point(l, t));
            self.line_to(point(r, t));
            self.line_to(point(r, b));
            self.line_to(point(l, b));
        }

        self.close();
    }

    fn round_rect(&mut self, center: Point, width: f32, height: f32, radius: f32) {
        let min_w_h = width.min(height);

        let radius = radius.min(min_w_h / 2.0);

        let radii: Vector = vector(radius, radius);

        let c_left = center.x - width / 2.0 + radius;
        let c_right = center.x + width / 2.0 - radius;
        let c_top = center.y + height / 2.0 - radius;
        let c_bottom = center.y - height / 2.0 + radius;

        let c_tl= point(c_left, c_top);
        let c_tr = point(c_right, c_top);
        let c_bl = point(c_left, c_bottom);
        let c_br = point(c_right, c_bottom);

        let t_r: Point = point(c_right, c_top + radius);
        let r_b: Point = point(c_right + radius, c_bottom);
        let b_l: Point = point(c_left, c_bottom - radius);
        let l_t: Point = point(c_left - radius, c_top);

        self.move_to(l_t);
        self.arc(c_tl, radii,Angle::degrees(-90.0), Angle::degrees(180.0));
        self.line_to(t_r);
        self.arc(c_tr, radii, Angle::degrees(-90.0), Angle::degrees(90.0));
        self.line_to(r_b);
        self.arc(c_br, radii, Angle::degrees(-90.0), Angle::degrees(0.0));
        self.line_to(b_l);
        self.arc(c_bl, radii, Angle::degrees(-90.0), Angle::degrees(-90.0));
        self.line_to(l_t);

        self.close();
    }

    fn circle(&mut self, center: Point, radius: f32) {
        let radii: Vector = vector(radius, radius);

        let (mut start_angle, mut arc_angle) = (180.0, 360.0);

        if self.path_direction == CtxDirection::CCW {
            start_angle = 0.0;
            arc_angle = -360.0;
        }

        self.move_to(point(center.x - radius, center.y));
        self.arc(
            center,
            radii,
            Angle::degrees(arc_angle),
            Angle::degrees(start_angle)
        );

        self.close();
    }

    fn set_direction(&mut self, direction: CtxDirection) {
        self.path_direction = direction;
    }

    fn color(&mut self, c: Color) {
        self.gradient_y(c, c);
    }

    fn gradient_y(&mut self, first_color: Color, last_color: Color) {
        self.commands.push(CtxCommand::Gradient(
                CtxDirection::GradientY,
                first_color,
                last_color
        ));
    }

    fn gradient_x(&mut self, first_color: Color, last_color: Color) {
        self.commands.push(CtxCommand::Gradient(
                CtxDirection::GradientX,
                first_color,
                last_color
        ));
    }

    fn gradient_radial(&mut self, first_color: Color, last_color: Color, center: Vector, radius: f32) {
        self.commands.push(CtxCommand::Gradient(
                CtxDirection::GradientRadial(center, radius),
                first_color,
                last_color,
        ));
    }

    fn stroke_width(&mut self, width: f32) {
        self.commands.push(CtxCommand::StrokeWidth(width));
    }

    fn font_size(&mut self, width: f32) {
        // Alias to `stroke_width` for text primitive
        self.commands.push(CtxCommand::StrokeWidth(width));
    }

    fn move_to(&mut self, p: Point) {
        self.commands.push(CtxCommand::MoveTo(p));
    }

    fn arc(&mut self, center: Point, radii: Vector, sweep_angle: Angle, x_rotation: Angle) {
        // Draws an arc with radii { radius.x, radius.y }, centered in center
        // from x_rotation for sweep_angle's radians
        self.commands.push(CtxCommand::Arc(
                center,
                radii,
                sweep_angle,
                x_rotation,
        ));
    }

    fn line_to(&mut self, p: Point) {
        self.commands.push(CtxCommand::LineTo(p));
    }

    fn close(&mut self) {
        self.commands.push(CtxCommand::Close);
    }
}

pub fn generate_mesh() -> CanvasData {
    let mut ctx = Ctx::new();

    ctx.begin_mesh();

    ctx.begin_primitive();
    ctx.circle(point(400.0, 300.0), 100.0);
    ctx.circle(point(300.0, 200.0), 48.0);
    ctx.color(Color::from_rgba(1.0, 1.0, 1.0, 0.3));
    ctx.fill();

    ctx.begin_primitive();
    ctx.circle(point(400.0, 300.0), 100.0);
    ctx.circle(point(300.0, 200.0), 48.0);
    ctx.gradient_radial(
        Color::from_rgba(1.0, 1.0, 0.0, 1.0),
        Color::from_rgba(0.0, 0.0, 0.0, 0.0),
        vector(400.0, 300.0),
        120.0,
    );
    ctx.fill();

    ctx.begin_primitive();
    ctx.circle(point(400.0, 300.0), 100.0);
    ctx.circle(point(300.0, 200.0), 48.0);
    ctx.gradient_x(
        Color::from_rgba(1.0, 0.0, 0.0, 1.0),
        Color::from_rgba(1.0, 1.0, 0.0, 1.0)
    );
    ctx.stroke_width(10.0);
    ctx.stroke();

    ctx.begin_primitive();
    ctx.round_rect(point(200.0, 200.0), 100.0, 100.0, 20.0);
    ctx.round_rect(point(200.0, 200.0), 70.0, 70.0, 20.0);
    ctx.gradient_y(
        Color::from_rgba(0.8, 0.4, 0.6, 1.0),
        Color::from_rgba(0.8, 0.4, 0.6, 0.0),
    );
    ctx.fill();

    ctx.begin_primitive();
    ctx.round_rect(point(200.0, 200.0), 100.0, 100.0, 20.0);
    ctx.round_rect(point(200.0, 200.0), 70.0, 70.0, 20.0);
    ctx.gradient_y(
        Color::from_rgba(0.8, 0.4, 0.6, 1.0),
        Color::from_rgba(0.8, 0.4, 0.6, 0.0),
    );
    ctx.fill();

    ctx.begin_primitive();
    ctx.gradient_y(
        Color::from_rgb(0.5, 0.4, 0.8),
        Color::from_rgb(0.9, 0.1, 0.2),
    );
    ctx.font_size(150.0);
    ctx.text(point(100.0, 600.0), String::from("Quliq"));

    ctx.begin_primitive();
    ctx.gradient_y(
        Color::from_rgb(0.5, 0.4, 0.8),
        Color::from_rgb(0.9, 0.1, 0.2),
    );
    ctx.font_size(150.0);
    ctx.text(point(0.0, 0.0), String::from("Quliq"));

    ctx.begin_primitive();
    ctx.color(Color::from_rgb(0.4, 0.5, 0.7));
    ctx.font_size(80.0);
    ctx.text(point(200.0, 200.0), String::from("Simone"));

    ctx.begin_primitive();
    ctx.color(Color::from_rgb(0.1, 0.7, 0.8));
    ctx.font_size(20.0);
    ctx.text(point(400.0, 400.0), String::from("Insomma come andiamo ragazzo bello bello culo 1234565767899"));

    ctx.end_mesh()
}

pub fn generate_mesh_from_widget(children: &Vec<Widget>) -> CanvasData {
    println!("Widgets: {}", children.len());
    let mut ctx = Ctx::new();

    ctx.begin_mesh();

    for child in children.iter() {
        ctx.begin_primitive();
        ctx.color(child.color);
        ctx.rect(child.position, child.size.width, child.size.height);
        ctx.fill();
    }

    ctx.end_mesh()
}
