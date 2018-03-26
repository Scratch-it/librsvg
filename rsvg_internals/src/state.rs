use cairo::{self, MatrixTrait};
use glib::translate::*;
use glib_sys;
use libc;
use pango;
use pango_sys;

use attributes::Attribute;
use color::{Color, ColorSpec};
use error::*;
use length::{RsvgLength, StrokeDasharray};
use node::RsvgNode;
use opacity::{Opacity, OpacitySpec};
use paint_server::PaintServer;
use parsers::{Parse, ParseError};
use util::utf8_cstr;

pub enum RsvgState {}

/// Holds the state of CSS properties
///
/// This is used for various purposes:
///
/// * Immutably, to store the attributes of element nodes after parsing.
/// * Mutably, during cascading/rendering.
///
/// Each property should have its own data type, and implement
/// `Default` and `parsers::Parse`.
///
/// If a property is `None`, is means it was not specified and must be
/// inherited from the parent state, or in the end the caller can
/// `.unwrap_or_default()` to get the default value for the property.
#[derive(Clone)]
pub struct State {
    pub affine: cairo::Matrix,

    pub join: Option<StrokeLinejoin>,

    pub cap: Option<StrokeLinecap>,

    pub fill_rule: Option<FillRule>,

    pub xml_space: Option<XmlSpace>,

    pub text_anchor: Option<TextAnchor>,
}

impl State {
    fn new() -> State {
        State {
            affine: cairo::Matrix::identity(),

            join: Default::default(),

            cap: Default::default(),

            fill_rule: Default::default(),

            xml_space: Default::default(),

            text_anchor: Default::default(),
        }
    }

    fn parse_style_pair(&mut self, attr: Attribute, value: &str) -> Result<(), AttributeError> {
        match attr {
            Attribute::StrokeLinejoin => match StrokeLinejoin::parse(value, ())? {
                StrokeLinejoin::Inherit => {
                    self.join = None;
                }

                v => {
                    self.join = Some(v);
                }
            },

            Attribute::StrokeLinecap => match StrokeLinecap::parse(value, ())? {
                StrokeLinecap::Inherit => {
                    self.cap = None;
                }

                v => {
                    self.cap = Some(v);
                }
            },

            Attribute::FillRule => match FillRule::parse(value, ())? {
                FillRule::Inherit => {
                    self.fill_rule = None;
                }

                v => {
                    self.fill_rule = Some(v);
                }
            },

            Attribute::XmlSpace => {
                self.xml_space = Some(XmlSpace::parse(value, ())?);
            }

            Attribute::TextAnchor => match TextAnchor::parse(value, ())? {
                TextAnchor::Inherit => {
                    self.text_anchor = None;
                }

                v => {
                    self.text_anchor = Some(v);
                }
            },

            _ => {
                // Maybe it's an attribute not parsed here, but in the
                // node implementations.
            }
        }

        Ok(())
    }
}

// Keep in sync with rsvg-styles.h:UnicodeBidi
// FIXME: these are not constructed in the Rust code yet, but they are in C.  Remove this
// when that code is moved to Rust.
#[allow(dead_code)]
#[repr(C)]
pub enum UnicodeBidi {
    Normal,
    Embed,
    Override,
}

// Keep in sync with rsvg-styles.h:TextDecoration
#[repr(C)]
#[derive(Copy, Clone)]
struct TextDecoration {
    overline: glib_sys::gboolean,
    underline: glib_sys::gboolean,
    strike: glib_sys::gboolean,
}

pub struct FontDecor {
    pub overline: bool,
    pub underline: bool,
    pub strike: bool,
}

impl From<TextDecoration> for FontDecor {
    fn from(td: TextDecoration) -> FontDecor {
        FontDecor {
            overline: from_glib(td.overline),
            underline: from_glib(td.underline),
            strike: from_glib(td.strike),
        }
    }
}

