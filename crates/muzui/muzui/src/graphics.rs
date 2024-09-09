use skia_safe::{
    font_style::{Slant, Weight, Width}, surfaces, textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle}, EncodedImageFormat, FontMgr, FontStyle, Image, RCHandle, Surface
};

use crate::{
    layout::{Measurer, Rect},
    styling::{Color, FontSlant, FontWeight, Style},
    Node,
};

pub struct Context {
    pub bounds: Rect,
    collection: FontCollection,
}

impl Context {
    pub fn new(width: f32, height: f32) -> Self {
        let mut collection = FontCollection::new();

        collection.set_default_font_manager(Some(FontMgr::default()), None);

        Self {
            bounds: Rect::from_xywh(0.0, 0.0, width, height),
            collection,
        }
    }

    pub fn create_paragraph<T: AsRef<str>>(&self, style: &Style, text: T) -> Paragraph {
        let font_style = FontStyle::new(
            match style.font_weight {
                FontWeight::Light => Weight::LIGHT,
                FontWeight::Normal => Weight::NORMAL,
                FontWeight::Bold => Weight::BOLD,
            },
            Width::NORMAL,
            match style.font_slant {
                FontSlant::Upright => Slant::Upright,
                FontSlant::Italic => Slant::Italic,
                FontSlant::Oblique => Slant::Oblique,
            },
        );

        let mut text_style = TextStyle::new();

        text_style.set_font_families(&[style.font_family.family.as_str()]);
        text_style.set_font_size(style.font_size.size);
        text_style.set_font_style(font_style);
        text_style.set_color(skia_safe::Color::new(
            style.color.unwrap_or(Color::from_rgb(0, 0, 0)).as_u32(),
        ));

        let mut paragraph_style = ParagraphStyle::new();

        paragraph_style.set_text_style(&text_style);

        let mut builder = ParagraphBuilder::new(&paragraph_style, &self.collection);

        builder.add_text(text.as_ref());

        builder.build()
    }
}

pub struct RenderContext {
    surface: Surface,
    context: Context,
}

impl RenderContext {
    #[must_use]
    pub fn new(width: i16, height: i16) -> Option<Self> {
        Some(Self {
            surface: surfaces::raster_n32_premul((i32::from(width), i32::from(height)))?,
            context: Context::new(f32::from(width), f32::from(height)),
        })
    }

    pub fn render(&mut self, node: &Node) {
        let canvas = self.surface.canvas();

        let measure_node = node.measure(&self.context, &self.context.bounds);

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
