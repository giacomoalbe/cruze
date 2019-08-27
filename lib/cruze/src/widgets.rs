use lyon::math::{
    Point,
    point
};

use super::canvas::{
    Size,
    Color,
    Ctx
};

use super::font_manager::FontManager;

use stretch::{
    Stretch,
    style::*,
};

use std::convert::{
    Into
};

#[derive(Clone, Copy)]
pub enum Orientation {
    Column,
    Row
}

impl Into<FlexDirection> for Orientation {
    fn into(self) -> FlexDirection {
        match self {
            Orientation::Column => FlexDirection::Column,
            Orientation::Row => FlexDirection::Row
        }
    }
}

#[derive(Clone, Copy)]
pub enum Alignment {
    Start,
    Center,
    End,
    Baseline,
    SpaceEvenly,
    SpaceAround,
    SpaceBetween,
    Undefined,
}

impl Into<AlignItems> for Alignment {
    fn into(self) -> AlignItems {
        match self {
            Alignment::Start => AlignItems::FlexStart,
            Alignment::Center => AlignItems::Center,
            Alignment::End => AlignItems::FlexEnd,
            Alignment::Baseline => AlignItems::Baseline,
            Alignment::Undefined => AlignItems::Stretch,
            _ => AlignItems::Stretch,
        }
    }
}

impl Into<JustifyContent> for Alignment {
    fn into(self) -> JustifyContent {
        match self {
            Alignment::Start => JustifyContent::FlexStart,
            Alignment::Center => JustifyContent::Center,
            Alignment::End => JustifyContent::FlexEnd,
            Alignment::Undefined => JustifyContent::FlexStart,
            Alignment::SpaceBetween => JustifyContent::SpaceBetween,
            Alignment::SpaceAround => JustifyContent::SpaceAround,
            Alignment::SpaceEvenly => JustifyContent::SpaceEvenly,
            _ => JustifyContent::FlexStart,
        }
    }
}

pub trait Widget {
    fn draw(&self, ctx: &mut Ctx, font_manager: &mut FontManager);
    fn generate_stretch_node(&self, stretch: &mut Stretch, font_manager: &mut FontManager) -> stretch::node::Node;
    fn set_size(&mut self, size: Size<f32>);
    fn set_position(&mut self, position: Point);
    fn update_layout(&mut self, stretch: &Stretch, node: &stretch::node::Node) {
        ()
    }
    fn update_coords(&mut self, position: Point);
    fn debug(&self);
}

pub struct WidgetOptions {
    pub color: Color,
    pub padding: stretch::geometry::Rect<stretch::style::Dimension>,
    pub margin: stretch::geometry::Rect<stretch::style::Dimension>,
    pub vertical_align: Alignment,
    pub horizontal_align: Alignment,
    pub radius: f32,
    pub size: stretch::geometry::Size<Dimension>,
    pub orientation: Orientation,
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

    pub fn uniform_size(amount: f32) -> stretch::geometry::Size<Dimension> {
        stretch::geometry::Size {
            width: Dimension::Points(amount),
            height: Dimension::Points(amount)
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
            orientation: Orientation::Row,
            size: stretch::geometry::Size {
                width: Dimension::Undefined,
                height: Dimension::Undefined
            },
            vertical_align: Alignment::Undefined,
            horizontal_align: Alignment::Undefined,
            flex: 1.0
        }
    }
}

#[derive(Default)]
pub struct Container {
    pub size: Size<f32>,
    pub position: Point,
    pub options: WidgetOptions,
    pub children: Vec<Box<dyn Widget>>
}

impl Container {
    pub fn new(children: Vec<Box<dyn Widget>>) -> Box<Container> {
        Box::new(Container {
            size: Size::new(0.0, 0.0),
            position: point(0.0, 0.0),
            options: WidgetOptions::default(),
            children
        })
    }
}

impl Widget for Container {
    fn draw(&self, ctx: &mut Ctx, font_manager: &mut FontManager) {
        ctx.begin_primitive();
        ctx.color(self.options.color);
        ctx.round_rect(self.position, self.size.width, self.size.height, self.options.radius);
        ctx.fill();

        for child in self.children.iter() {
            child.draw(ctx, font_manager);
        }
    }