#[allow(improper_ctypes)]
extern "C" {
    fn rsvg_state_new() -> *mut RsvgState;
    fn rsvg_state_free(state: *mut RsvgState);
    fn rsvg_state_reinit(state: *mut RsvgState);
    fn rsvg_state_reconstruct(state: *mut RsvgState, node: *const RsvgNode);
    fn rsvg_state_is_overflow(state: *const RsvgState) -> glib_sys::gboolean;
    fn rsvg_state_has_overflow(state: *const RsvgState) -> glib_sys::gboolean;
    fn rsvg_state_get_cond_true(state: *const RsvgState) -> glib_sys::gboolean;
    fn rsvg_state_set_cond_true(state: *const RsvgState, cond_true: glib_sys::gboolean);
    fn rsvg_state_get_stop_color(state: *const RsvgState) -> *const ColorSpec;
    fn rsvg_state_get_stop_opacity(state: *const RsvgState) -> *const OpacitySpec;
    fn rsvg_state_get_stroke_dasharray(state: *const RsvgState) -> *const StrokeDasharray;
    fn rsvg_state_get_dash_offset(state: *const RsvgState) -> RsvgLength;
    fn rsvg_state_get_current_color(state: *const RsvgState) -> u32;
    fn rsvg_state_get_shape_rendering_type(state: *const RsvgState) -> cairo::Antialias;
    fn rsvg_state_get_text_rendering_type(state: *const RsvgState) -> cairo::Antialias;
    fn rsvg_state_get_stroke(state: *const RsvgState) -> *const PaintServer;
    fn rsvg_state_get_stroke_opacity(state: *const RsvgState) -> u8;
    fn rsvg_state_get_stroke_width(state: *const RsvgState) -> RsvgLength;
    fn rsvg_state_get_miter_limit(state: *const RsvgState) -> f64;
    fn rsvg_state_get_language(state: *const RsvgState) -> *const libc::c_char;
    fn rsvg_state_get_unicode_bidi(state: *const RsvgState) -> UnicodeBidi;
    fn rsvg_state_get_text_dir(state: *const RsvgState) -> pango_sys::PangoDirection;
    fn rsvg_state_get_text_gravity(state: *const RsvgState) -> pango_sys::PangoGravity;
    fn rsvg_state_get_font_family(state: *const RsvgState) -> *const libc::c_char;
    fn rsvg_state_get_font_style(state: *const RsvgState) -> pango_sys::PangoStyle;
    fn rsvg_state_get_font_variant(state: *const RsvgState) -> pango_sys::PangoVariant;
    fn rsvg_state_get_font_weight(state: *const RsvgState) -> pango_sys::PangoWeight;
    fn rsvg_state_get_font_stretch(state: *const RsvgState) -> pango_sys::PangoStretch;
    fn rsvg_state_get_letter_spacing(state: *const RsvgState) -> RsvgLength;
    fn rsvg_state_get_font_decor(state: *const RsvgState) -> *const TextDecoration;
    fn rsvg_state_get_clip_rule(state: *const RsvgState) -> cairo::FillRule;
    fn rsvg_state_get_fill(state: *const RsvgState) -> *const PaintServer;
    fn rsvg_state_get_fill_opacity(state: *const RsvgState) -> u8;

    fn rsvg_state_get_state_rust(state: *const RsvgState) -> *mut State;
}

pub fn new() -> *mut RsvgState {
    unsafe { rsvg_state_new() }
}

pub fn free(state: *mut RsvgState) {
    unsafe {
        rsvg_state_free(state);
    }
}

pub fn reinit(state: *mut RsvgState) {
    unsafe {
        rsvg_state_reinit(state);
    }
}

pub fn reconstruct(state: *mut RsvgState, node: *const RsvgNode) {
    unsafe {
        rsvg_state_reconstruct(state, node);
    }
}

pub fn is_overflow(state: *const RsvgState) -> bool {
    unsafe { from_glib(rsvg_state_is_overflow(state)) }
}

pub fn has_overflow(state: *const RsvgState) -> bool {
    unsafe { from_glib(rsvg_state_has_overflow(state)) }
}

pub fn get_cond_true(state: *const RsvgState) -> bool {
    unsafe { from_glib(rsvg_state_get_cond_true(state)) }
}

pub fn set_cond_true(state: *const RsvgState, cond_true: bool) {
    unsafe {
        rsvg_state_set_cond_true(state, cond_true.to_glib());
    }
}

pub fn get_stop_color(state: *const RsvgState) -> Result<Option<Color>, AttributeError> {
    unsafe {
        let spec_ptr = rsvg_state_get_stop_color(state);

        if spec_ptr.is_null() {
            Ok(None)
        } else {
            Color::from_color_spec(&*spec_ptr).map(Some)
        }
    }
}

