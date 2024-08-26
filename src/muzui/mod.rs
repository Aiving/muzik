use elements::text::TextElement;
use layout::{Measurer, Rect, Size};
use material_colors::color::Argb;
use node::Node;
use skia_safe::{
    font_style::{FontStyle, Width},
    surfaces,
    textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
    EncodedImageFormat, FontMgr, Surface,
};
use style::Style;

use crate::ArgbExt;

pub mod elements;
pub mod language;
pub mod layout;
pub mod node;
pub mod style;

struct Context {
    size: Size,
    collection: FontCollection,
}

impl Context {
    pub fn new(width: f32, height: f32) -> Self {
        let mut collection = FontCollection::new();

        collection.set_default_font_manager(Some(FontMgr::default()), None);

        Self {
            size: Size::new(width, height),
            collection,
        }
    }

    pub fn create_paragraph(&self, style: &Style, text: &TextElement) -> Paragraph {
        let font_style = FontStyle::new(
            style.font_weight.into(),
            Width::NORMAL,
            style.font_slant.into(),
        );

        let mut text_style = TextStyle::new();

        text_style.set_font_families(&[style.font_family.family.as_str()]);
        text_style.set_font_size(style.font_size.size);
        text_style.set_font_style(font_style);
        text_style.set_color(style.color.unwrap_or(Argb::new(255, 0, 0, 0)).as_color());

        let mut paragraph_style = ParagraphStyle::new();

        paragraph_style.set_text_style(&text_style);

        let mut builder = ParagraphBuilder::new(&paragraph_style, &self.collection);

        builder.add_text(text.data.as_str());

        builder.build()
    }
}

pub struct RenderContext {
    surface: Surface,
    context: Context,
}

impl RenderContext {
    #[must_use]
    pub fn new(width: i32, height: i32) -> Option<Self> {
        Some(Self {
            surface: surfaces::raster_n32_premul((width, height))?,
            context: Context::new(width as f32, height as f32),
        })
    }

    pub fn render(&mut self, node: &Node) {
        let canvas = self.surface.canvas();

        let measure_node = node.measure(&self.context, &Rect::from_size(self.context.size));

        println!("{measure_node:#?}");

        node.draw(canvas, &self.context, measure_node);
    }

