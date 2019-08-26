use super::window::Widget;

use super::canvas;

use stretch::{
    style::*,
    number::Number,
    node::{
        Node,
        Stretch,
    },
    geometry::Size
};

pub struct WidgetPosLoc {
    pub size: canvas::Size<f32>,
    pub position: lyon::math::Point,
    pub index: usize
}

pub struct LayoutBuilder {
}

impl LayoutBuilder {
    pub fn new() -> LayoutBuilder {
        LayoutBuilder {
        }
    }
    pub fn build(&mut self, size: glutin::dpi::LogicalSize, children: &Vec<Widget>) -> Vec<WidgetPosLoc> {
        let mut stretch = Stretch::new();

        let mut children_nodes  = vec![];

        for child in children.iter() {
            children_nodes.push(
                stretch.new_node(
                    Style {
                        flex_grow: child.flex,
                        ..Default::default()
                    },
                    vec![],
                ).unwrap()
            );
        }

        let main_node = stretch.new_node(
            Style {
                size: Size {
                    width: Dimension::Points(size.width as f32),
                    height: Dimension::Points(size.height as f32),
                },
                ..Default::default()
            },
            children_nodes
        ).unwrap();

        stretch.compute_layout(
            main_node,
            Size::undefined()
        ).unwrap();

        let mut children_pos_loc = vec![];

        for (index, child_node) in stretch.children(main_node).unwrap().iter().enumerate() {
            let child_node_layout = stretch.layout(*child_node).unwrap();

            children_pos_loc.push(WidgetPosLoc {
                size: canvas::Size {
                    width: child_node_layout.size.width,
                    height: child_node_layout.size.height
                },
                position: lyon::math::point(
                    child_node_layout.location.x,
                    child_node_layout.location.y
                ),
                index: index,
            });

            println!("Index: {}\nSize: {:?}\nLocation: {:?}\n#############", index, child_node_layout.size, child_node_layout.location);
        }

        children_pos_loc
    }
}