pub fn get_stop_opacity(state: *const RsvgState) -> Result<Option<Opacity>, AttributeError> {
    unsafe {
        let opacity_ptr = rsvg_state_get_stop_opacity(state);

        if opacity_ptr.is_null() {
            Ok(None)
        } else {
            Opacity::from_opacity_spec(&*opacity_ptr).map(Some)
        }
    }
}

pub fn get_stroke_dasharray<'a>(state: *const RsvgState) -> Option<&'a StrokeDasharray> {
    let dash = unsafe { rsvg_state_get_stroke_dasharray(state) };

    if dash.is_null() {
        None
    } else {
        unsafe { Some(&*dash) }
    }
}

pub fn get_dash_offset(state: *const RsvgState) -> RsvgLength {
    unsafe { rsvg_state_get_dash_offset(state) }
}

pub fn get_current_color(state: *const RsvgState) -> Color {
    let argb = unsafe { rsvg_state_get_current_color(state) };

    Color::from(argb)
}

pub fn get_shape_rendering_type(state: *const RsvgState) -> cairo::Antialias {
    unsafe { rsvg_state_get_shape_rendering_type(state) }
}

pub fn get_text_rendering_type(state: *const RsvgState) -> cairo::Antialias {
    unsafe { rsvg_state_get_text_rendering_type(state) }
}

pub fn get_stroke<'a>(state: *const RsvgState) -> Option<&'a PaintServer> {
    unsafe {
        let ps = rsvg_state_get_stroke(state);

        if ps.is_null() {
            None
        } else {
            Some(&*ps)
        }
    }
}

pub fn get_stroke_opacity(state: *const RsvgState) -> u8 {
    unsafe { rsvg_state_get_stroke_opacity(state) }
}

pub fn get_stroke_width(state: *const RsvgState) -> RsvgLength {
    unsafe { rsvg_state_get_stroke_width(state) }
}

pub fn get_miter_limit(state: *const RsvgState) -> f64 {
    unsafe { rsvg_state_get_miter_limit(state) }
}

pub fn get_language(state: *const RsvgState) -> Option<String> {
    unsafe { from_glib_none(rsvg_state_get_language(state)) }
}

pub fn get_unicode_bidi(state: *const RsvgState) -> UnicodeBidi {
    unsafe { rsvg_state_get_unicode_bidi(state) }
}

pub fn get_text_dir(state: *const RsvgState) -> pango::Direction {
    unsafe { from_glib(rsvg_state_get_text_dir(state)) }
}

pub fn get_text_gravity(state: *const RsvgState) -> pango::Gravity {
    unsafe { from_glib(rsvg_state_get_text_gravity(state)) }
}

pub fn get_font_family(state: *const RsvgState) -> Option<String> {
    unsafe { from_glib_none(rsvg_state_get_font_family(state)) }
}

pub fn get_font_style(state: *const RsvgState) -> pango::Style {
    unsafe { from_glib(rsvg_state_get_font_style(state)) }
}

pub fn get_font_variant(state: *const RsvgState) -> pango::Variant {
    unsafe { from_glib(rsvg_state_get_font_variant(state)) }
}

pub fn get_font_weight(state: *const RsvgState) -> pango::Weight {
    unsafe { from_glib(rsvg_state_get_font_weight(state)) }
}

pub fn get_font_stretch(state: *const RsvgState) -> pango::Stretch {
    unsafe { from_glib(rsvg_state_get_font_stretch(state)) }
}

pub fn get_letter_spacing(state: *const RsvgState) -> RsvgLength {
    unsafe { rsvg_state_get_letter_spacing(state) }
}

pub fn get_font_decor(state: *const RsvgState) -> Option<FontDecor> {
    unsafe {
        let td = rsvg_state_get_font_decor(state);
        if td.is_null() {
            None
        } else {
            Some(FontDecor::from(*td))
        }
    }
}

pub fn get_clip_rule(state: *const RsvgState) -> cairo::FillRule {
    unsafe { rsvg_state_get_clip_rule(state) }
}

pub fn get_fill<'a>(state: *const RsvgState) -> Option<&'a PaintServer> {
    unsafe {
        let ps = rsvg_state_get_fill(state);

        if ps.is_null() {
            None
        } else {
            Some(&*ps)
        }
    }
}

pub fn get_fill_opacity(state: *const RsvgState) -> u8 {
    unsafe { rsvg_state_get_fill_opacity(state) }
}