    fn generate_stretch_node(&self, stretch: &mut Stretch, font_manager: &mut FontManager) -> stretch::node::Node {
        let mut children_nodes = vec![];

        for child in self.children.iter() {
            children_nodes.push(child.generate_stretch_node(stretch, font_manager));
        }

        let mut align_items = Alignment::Undefined;
        let mut justify_content = Alignment::Undefined;

        match self.options.orientation {
            Orientation::Row => {
                align_items = self.options.vertical_align;
                justify_content = self.options.horizontal_align;
            },
            Orientation::Column => {
                align_items = self.options.horizontal_align;
                justify_content = self.options.vertical_align;
            },
            _ => ()
        }

        stretch.new_node(
            Style {
                align_items: align_items.into(),
                justify_content: justify_content.into(),
                flex_direction: self.options.orientation.into(),
                flex_grow: self.options.flex,
                min_size: self.options.size,
                max_size: self.options.size,
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

        //println!("Draw cont (container): ({} x {}) [{}, {}]", self.size.width, self.size.height, self.position.x, self.position.y);
    }

    fn set_size(&mut self, size: Size<f32>) {
        self.size = size;
    }

    fn set_position(&mut self, position: Point) {
        self.position = position;
    }

    fn debug(&self) {
        println!("({} x {}) [{}, {}]", self.size.width, self.size.height, self.position.x, self.position.y);
    }

}

pub struct Rect;

impl Rect {
    pub fn new(options: WidgetOptions, children: Vec<Box<dyn Widget>>) -> Box<Container> {
        Box::new(Container {
            size: Size::new(0.0, 0.0),
            position: point(0.0, 0.0),
            options: options,
            children
        })
    }
}

pub struct Label {
    pub size: Size<f32>,
    pub position: Point,
    pub text: String,
    pub options: WidgetOptions,
}

impl Label {
    pub fn new(options: WidgetOptions, text: String) -> Box<Label> {
        Box::new(Label {
            size: Size::new(0.0, 0.0),
            position: point(0.0, 0.0),
            options,
            text,
        })
    }
}

impl Widget for Label {
    fn draw(&self, ctx: &mut Ctx, font_manager: &mut FontManager) {
        ctx.begin_primitive();
        ctx.color(Color::from_rgb(1.0, 0.0, 0.0));
        ctx.round_rect(self.position, self.size.width, self.size.height, self.options.radius);
        ctx.fill();

        ctx.begin_primitive();
        ctx.color(self.options.color);
        ctx.font_size(24.0);
        ctx.text(self.position, self.text.clone(), font_manager);
    }

    fn generate_stretch_node<'a>(&self, stretch: &mut Stretch, font_manager: &mut FontManager) -> stretch::node::Node {
        let (bbox, _) = font_manager.calculate_text_bbox(24.0, "dejavu".to_string(), &self.text);

        println!("BBOX: ({} x {})", (bbox.y - bbox.w), (bbox.x - bbox.z));

        stretch.new_leaf(
            Style {
                padding: self.options.padding,
                margin: self.options.margin,
                ..Default::default()
            },
            Box::new(move |s| {
                Ok(stretch::geometry::Size {
                    width: (bbox.y - bbox.w),
                    height: (bbox.x - bbox.z),
                })
            })
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

    }

    fn update_coords(&mut self, position: Point) {
        self.set_position(lyon::math::point(
            self.position.x + position.x,
            self.position.y + position.y
        ));

        println!("Draw cont (label): ({} x {}) [{}, {}]", self.size.width, self.size.height, self.position.x, self.position.y);
    }

    fn set_size(&mut self, size: Size<f32>) {
        self.size = size;
    }

    fn set_position(&mut self, position: Point) {
        self.position = position;
    }

    fn debug(&self) {
        println!("({} x {}) [{}, {}]", self.size.width, self.size.height, self.position.x, self.position.y);
    }
}
