use super::{owned_section::*, *};
use ordered_float::OrderedFloat;
use std::{borrow::Cow, f32, hash::*};

pub type Color = [f32; 4];

/// An object that contains all the info to render a varied section of text. That is one including
/// many parts with differing fonts/scales/colors bowing to a single layout.
///
/// # Example
/// ```
/// use glyph_brush::{HorizontalAlign, Layout, Section, Text};
///
/// let section = Section::default()
///     .add_text(Text::new("The last word was ").with_color([0.0, 0.0, 0.0, 1.0]))
///     .add_text(Text::new("RED").with_color([1.0, 0.0, 0.0, 1.0]))
///     .with_layout(Layout::default().h_align(HorizontalAlign::Center));
/// ```
///
/// # Extra
/// Extra text section data is stored in a generic type, default `Extra`. To use custom
/// `extra` data ensure your custom type implements `Debug`, `Clone`, `PartialEq` & `Hash`.
#[derive(Debug, Clone, PartialEq)]
pub struct Section<'a, X = Extra> {
    /// Position on screen to render text, in pixels from top-left. Defaults to (0, 0).
    pub screen_position: (f32, f32),
    /// Max (width, height) bounds, in pixels from top-left. Defaults to unbounded.
    pub bounds: (f32, f32),
    /// Built in layout, can be overridden with custom layout logic
    /// see [`queue_custom_layout`](struct.GlyphBrush.html#method.queue_custom_layout)
    pub layout: Layout<BuiltInLineBreaker>,
    /// Text to render, rendered next to one another according the layout.
    pub text: Vec<Text<'a, X>>,
}

impl<X: Clone> Section<'_, X> {
    #[inline]
    pub(crate) fn clone_extras(&self) -> Vec<X> {
        self.text.iter().map(|t| &t.extra).cloned().collect()
    }
}

impl Default for Section<'static, Extra> {
    #[inline]
    fn default() -> Self {
        Section::new()
    }
}

impl<'a, X> Section<'a, X> {
    #[inline]
    pub fn new() -> Self {
        Self {
            screen_position: (0.0, 0.0),
            bounds: (f32::INFINITY, f32::INFINITY),
            layout: Layout::default(),
            text: vec![],
        }
    }
}

impl<'a, X> Section<'a, X> {
    #[inline]
    pub fn with_screen_position<P: Into<(f32, f32)>>(mut self, position: P) -> Self {
        self.screen_position = position.into();
        self
    }

    #[inline]
    pub fn with_bounds<P: Into<(f32, f32)>>(mut self, bounds: P) -> Self {
        self.bounds = bounds.into();
        self
    }

    #[inline]
    pub fn with_layout<L: Into<Layout<BuiltInLineBreaker>>>(mut self, layout: L) -> Self {
        self.layout = layout.into();
        self
    }

    #[inline]
    pub fn add_text<T: Into<Text<'a, X>>>(mut self, text: T) -> Self {
        self.text.push(text.into());
        self
    }

    #[inline]
    pub fn with_text<X2>(self, text: Vec<Text<'_, X2>>) -> Section<'_, X2> {
        Section {
            text,
            screen_position: self.screen_position,
            bounds: self.bounds,
            layout: self.layout,
        }
    }
}

impl<'a, X: Clone> From<Section<'a, X>> for Cow<'a, Section<'a, X>> {
    #[inline]
    fn from(owned: Section<'a, X>) -> Self {
        Cow::Owned(owned)
    }
}

impl<'a, 'b, X: Clone> From<&'b Section<'a, X>> for Cow<'b, Section<'a, X>> {
    #[inline]
    fn from(owned: &'b Section<'a, X>) -> Self {
        Cow::Borrowed(owned)
    }
}

impl<X: Hash> Hash for Section<'_, X> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Section {
            screen_position: (screen_x, screen_y),
            bounds: (bound_w, bound_h),
            layout,
            ref text,
        } = *self;

        let ord_floats: &[OrderedFloat<_>] = &[
            screen_x.into(),
            screen_y.into(),
            bound_w.into(),
            bound_h.into(),
        ];

        layout.hash(state);

        hash_section_text(state, text);

        ord_floats.hash(state);
    }
}

