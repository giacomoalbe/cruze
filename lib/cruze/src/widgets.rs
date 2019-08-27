use lyon::math::{
    Point,
    point
};

use super::canvas::{
    Size,
    Color,
    Ctx
};

use stretch::{
    Stretch,
    style::*,
};

pub trait Widget {
    fn draw(&self, ctx:&mut Ctx);
    fn generate_stretch_node(&self, stretch: &mut Stretch) -> stretch::node::Node;
    fn set_size(&mut self, size: Size<f32>);
    fn set_position(&mut self, position: Point);
    fn update_layout(&mut self, stretch: &Stretch, node: &stretch::node::Node) {
        ()
    }
    fn update_coords(&mut self, position: Point);
}

pub struct WidgetOptions {
    pub color: Color,
    pub padding: stretch::geometry::Rect<stretch::style::Dimension>,
    pub margin: stretch::geometry::Rect<stretch::style::Dimension>,
    pub radius: f32,
    pub size: stretch::geometry::Size<Dimension>,
    pub flex: f32,
}

impl WidgetOptions {
    pub fn uniform_padding(amount: f32) -> stretch::geometry::Rect<stretch::style::Dimension> {
        stretch::geometry::Rect {
            top: stretch::style::Dimension::Points(amount),
            bottom: stretch::style::Dimension::Points(amount),
            start: stretch::style::Dimension::Points(amount),
            end: stretch::style::Dimension::Points(amount)
        }
    }

    pub fn percent(amount: f32) -> stretch::geometry::Size<Dimension> {
        stretch::geometry::Size {
            width: Dimension::Percent(amount / 100.0),
            height: Dimension::Auto
        }
    }
}

impl Default for WidgetOptions {
    fn default() -> WidgetOptions {
        WidgetOptions {
            color: Color::from_rgb(1.0, 1.0, 1.0),
            padding: WidgetOptions::uniform_padding(0.0),
            margin: WidgetOptions::uniform_padding(0.0),
            radius: 0.0,
            size: stretch::geometry::Size {
                width: Dimension::Undefined,
                height: Dimension::Undefined
            },
            flex: 1.0
        }
    }
}

pub struct Rect {
    pub size: Size<f32>,
    pub position: Point,
    pub options: WidgetOptions,
}

impl Rect {
    pub fn new(options: WidgetOptions) -> Box<Rect> {
        Box::new(Rect {
            size: Size::new(0.0, 0.0),
            position: point(0.0, 0.0),
            options,
        })
    }
}

impl Widget for Rect {
    fn draw(&self, ctx: &mut Ctx) {
        ctx.begin_primitive();
        ctx.color(self.options.color);
        ctx.round_rect(self.position, self.size.width, self.size.height, self.options.radius);
        ctx.fill();
    }

    fn generate_stretch_node(&self, stretch: &mut Stretch) -> stretch::node::Node {
        println!("{:?}", self.options.size);
        stretch.new_node(
            Style {
                flex_grow: self.options.flex,
                min_size: self.options.size,
                max_size: self.options.size,
                ..Default::default()
            },
            vec![],
        ).unwrap()
    }

    fn set_size(&mut self, size: Size<f32>) {
        self.size = size;
    }

    fn set_position(&mut self, position: Point) {
        self.position = position;
    }

    fn update_layout(&mut self, stretch: &Stretch, node: &stretch::node::Node) {
        let layout = stretch.layout(*node).unwrap();

        self.set_size(Size {
            width: layout.size.width,
            height: layout.size.height
        });

        self.set_position(lyon::math::point(
            layout.location.x,
            layout.location.y
        ));

        //println!("Draw rect: ({} x {}) [{}, {}]", self.size.width, self.size.height, self.position.x, self.position.y);
    }

    fn update_coords(&mut self, position: Point) {
        self.set_position(lyon::math::point(
            self.position.x + position.x,
            self.position.y + position.y
        ));
    }
}

#[derive(Default)]
pub struct Container {
    pub size: Size<f32>,
    pub position: Point,
    pub options: WidgetOptions,
    pub orientation: FlexDirection,
    pub children: Vec<Box<dyn Widget>>
}

impl Container {
    pub fn new(color: Color, flex: f32, children: Vec<Box<dyn Widget>>) -> Box<Container> {
        Box::new(Container {
            size: Size::new(0.0, 0.0),
            position: point(0.0, 0.0),
            orientation: FlexDirection::default(),
            options: WidgetOptions::default(),
            children
        })
    }
}

impl Widget for Container {
    fn draw(&self, ctx: &mut Ctx) {
        ctx.begin_primitive();
        ctx.color(self.options.color);
        ctx.rect(self.position, self.size.width, self.size.height);
        ctx.fill();

        for child in self.children.iter() {
            child.draw(ctx);
        }
    }

    fn generate_stretch_node(&self, stretch: &mut Stretch) -> stretch::node::Node {
        let mut children_nodes = vec![];

        for child in self.children.iter() {
            children_nodes.push(child.generate_stretch_node(stretch));
        }

        stretch.new_node(
            Style {
                flex_direction: self.orientation,
                flex_grow: self.options.flex,
                padding: self.options.padding,
                margin: self.options.margin,
                ..Default::default()
            },
            children_nodes
        ).unwrap()
    }

    fn update_layout(&mut self, stretch: &Stretch, node: &stretch::node::Node) {
        let layout = stretch.layout(*node).unwrap();

        self.set_size(Size {
            width: layout.size.width,
            height: layout.size.height
        });

        self.set_position(lyon::math::point(
            layout.location.x,
            layout.location.y
        ));

        //println!("Draw cont: ({} x {}) [{}, {}]", self.size.width, self.size.height, self.position.x, self.position.y);

        for (index, child_node) in stretch.children(*node).unwrap().iter().enumerate() {
            // Get child at position index, this is the array of children of Window
            let mut child = self.children.get_mut(index).unwrap();

            child.update_layout(&stretch, child_node);
            child.update_coords(self.position);
        }
    }

    fn update_coords(&mut self, position: Point) {
        self.set_position(lyon::math::point(
            self.position.x + position.x,
            self.position.y + position.y
        ));
    }

    fn set_size(&mut self, size: Size<f32>) {
        self.size = size;
    }

    fn set_position(&mut self, position: Point) {
        self.position = position;
    }
}

#[derive(Default)]
pub struct Col;

impl Col {
    pub fn new(options: WidgetOptions, children: Vec<Box<dyn Widget>>) -> Box<Container> {
        Box::new(Container {
            size: Size::new(0.0, 0.0),
            position: point(0.0, 0.0),
            orientation: FlexDirection::Column,
            options: options,
            children
        })
    }
}

#[derive(Default)]
pub struct Row;

impl Row {
    pub fn new(options: WidgetOptions, children: Vec<Box<dyn Widget>>) -> Box<Container> {
        Box::new(Container {
            size: Size::new(0.0, 0.0),
            position: point(0.0, 0.0),
            orientation: FlexDirection::Row,
            options: options,
            children
        })
    }
}
