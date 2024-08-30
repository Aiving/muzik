use muzui_geometry::Rect;

#[derive(Default, Debug, Clone, Copy)]
pub struct Thickness {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Thickness {
    const fn all(value: f32) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    const fn axis(vertical: f32, horizontal: f32) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }

    const fn custom(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}

impl From<Thickness> for Rect<f32> {
    fn from(value: Thickness) -> Self {
        Self::from_xywh(value.left, value.top, value.right, value.bottom)
    }
}

impl From<f32> for Thickness {
    fn from(value: f32) -> Self {
        Self::all(value)
    }
}

impl From<[f32; 2]> for Thickness {
    fn from([vertical, horizontal]: [f32; 2]) -> Self {
        Self::axis(vertical, horizontal)
    }
}

impl From<[f32; 4]> for Thickness {
    fn from([left, top, right, bottom]: [f32; 4]) -> Self {
        Self::custom(left, top, right, bottom)
    }
}

// impl Parse for Thickness {
//     fn parse(parser: &mut Parser) -> Result<Self> {
//         match [
//             parser.consume_map(Token::try_as_f32),
//             parser.consume_map(Token::try_as_f32),
//             parser.consume_map(Token::try_as_f32),
//             parser.consume_map(Token::try_as_f32),
//         ] {
//             [Ok(x), Ok(y), Ok(z), Ok(w)] => Ok([x, y, z, w].into()),
//             [Ok(x), Ok(y), ..] => Ok([x, y].into()),
//             [Ok(x), ..] => Ok(x.into()),
//             _ => Err(ParseError::new("failed to parse thickness")),
//         }
//     }
// }
