extern crate lyon;

use lyon::math::{
    point,
    Point,
    vector,
    Vector,
    Angle,
    rect
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

use lyon::path::builder::*;
use std::ffi::{CStr};

use cgmath::prelude::*;
use super::render_gl::Program;

#[derive(Debug)]
pub struct Point2<T> {
    x: T,
    y: T,
}

impl<T> Point2<T> {
    pub fn new(x: T, y: T) -> Point2<T> {
        Point2 {
            x,
            y
        }
    }
}

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

#[derive(Debug)]
pub struct Size<T> {
    width: T,
    height: T,
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
pub struct Gradient {
    pub start_pos: Vector,
    pub end_pos: Vector,
    pub first_color: Color,
    pub last_color: Color
}

impl Gradient {
    pub fn new() -> Gradient {
        Gradient {
            start_pos: vector(0.5, 0.0),
            end_pos: vector(0.5, 1.0),
            first_color: Color::from_rgb(0.0, 0.0, 0.0),
            last_color: Color::from_rgb(0.0, 0.0, 0.0),
        }
    }

    pub fn from_values(
        start_pos: Vector,
        end_pos: Vector,
        first_color: Color,
        last_color: Color)
    -> Gradient
    {
        Gradient {
            start_pos,
            end_pos,
            first_color,
            last_color
        }
    }
}

#[derive(Debug)]
pub struct Primitive {
    gradient: Gradient,
    num_vertices: u32,
    stroke_width: f32,
    bbox: cgmath::Vector4<f32>,
}

impl Primitive {
    pub fn new() -> Primitive {
        Primitive {
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

enum CtxCommand {
    MoveTo(Point),
    LineTo(Point),
    Gradient(CtxDirection, Color, Color),
    StrokeWidth(f32),
    Arc(Point, Vector, Angle, Angle),
    Close,
}

#[derive(PartialEq)]
enum CtxDirection {
    CCW,
    CW,
    GradientX,
    GradientY,
}

struct Ctx {
    fill_tess: FillTessellator,
    stroke_tess: StrokeTessellator,
    mesh: VertexBuffers<CtxVertex, u32>,
    primitives: Vec<Primitive>,
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
            gradient_direction: CtxDirection::GradientY,
            path_direction: CtxDirection::CW,
            prim_id: 0,
            commands: vec![],
        }
    }

    fn begin_mesh(&mut self) {
    }

    fn end_mesh(self) -> (Vec<f32>, Vec<u32>, Vec<Primitive>) {
        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for vertex in self.mesh.vertices.iter() {
            vertices.push(vertex.position.x);
            vertices.push(vertex.position.y);
            vertices.push(0.0);
        }

        indices = self.mesh
            .indices
            .iter()
            .map(|index| *index as u32)
            .collect();

        (vertices, indices, self.primitives)
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
                                start_pos: vector(0.5, 1.0),
                                end_pos: vector(0.5, 0.0),
                                first_color: *f_c,
                                last_color: *l_c
                            };
                        },
                        CtxDirection::GradientX => {
                            current_primitive.gradient = Gradient {
                                start_pos: vector(1.0, 0.5),
                                end_pos: vector(0.0, 0.5),
                                first_color: *f_c,
                                last_color: *l_c
                            };
                        },
                        _ => ()
                    }
                },
                CtxCommand::StrokeWidth(w) => {
                    current_primitive.stroke_width = *w;
                },
                CtxCommand::Arc(c, r, s, x) => {
                    builder.arc(*c, *r, *s, *x);
                },
                CtxCommand::Close => builder.close(),
            };
        }

        let path = builder.build();

        let bbox =  lyon::algorithms::aabb::fast_bounding_rect(path.iter());

        // BBox is TOP, RIGHT, BOTTOM, LEFT coordinates
        // of the current path bounding box, calculated from
        // the center using width and height
        current_primitive.bbox = cgmath::Vector4::new(
            bbox.center().y + bbox.size.height / 2.0,
            bbox.center().x + bbox.size.width / 2.0,
            bbox.center().y - bbox.size.height / 2.0,
            bbox.center().x - bbox.size.width / 2.0,
        );

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

    fn rect(&mut self, center: Point, width: f32, height: f32) {
        let l = center.x - width / 2.0;
        let r = center.x + width / 2.0;
        let b = center.y - height / 2.0;
        let t = center.y + height / 2.0;

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

        let t_l: Point = point(c_left, c_top + radius);
        let t_r: Point = point(c_right, c_top + radius);
        let r_t: Point = point(c_right + radius, c_top);
        let r_b: Point = point(c_right + radius, c_bottom);
        let b_r: Point = point(c_right, c_bottom - radius);
        let b_l: Point = point(c_left, c_bottom - radius);
        let l_b: Point = point(c_left - radius, c_bottom);
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

        //self.arc(center, radii, Angle::degrees(360.0), Angle::degrees(0.0));
        //self.arc(center, radii, Angle::degrees(360.0), Angle::degrees(90.0));
        //self.arc(center, radii, Angle::degrees(360.0 + 90.0), Angle::degrees(360.0));
        //self.arc(center, radii, Angle::degrees(180.0), Angle::degrees(270.0));
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

    fn stroke_width(&mut self, width: f32) {
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

pub fn generate_mesh() -> (Vec<f32>, Vec<u32>, Vec<Primitive>) {
    let mut ctx = Ctx::new();

    ctx.begin_mesh();

    /*
    ctx.begin_primitive();
    ctx.rect(point(300.0, 300.0), 200.0, 60.0);
    ctx.color(color(1.0, 0.0, 0.0));
    ctx.fill();

    ctx.begin_primitive();
    ctx.rect(point(600.0, 400.0), 200.0, 60.0);
    ctx.color(color(0.0, 1.0, 1.0));
    ctx.fill();

    ctx.begin_primitive();
    ctx.rect(point(400.0, 60.0), 50.0, 50.0);
    ctx.color(color(0.0, 0.0, 1.0));
    ctx.fill();

    ctx.begin_primitive();
    ctx.rect(point(500.0, 200.0), 5.0, 5.0);
    ctx.color(color(1.0, 0.0, 0.0));
    ctx.fill();

    ctx.begin_primitive();
    ctx.rect(point(200.0, 100.0), 200.0, 60.0);
    ctx.color(color(0.0, 1.0, 1.0));
    ctx.fill();

    ctx.begin_primitive();
    ctx.round_rect(point(150.0, 400.0), 250.0, 150.0, 5.0);
    ctx.color(color(1.0, 1.0, 1.0));
    ctx.fill();

    ctx.begin_primitive();
    ctx.circle(point(500.0, 200.0), 80.0);
    ctx.gradient_y(color(1.0, 0.0, 0.0), color(0.0, 0.0, 1.0));
    ctx.fill();

    ctx.begin_primitive();
    ctx.round_rect(point(200.0, 200.0), 80.0, 80.0, 10.0);
    ctx.color(color(1.0, 0.0, 0.0));
    ctx.fill();

    ctx.begin_primitive();
    ctx.round_rect(point(200.0, 200.0), 200.0, 200.0, 5.0);
    ctx.color(color(0.0, 0.0, 1.0));
    ctx.fill();

    ctx.begin_primitive();
    ctx.round_rect(point(200.0, 200.0), 200.0, 200.0, 5.0);
    ctx.color(color(1.0, 1.0, 1.0));
    ctx.stroke_width(3.0);
    ctx.stroke();

    ctx.begin_primitive();
    ctx.round_rect(point(400.0, 500.0), 100.0, 80.0, 10.0);
    ctx.rect(point(400.0, 500.0), 80.0, 60.0);
    ctx.gradient_y(color(1.0, 0.0, 0.0), color(0.0, 1.0, 0.0));
    ctx.fill();
    */

    ctx.begin_primitive();
    ctx.circle(point(400.0, 300.0), 300.0);
    //ctx.circle(point(300.0, 200.0), 48.0);
    ctx.gradient_y(
        Color::from_rgba(1.0, 0.0, 0.0, 1.0),
        Color::from_rgba(0.0, 0.0, 0.0, 0.0),
    );
    ctx.fill();

    ctx.begin_primitive();
    ctx.circle(point(400.0, 300.0), 300.0);
    //ctx.circle(point(300.0, 200.0), 48.0);
    ctx.color(Color::from_rgba(1.0, 1.0, 1.0, 1.0));
    ctx.stroke_width(10.0);
    ctx.stroke();

    ctx.end_mesh()
}

pub fn draw_primitives(gl: gl::Gl, program: &mut Program, primitives: &Vec<Primitive>) {
    let start_time = std::time::Instant::now();

    let mut tris_offset = 0;

    for primitive in primitives {
        unsafe {
            program.set_vec4("bbox", &primitive.bbox);
            program.set_gradient(&primitive.gradient);

            gl.DrawElements(
                gl::TRIANGLES,
                primitive.num_vertices as i32,
                gl::UNSIGNED_INT,
                (tris_offset * std::mem::size_of::<gl::types::GLuint>())
                    as *const std::ffi::c_void
            );
        }

        tris_offset += primitive.num_vertices as usize;
    }

    //println!("Frame render: {}", start_time.elapsed().as_micros());
}