    pub fn encode(&mut self) -> Option<Vec<u8>> {
        let image = self.surface.image_snapshot();

        image
            .encode(None, EncodedImageFormat::PNG, 100)
            .map(|data| data.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use material_colors::{color::Argb, theme::ThemeBuilder};

    use super::{
        language::{lexer::Lexer, parser::Parser, program::parse_node},
        node::Node,
        style::{font::FontWeight, position::Position},
        RenderContext,
    };

    #[test]
    fn test_measure() {
        let root = Node::column()
            .padding(8.0)
            .background(Argb::new(255, 255, 0, 0))
            .spacing(8.0)
            .width_p(50.0)
            .child(
                Node::row()
                    .height(200.0)
                    .width_p(100.0)
                    .background(Argb::new(255, 0, 255, 0))
                    .corner_radius(50.0)
                    .child(
                        Node::row()
                            .height_p(50.0)
                            .width_p(50.0)
                            .background(Argb::new(255, 0, 0, 255))
                            .corner_radius(50.0)
                            .build(),
                    )
                    .build(),
            )
            .child(
                Node::image(
                    std::fs::read("/home/aiving/Pictures/e4c2e8651098c6989b2e47b1b910e788.png")
                        .unwrap(),
                        "sussy"
                )
                .size(200.0)
                .position(Position::Absolute)
                .x(100.0)
                .y(50.0)
                .background(Argb::new(255, 0, 255, 0))
                .corner_radius(50.0)
                .build(),
            )
            .child(
                Node::row()
                    .size(200.0)
                    .background(Argb::new(255, 0, 255, 0))
                    .corner_radius(50.0)
                    .build(),
            )
            .child(
                Node::text("АЛО ДЕБИЛЫ ВАШИХ")
                    .background(Argb::new(255, 0, 255, 0))
                    .color(Argb::new(255, 0, 0, 255))
                    .font_family("Source Code Pro")
                    .font_size(32.0)
                    .font_weight(FontWeight::Bold)
                    .build(),
            )
            .child(
                Node::text("АЛО ДЕБИЛЫ Я ВАШИХ МАТЕРЕЙ")
                    .background(Argb::new(255, 0, 255, 0))
                    .color(Argb::new(255, 0, 0, 255))
                    .font_family("Source Code Pro")
                    .font_size(32.0)
                    .font_weight(FontWeight::Bold)
                    .build(),
            )
            .build();

        let mut context = RenderContext::new(1000, 1000).unwrap();

        context.render(&root);

        if let Some(data) = context.encode() {
            std::fs::write("/home/aiving/test.png", &data).unwrap();
        }
    }

    #[test]
    fn test_masonry() {
        let theme = ThemeBuilder::with_source(Argb::from_u32(0xFF00FF00))
            .build()
            .schemes
            .dark;
        let heights = [200.0, 100.0, 150.0, 175.0, 75.0];
        let root = Node::masonry(100.0)
            .padding(8.0)
            .background(theme.surface_container_high)
            .spacing(8.0)
            .width_p(100.0)
            .height_p(100.0)
            .children(
                (0..25)
                    .map(|index| {
                        Node::column()
                            .height(heights[index % 5])
                            .width_p(100.0)
                            .background(theme.surface_bright)
                            .corner_radius(12.0)
                            .padding(4.0)
                            .children(vec![Node::column()
                                .background(theme.primary_container)
                                .size_p(100.0)
                                .corner_radius(12.0)
                                .build()])
                            .build()
                    })
                    .collect(),
            )
            .build();

        let mut context = RenderContext::new(1000, 1000).unwrap();

        context.render(&root);

        if let Some(data) = context.encode() {
            std::fs::write("/home/aiving/test.png", &data).unwrap();
        }
    }

    #[test]
    fn test_a() {
        let mut parser = Parser::new(Lexer::parse(
            r#"Column {
  padding: 96 96 96 0,
  spacing: 48,
  background: theme.surface_container,
  width: 100%,
  height: 100%,

  Row {
    spacing: 48,

    Column {
      padding: 320 0 48 0,
      background: theme.surface_container_highest,
      corner-radius: 48,

      Text("{user_info.user_info.nickname}") {
        background: theme.primary_container,
        color: theme.primary,
        font-size: 96,
      }
    }

    Column {
      padding: 0 48,
      background: theme.surface_container_highest,
      corner-radius: 48,

      Text("{user_info.user_info.introduce} a") {
        color: theme.primary,
        font-size: 96,
      }
    }
  }

  Column {
    background: theme.surface_container_highest,
    width: 100%,
    height: 100%,
    corner-radius: 72 72 0 0,
    padding: 12 120 12 12,

    Text("{characters.list[0].weapon.name} a") {
      background: theme.primary_container,
      color: theme.primary,
      font-weight: bold,
      font-size: 80
    }

    Text("{characters.list[0].weapon.desc}") {
      color: theme.primary,
      font-size: 72
    }
  }

  Image("") {
    position: absolute,
    corner-radius: 72,
    width: 320,
    height: 320,
    x: 48,
    y: 48
  }
}"#,
        ));

        let theme = ThemeBuilder::with_source(Argb::from_u32(0xFF00FF00))
            .build()
            .schemes
            .dark
            .into_iter()
            .collect();

        let root = parse_node(&mut parser, &theme, None).unwrap();

        let mut context = RenderContext::new(1920, 1080).unwrap();

        context.render(&root);

        if let Some(data) = context.encode() {
            std::fs::write("/home/aiving/test.png", &data).unwrap();
        }
    }
}