pub fn get_state_rust<'a>(state: *const RsvgState) -> &'a mut State {
    unsafe { &mut *rsvg_state_get_state_rust(state) }
}

// StrokeLineJoin ----------------------------------------

make_ident_property!(
    StrokeLinejoin,
    default: Miter,

    "miter" => Miter,
    "round" => Round,
    "bevel" => Bevel,
    "inherit" => Inherit,
);

// StrokeLinecap ----------------------------------------

make_ident_property!(
    StrokeLinecap,
    default: Butt,

    "butt" => Butt,
    "round" => Round,
    "square" => Square,
    "inherit" => Inherit,
);

// FillRule ----------------------------------------

make_ident_property!(
    FillRule,
    default: NonZero,

    "nonzero" => NonZero,
    "evenodd" => EvenOdd,
    "inherit" => Inherit,
);

// XmlSpace ----------------------------------------

make_ident_property!(
    XmlSpace,
    default: Default,

    "default" => Default,
    "preserve" => Preserve,
);

// TextAnchor --------------------------------------

make_ident_property!(
    TextAnchor,
    default: Start,

    "start" => Start,
    "middle" => Middle,
    "end" => End,
    "inherit" => Inherit,
);

// Rust State API for consumption from C ----------------------------------------

#[no_mangle]
pub extern "C" fn rsvg_state_rust_new() -> *mut State {
    Box::into_raw(Box::new(State::new()))
}

#[no_mangle]
pub extern "C" fn rsvg_state_rust_free(state: *mut State) {
    assert!(!state.is_null());

    unsafe {
        Box::from_raw(state);
    }
}

#[no_mangle]
pub extern "C" fn rsvg_state_rust_clone(state: *const State) -> *mut State {
    assert!(!state.is_null());

    unsafe { Box::into_raw(Box::new((*state).clone())) }
}

#[no_mangle]
pub extern "C" fn rsvg_state_rust_parse_style_pair(
    state: *mut State,
    attr: Attribute,
    value: *const libc::c_char,
) -> glib_sys::gboolean {
    assert!(!state.is_null());
    assert!(!value.is_null());

    let state = unsafe { &mut *state };
    let value = unsafe { utf8_cstr(value) };

    match state.parse_style_pair(attr, value) {
        Ok(_) => true.to_glib(),
        Err(_) => false.to_glib(),
    }
}

fn inherit_from_src(
    inherit_fn: extern "C" fn(glib_sys::gboolean, glib_sys::gboolean) -> glib_sys::gboolean,
    dst: bool,
    src: bool,
) -> bool {
    from_glib(inherit_fn(dst.to_glib(), src.to_glib()))
}

#[no_mangle]
pub extern "C" fn rsvg_state_rust_inherit_run(
    dst: *mut State,
    src: *const State,
    inherit_fn: extern "C" fn(glib_sys::gboolean, glib_sys::gboolean) -> glib_sys::gboolean,
) {
    assert!(!dst.is_null());
    assert!(!src.is_null());

    unsafe {
        let dst = &mut *dst;
        let src = &*src;

        if inherit_from_src(inherit_fn, dst.join.is_some(), src.join.is_some()) {
            dst.join = src.join;
        }

        if inherit_from_src(inherit_fn, dst.cap.is_some(), src.cap.is_some()) {
            dst.cap = src.cap;
        }

        if inherit_from_src(inherit_fn, dst.fill_rule.is_some(), src.fill_rule.is_some()) {
            dst.fill_rule = src.fill_rule;
        }

        if inherit_from_src(inherit_fn, dst.xml_space.is_some(), src.xml_space.is_some()) {
            dst.xml_space = src.xml_space;
        }

        if inherit_from_src(
            inherit_fn,
            dst.text_anchor.is_some(),
            src.text_anchor.is_some(),
        ) {
            dst.text_anchor = src.text_anchor;
        }
    }
}

#[no_mangle]
pub extern "C" fn rsvg_state_rust_get_affine(state: *const State) -> cairo::Matrix {
    unsafe {
        let state = &*state;
        state.affine
    }
}

#[no_mangle]
pub extern "C" fn rsvg_state_rust_set_affine(state: *mut State, affine: cairo::Matrix) {
    unsafe {
        let state = &mut *state;
        state.affine = affine;
    }
}
