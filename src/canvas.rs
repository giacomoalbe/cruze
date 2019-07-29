extern crate lyon;

use lyon::math::rect;
use lyon::tessellation::basic_shapes::*;
use lyon::tessellation::{
    VertexBuffers,
    FillOptions,
    FillVertex
};
use lyon::tessellation::geometry_builder::BuffersBuilder;

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

pub fn rectangle(width: f32, height: f32, radius: f32)
->  (Vec<f32>, Vec<u32>)
{
    #[derive(Copy, Clone, Debug)]
    struct MyVertex {
        x: f32,
        y: f32,
        z: f32
    };

    let mut geometry: VertexBuffers<MyVertex, u16> =
        VertexBuffers::new();

    let options = FillOptions::tolerance(0.0001);

    let result = fill_rounded_rectangle(
        &rect(0.0, 0.0, width, height),
        &BorderRadii {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius
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
        vertices.push(1.0);
        vertices.push(0.0);
        vertices.push(0.0);
    }

    indices = geometry.indices.iter().map(|index| *index as u32).collect();

    (vertices, indices)
}