/// `SectionText` + extra.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Text<'a, X = Extra> {
    /// Text to render.
    pub text: &'a str,
    /// Pixel scale of text. Defaults to 16.
    pub scale: PxScale,
    /// Font id to use for this section.
    ///
    /// It must be a valid id in the `FontMap` used for layout calls.
    /// The default `FontId(0)` should always be valid.
    pub font_id: FontId,
    /// Extra stuff for vertex generation.
    pub extra: X,
}

impl<X: Default> Default for Text<'static, X> {
    #[inline]
    fn default() -> Self {
        Self {
            text: "",
            scale: PxScale::from(16.0),
            font_id: <_>::default(),
            extra: <_>::default(),
        }
    }
}

impl<'a, X> Text<'a, X> {
    #[inline]
    pub fn with_text(self, text: &str) -> Text<'_, X> {
        Text {
            text,
            scale: self.scale,
            font_id: self.font_id,
            extra: self.extra,
        }
    }

    #[inline]
    pub fn with_scale<S: Into<PxScale>>(mut self, scale: S) -> Self {
        self.scale = scale.into();
        self
    }

    #[inline]
    pub fn with_font_id<F: Into<FontId>>(mut self, font_id: F) -> Self {
        self.font_id = font_id.into();
        self
    }

    #[inline]
    pub fn with_extra<X2>(self, extra: X2) -> Text<'a, X2> {
        Text {
            text: self.text,
            scale: self.scale,
            font_id: self.font_id,
            extra,
        }
    }
}

impl<'a> Text<'a, Extra> {
    #[inline]
    pub fn new(text: &'a str) -> Self {
        Text::default().with_text(text)
    }

    #[inline]
    pub fn with_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.extra.color = color.into();
        self
    }

    #[inline]
    pub fn with_outline_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.extra.outline_color = color.into();
        self
    }

    #[inline]
    pub fn with_z<Z: Into<f32>>(mut self, z: Z) -> Self {
        self.extra.z = z.into();
        self
    }
}

impl<X> ToSectionText for Text<'_, X> {
    #[inline]
    fn to_section_text(&self) -> SectionText<'_> {
        SectionText {
            text: self.text,
            scale: self.scale,
            font_id: self.font_id,
        }
    }
}

#[inline]
fn hash_section_text<X: Hash, H: Hasher>(state: &mut H, text: &[Text<'_, X>]) {
    for t in text {
        let Text {
            text,
            scale,
            font_id,
            ref extra,
        } = *t;

        let ord_floats: [OrderedFloat<_>; 2] = [scale.x.into(), scale.y.into()];

        (text, font_id, extra, ord_floats).hash(state);
    }
}

impl<'text, X: Clone> Section<'text, X> {
    pub fn to_owned(&self) -> OwnedSection<X> {
        OwnedSection {
            screen_position: self.screen_position,
            bounds: self.bounds,
            layout: self.layout,
            text: self.text.iter().map(OwnedText::from).collect(),
        }
    }

    #[inline]
    pub(crate) fn to_hashable_parts(&self) -> HashableSectionParts<'_, X> {
        let Section {
            screen_position: (screen_x, screen_y),
            bounds: (bound_w, bound_h),
            ref text,
            layout: _,
        } = *self;

        let geometry = [
            screen_x.into(),
            screen_y.into(),
            bound_w.into(),
            bound_h.into(),
        ];

        HashableSectionParts { geometry, text }
    }
}

impl<X> From<&Section<'_, X>> for SectionGeometry {
    #[inline]
    fn from(section: &Section<'_, X>) -> Self {
        Self {
            bounds: section.bounds,
            screen_position: section.screen_position,
        }
    }
}

pub(crate) struct HashableSectionParts<'a, X> {
    geometry: [OrderedFloat<f32>; 4],
    text: &'a [Text<'a, X>],
}

impl<X: Hash> HashableSectionParts<'_, X> {
    #[inline]
    pub fn hash_geometry<H: Hasher>(&self, state: &mut H) {
        self.geometry.hash(state);
    }

    #[inline]
    pub fn hash_text_no_extra<H: Hasher>(&self, state: &mut H) {
        for t in self.text {
            let Text {
                text,
                scale,
                font_id,
                ..
            } = *t;

            let ord_floats: &[OrderedFloat<_>] = &[scale.x.into(), scale.y.into()];

            (text, font_id, ord_floats).hash(state);
        }
    }

    #[inline]
    pub fn hash_extra<H: Hasher>(&self, state: &mut H) {
        self.text.iter().for_each(|t| t.extra.hash(state));
    }
}
