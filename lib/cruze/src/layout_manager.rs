use super::widgets::Widget;

use super::canvas;

use stretch::{
    style::*,
    node::{
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

    pub fn build(&mut self, size: glutin::dpi::LogicalSize, children: &mut Vec<Box<dyn Widget>>) {
        let mut stretch = Stretch::new();

        let mut children_nodes  = vec![];

        for child in children.iter() {
            children_nodes.push(child.generate_stretch_node(&mut stretch));
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

        for (index, child_node) in stretch.children(main_node).unwrap().iter().enumerate() {
            // Get child at position index, this is the array of children of Window
            let mut child = children.get_mut(index).unwrap();

            child.update_layout(&stretch, child_node);

            //println!("Index: {}\nSize: {:?}\nLocation: {:?}\n#############", index, child_node_layout.size, child_node_layout.location);
        }
    }
}
