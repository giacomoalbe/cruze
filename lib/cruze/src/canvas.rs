extern crate lyon;

use lyon::math::{
    point,
    Point,
    vector,
    Vector,
    Angle,
    rect
};

use lyon::tessellation::basic_shapes::*;
use lyon::tessellation::{
    FillOptions,
    FillTessellator,
    StrokeTessellator,
    FillVertex
};

use lyon::path::Path;
use lyon::path::builder::*;
use lyon::tessellation::geometry_builder::{BuffersBuilder, VertexBuffers};
use std::borrow::BorrowMut;
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
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Color {
        Color {
            r,
            g,
            b
        }
    }

    pub fn to_vec3(&self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(self.r, self.g, self.b)
    }
}

fn color(r: f32, g: f32, b: f32) -> Color {
    Color::new(r,g,b)
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

pub struct Screen {
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
        }
    }

    pub fn draw(&self) {
        println!("Just drawing...");
    }
}

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

pub struct Primitive {
    color: Color,
    num_vertices: u32,
}

impl Primitive {
    pub fn new() -> Primitive {
        Primitive {
            color: Color::new(1.0, 0.0, 0.0),
            num_vertices: 0
        }
    }
}

enum CtxCommand {
    MoveTo(Point),
    LineTo(Point),
    FillColor(Color),
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
    mesh: VertexBuffers<FillVertex, u32>,
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

    fn end_primitive(&mut self) {
        let mut current_primitive = Primitive::new();

        let mut builder = Path::builder();

        for command in &self.commands {
            match command {
                CtxCommand::LineTo(p) => builder.line_to(*p),
                CtxCommand::MoveTo(p) => builder.move_to(*p),
                CtxCommand::FillColor(c) => {
                    current_primitive.color = c.clone();
                },
                CtxCommand::Arc(c, r, s, x) => {
                    builder.arc(*c, *r, *s, *x);
                },
                CtxCommand::Close => builder.close(),
            };
        }

        let path = builder.build();

        #[derive(Clone, Debug, Copy)]
        struct MyVertex { position: [f32; 2], normal: [f32; 2] };

        let result = self.fill_tess.tessellate_path(
            &path,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut self.mesh, |vertex : FillVertex| {
                vertex
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

        println!("True radius: {}", radius);

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

    fn wind_rect(&mut self, center: Point, width: f32, height: f32, inset: f32) {
        let l = center.x - width / 2.0;
        let r = center.x + width / 2.0;
        let b = center.y - height / 2.0;
        let t = center.y + height / 2.0;

        let l_i = l + inset;
        let r_i = r - inset;
        let t_i = t - inset;
        let b_i = b + inset;

        let tl = point(l, t);
        let tr = point(r, t);
        let bl = point(l, b);
        let br = point(r, b);

        let tl_i: Point = point(l_i, t_i);
        let tr_i: Point = point(r_i, t_i);
        let bl_i: Point = point(l_i, b_i);
        let br_i: Point = point(r_i, b_i);

        self.move_to(tl);
        self.line_to(tr);
        self.line_to(br);
        self.line_to(bl);
        self.line_to(tl);

        self.close();

        self.move_to(tl_i);
        self.line_to(tr_i);
        self.line_to(br_i);
        self.line_to(bl_i);

        self.close();
    }

    fn set_direction(&mut self, direction: CtxDirection) {
        self.path_direction = direction;
    }

    fn fill_color(&mut self, c: Color) {
        self.commands.push(CtxCommand::FillColor(c));
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
    ctx.move_to(point(100.0, 100.0));
    ctx.line_to(point(100.0, 200.0));
    ctx.line_to(point(200.0, 200.0));
    ctx.line_to(point(200.0, 100.0));
    ctx.fill_color(color(0.5, 0.1, 1.0));
    ctx.close();

    ctx.end_primitive();

    ctx.begin_primitive();
    ctx.move_to(point(400.0, 400.0));
    ctx.line_to(point(400.0, 500.0));
    ctx.line_to(point(500.0, 500.0));
    ctx.line_to(point(500.0, 400.0));
    ctx.fill_color(color(0.1, 0.8, 0.2));
    ctx.close();
    ctx.end_primitive();
    */

    ctx.begin_primitive();
    ctx.rect(point(300.0, 300.0), 200.0, 60.0);
    ctx.fill_color(color(1.0, 0.0, 0.0));
    ctx.end_primitive();

    ctx.begin_primitive();
    ctx.rect(point(600.0, 400.0), 200.0, 60.0);
    ctx.fill_color(color(0.0, 1.0, 1.0));
    ctx.end_primitive();

    ctx.begin_primitive();
    ctx.rect(point(400.0, 60.0), 50.0, 50.0);
    ctx.fill_color(color(0.0, 0.0, 1.0));
    ctx.end_primitive();

    ctx.begin_primitive();
    ctx.rect(point(500.0, 200.0), 5.0, 5.0);
    ctx.fill_color(color(1.0, 0.0, 0.0));
    ctx.end_primitive();

    ctx.begin_primitive();
    ctx.rect(point(200.0, 100.0), 200.0, 60.0);
    ctx.fill_color(color(0.0, 1.0, 1.0));
    ctx.end_primitive();

    ctx.begin_primitive();
    ctx.round_rect(point(150.0, 400.0), 250.0, 150.0, 5.0);
    ctx.fill_color(color(1.0, 1.0, 1.0));
    ctx.end_primitive();

    ctx.begin_primitive();
    ctx.circle(point(500.0, 200.0), 80.0);
    ctx.fill_color(color(0.0, 0.5, 0.8));
    ctx.end_primitive();

    ctx.begin_primitive();
    ctx.round_rect(point(200.0, 200.0), 80.0, 80.0, 10.0);
    ctx.fill_color(color(1.0, 0.0, 0.0));
    ctx.end_primitive();

    ctx.begin_primitive();
    ctx.rect(point(200.0, 200.0), 40.0, 40.0);
    ctx.fill_color(color(1.0, 1.0, 0.0));
    ctx.end_primitive();

    ctx.begin_primitive();
    ctx.round_rect(point(400.0, 500.0), 100.0, 80.0, 10.0);
    ctx.rect(point(400.0, 500.0), 80.0, 60.0);
    ctx.fill_color(color(0.0, 0.5, 1.0));
    ctx.end_primitive();

    ctx.end_mesh()
}

pub fn draw_primitives(gl: gl::Gl, program: &mut Program, primitives: &Vec<Primitive>) {
    let start_time = std::time::Instant::now();

    let mut tris_offset = 0;

    for primitive in primitives {
        unsafe {
            program.set_color(
                CStr::from_bytes_with_nul_unchecked(b"color\0"),
                &primitive.color
            );

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

    println!("Frame render: {}", start_time.elapsed().as_micros());
}
