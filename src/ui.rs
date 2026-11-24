//! FLTK user interface: custom knobs / sliders / views, preset browser, on-screen + computer keyboard.
//! All widgets read parameter values straight from the lock-free store when they draw, so a preset
//! change (or MIDI/automation) only needs a redraw.

use crate::params::*;
use crate::presets::{self, Preset};
use crate::shared::{Event as SynthEvent, Shared, SCOPE_LEN};
use crate::wavetable::WaveBank;
use crate::{audio, midi, APP_ID, APP_VERSION};
use fltk::{app, button, dialog, draw, draw::LineStyle, enums::*, frame, group, menu, prelude::*, widget, window};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

// ------------------------------------------------------------------ theme
const BG: u32 = 0x12141a;
const PANEL: u32 = 0x1b1f28;
const PANEL2: u32 = 0x262c39;
const EDGE: u32 = 0x303746;
const TXT: u32 = 0xc3cad8;
const DIM: u32 = 0x7a8396;
const ACC: u32 = 0x2fd0e6;
const ACC2: u32 = 0xf0a030;
const TRACK: u32 = 0x363e4e;
fn col(h: u32) -> Color { Color::from_hex(h) }

/// Parameters that belong to the *session*, not the patch: tempo, humanize, drum machine, looper.
/// They are kept when you browse presets (except "Beats" demos and files you load yourself).
fn is_global(i: usize) -> bool { let k = PARAMS[i].key; k == "BPM" || k.starts_with("DR_") || k.starts_with("LP_") || k.starts_with("HUM_") }

type Refresh = Rc<RefCell<Vec<Box<dyn Fn()>>>>;

#[derive(Clone)]
struct Ctx { sh: Arc<Shared>, bank: Arc<WaveBank>, refresh: Refresh }

#[derive(Clone, Copy, PartialEq)]
enum Style { Knob, VSlider, HSlider }

fn esc(s: &str) -> String { s.replace('&', "&&").replace('/', "\\/") }

// ------------------------------------------------------------------ small widgets

fn panel(x: i32, y: i32, w: i32, h: i32, title: &str) {
    let mut f = widget::Widget::new(x, y, w, h, "");
    let t = title.to_string();
    f.draw(move |f| {
        draw::set_draw_color(col(PANEL));
        draw::draw_rounded_rectf(f.x(), f.y(), f.w(), f.h(), 6);
        draw::set_draw_color(col(EDGE));
        draw::draw_rect(f.x() + 1, f.y() + 1, f.w() - 2, f.h() - 2);
        draw::set_font(Font::HelveticaBold, 11);
        draw::set_draw_color(col(ACC));
        draw::draw_text2(&t, f.x() + 8, f.y() + 3, f.w() - 16, 14, Align::Left | Align::Inside);
    });
}

fn label(x: i32, y: i32, w: i32, h: i32, text: &str, color: u32, size: i32) -> frame::Frame {
    let mut f = frame::Frame::new(x, y, w, h, "");
    f.set_label(text);
    f.set_label_color(col(color));
    f.set_label_size(size);
    f.set_align(Align::Left | Align::Inside);
    f
}

fn style_choice(ch: &mut menu::Choice) {
    ch.set_color(col(PANEL2));
    ch.set_text_color(col(TXT));
    ch.set_text_size(11);
    ch.set_frame(FrameType::FlatBox);
    ch.set_down_frame(FrameType::FlatBox);
    ch.clear_visible_focus();
}

fn style_button(b: &mut button::Button) {
    b.set_color(col(PANEL2));
    b.set_selection_color(col(ACC));
    b.set_label_color(col(TXT));
    b.set_label_size(11);
    b.set_frame(FrameType::FlatBox);
    b.set_down_frame(FrameType::FlatBox);
    b.clear_visible_focus();
}

fn choice(c: &Ctx, x: i32, y: i32, w: i32, h: i32, p: usize) -> menu::Choice {
    let mut ch = menu::Choice::new(x, y, w, h, "");
    style_choice(&mut ch);
    if let Kind::Enum(names) = PARAMS[p].kind { for n in names.iter() { ch.add_choice(&esc(n)); } }
    ch.set_value(c.sh.params.get(p) as i32);
    let sh = c.sh.clone();
    ch.set_callback(move |cc| sh.params.set(p, cc.value().max(0) as f32));
    let (sh2, ch2) = (c.sh.clone(), ch.clone());
    c.refresh.borrow_mut().push(Box::new(move || { let mut k = ch2.clone(); k.set_value(sh2.params.get(p) as i32); }));
    ch
}

fn toggle(c: &Ctx, x: i32, y: i32, w: i32, h: i32, p: usize, text: &str) -> widget::Widget {
    let mut wd = widget::Widget::new(x, y, w, h, "");
    let (sh, lab) = (c.sh.clone(), text.to_string());
    wd.draw(move |w| {
        let on = sh.params.get(p) >= 0.5;
        draw::set_draw_color(col(if on { ACC } else { PANEL2 }));
        draw::draw_rounded_rectf(w.x(), w.y(), w.w(), w.h(), 4);
        draw::set_font(Font::HelveticaBold, 10);
        draw::set_draw_color(col(if on { BG } else { DIM }));
        draw::draw_text2(&lab, w.x(), w.y(), w.w(), w.h(), Align::Center);
    });
    let sh = c.sh.clone();
    wd.handle(move |w, ev| match ev {
        Event::Push => { let v = if sh.params.get(p) >= 0.5 { 0.0 } else { 1.0 }; sh.params.set(p, v); w.redraw(); true }
        _ => false,
    });
    wd
}

/// Knob / vertical slider / horizontal slider bound to parameter `p`.
fn ctl(c: &Ctx, x: i32, y: i32, w: i32, h: i32, p: usize, text: &str, style: Style) -> widget::Widget {
    let mut wd = widget::Widget::new(x, y, w, h, "");
    let (sh, lab) = (c.sh.clone(), text.to_string());
    let dragging = Rc::new(Cell::new(false));
    let hover = Rc::new(Cell::new(false));
    let (dr1, hv1) = (dragging.clone(), hover.clone());
    wd.draw(move |w| {
        let (x, y, ww, hh) = (w.x(), w.y(), w.w(), w.h());
        let def = &PARAMS[p];
        let v = sh.params.get(p);
        let n = def.to_norm(v);
        draw::set_draw_color(col(PANEL));
        draw::draw_rectf(x, y, ww, hh);
        let active = dr1.get();
        let lab_h = if style == Style::HSlider { 0 } else { 13 };
        match style {
            Style::Knob => {
                let d = (ww - 8).min(hh - lab_h - 4).clamp(16, 38);
                let r = d / 2;
                let (cx, cy) = (x + ww / 2, y + (hh - lab_h) / 2);
                draw::draw_circle_fill(cx - r + 3, cy - r + 3, d - 6, col(PANEL2));
                draw::set_line_style(LineStyle::Solid | LineStyle::CapRound, 3);
                draw::set_draw_color(col(TRACK));
                draw::draw_arc(cx - r, cy - r, d, d, -45.0, 225.0);
                let ang = 225.0 - 270.0 * n as f64;
                let bipolar = def.min < 0.0 && def.max > 0.0 && def.kind == Kind::Lin;
                draw::set_draw_color(col(if active { 0x8af0ff } else { ACC }));
                if bipolar { let (a, b) = if ang >= 90.0 { (90.0, ang) } else { (ang, 90.0) }; draw::draw_arc(cx - r, cy - r, d, d, a, b); }
                else { draw::draw_arc(cx - r, cy - r, d, d, ang, 225.0); }
                draw::set_line_style(LineStyle::Solid, 2);
                draw::set_draw_color(col(TXT));
                let rad = ang.to_radians();
                let (ix, iy) = (cx as f64 + (r as f64 - 2.0) * rad.cos(), cy as f64 - (r as f64 - 2.0) * rad.sin());
                let (jx, jy) = (cx as f64 + (r as f64 * 0.35) * rad.cos(), cy as f64 - (r as f64 * 0.35) * rad.sin());
                draw::draw_line(jx as i32, jy as i32, ix as i32, iy as i32);
                draw::set_line_style(LineStyle::Solid, 0);
            }
            Style::VSlider => {
                let (cx, top, bot) = (x + ww / 2, y + 4, y + hh - lab_h - 3);
                draw::set_draw_color(col(TRACK));
                draw::draw_rectf(cx - 2, top, 4, bot - top);
                let ty = bot - ((bot - top) as f32 * n) as i32;
                draw::set_draw_color(col(ACC));
                draw::draw_rectf(cx - 2, ty, 4, bot - ty);
                draw::set_draw_color(col(if active { 0x8af0ff } else { TXT }));
                draw::draw_rounded_rectf(cx - 9, ty - 4, 18, 8, 3);
            }
            Style::HSlider => {
                let (left, right, cy) = (x + 6, x + ww - 6, y + hh / 2);
                draw::set_draw_color(col(TRACK));
                draw::draw_rectf(left, cy - 2, right - left, 4);
                let tx = left + ((right - left) as f32 * n) as i32;
                let zero = if def.min < 0.0 { left + (right - left) / 2 } else { left };
                draw::set_draw_color(col(ACC));
                draw::draw_rectf(zero.min(tx), cy - 2, (tx - zero).abs().max(1), 4);
                draw::set_draw_color(col(if active { 0x8af0ff } else { TXT }));
                draw::draw_rounded_rectf(tx - 4, cy - 8, 8, 16, 3);
                if active || hv1.get() {
                    draw::set_font(Font::Helvetica, 10);
                    draw::set_draw_color(col(ACC2));
                    draw::draw_text2(&def.format(v), x, y - 1, ww, 11, Align::Center);
                }
            }
        }
        if lab_h > 0 {
            draw::set_font(Font::Helvetica, 10);
            let (t, c) = if active || hv1.get() { (def.format(v), ACC2) } else { (lab.clone(), DIM) };
            draw::set_draw_color(col(c));
            draw::draw_text2(&t, x, y + hh - lab_h, ww, lab_h, Align::Center);
        }
    });

    let sh = c.sh.clone();
    let (start_n, start_x, start_y) = (Rc::new(Cell::new(0.0f32)), Rc::new(Cell::new(0i32)), Rc::new(Cell::new(0i32)));
    wd.handle(move |w, ev| {
        let def = &PARAMS[p];
        let discrete = def.is_discrete();
        let span = (def.max - def.min).max(1.0);
        match ev {
            Event::Push => {
                if app::event_button() != 1 { return false; }
                if app::event_clicks() { sh.params.set(p, def.def); w.redraw(); return true; }
                start_n.set(def.to_norm(sh.params.get(p)));
                start_x.set(app::event_x()); start_y.set(app::event_y());
                dragging.set(true);
                if style != Style::Knob { drag_abs(&sh, w, p, style); }
                w.redraw(); true
            }
            Event::Drag => {
                if !dragging.get() { return false; }
                match style {
                    Style::Knob => {
                        let dy = (start_y.get() - app::event_y()) as f32;
                        let fine = app::event_state().contains(EventState::Shift);
                        let per_px = if discrete { 1.0 / (10.0 * span) } else if fine { 1.0 / 1600.0 } else { 1.0 / 180.0 };
                        sh.params.set(p, def.from_norm(start_n.get() + dy * per_px));
                    }
                    _ => drag_abs(&sh, w, p, style),
                }
                w.redraw(); true
            }
            Event::Released => { dragging.set(false); w.redraw(); true }
            Event::MouseWheel => {
                let dy = -(app::event_dy_value() as f32);
                if dy == 0.0 { return false; }
                let cur = sh.params.get(p);
                if discrete { sh.params.set(p, cur + dy.signum()); }
                else {
                    let step = if app::event_state().contains(EventState::Shift) { 0.002 } else { 0.02 };
                    sh.params.set(p, def.from_norm(def.to_norm(cur) + dy.signum() * step));
                }
                w.redraw(); true
            }
            Event::Enter => { hover.set(true); w.redraw(); true }
            Event::Leave => { hover.set(false); w.redraw(); true }
            _ => false,
        }
    });
    wd
}

fn drag_abs(sh: &Shared, w: &widget::Widget, p: usize, style: Style) {
    let def = &PARAMS[p];
    let n = match style {
        Style::VSlider => { let (top, bot) = (w.y() + 4, w.y() + w.h() - 16); 1.0 - (app::event_y() - top) as f32 / (bot - top).max(1) as f32 }
        _ => { let (l, r) = (w.x() + 6, w.x() + w.w() - 6); (app::event_x() - l) as f32 / (r - l).max(1) as f32 }
    };
    sh.params.set(p, def.from_norm(n));
}

// ------------------------------------------------------------------ views

fn wt_view(c: &Ctx, x: i32, y: i32, w: i32, h: i32, osc: usize) {
    let mut wd = widget::Widget::new(x, y, w, h, "");
    let (sh, bank) = (c.sh.clone(), c.bank.clone());
    wd.draw(move |w| {
        let (x, y, ww, hh) = (w.x(), w.y(), w.w(), w.h());
        draw::set_draw_color(col(BG));
        draw::draw_rectf(x, y, ww, hh);
        let p = &OSC[osc];
        let mut buf = vec![0.0f32; ww.max(2) as usize];
        let pos = sh.params.get(p.pos);
        let dim = sh.params.get(p.on) < 0.5;
        // ghost frames either side of the current position
        for (off, c) in [(-0.06f32, 0x24404a), (0.06, 0x24404a), (0.0, if dim { 0x4a5a60 } else { ACC })] {
            bank.preview(sh.params.get(p.wt) as usize, (pos + off).clamp(0.0, 1.0), &mut buf);
            draw::set_draw_color(col(c));
            draw::set_line_style(LineStyle::Solid, if off == 0.0 { 2 } else { 1 });
            draw::begin_line();
            for (i, v) in buf.iter().enumerate() { draw::vertex((x + i as i32) as f64, (y + hh / 2) as f64 - (*v * (hh as f32 * 0.42)) as f64); }
            draw::end_line();
        }
        draw::set_line_style(LineStyle::Solid, 0);
        draw::set_draw_color(col(EDGE));
        draw::draw_rect(x, y, ww, hh);
    });
}

fn env_view(c: &Ctx, x: i32, y: i32, w: i32, h: i32, e: usize) {
    let mut wd = widget::Widget::new(x, y, w, h, "");
    let sh = c.sh.clone();
    wd.draw(move |w| {
        let (x, y, ww, hh) = (w.x(), w.y(), w.w(), w.h());
        draw::set_draw_color(col(BG));
        draw::draw_rectf(x, y, ww, hh);
        let ep = &ENV[e];
        let g = |i: usize| sh.params.get(i);
        let seg = |t: f32| t.max(0.0005).powf(0.4);
        let (a, hd, d, r) = (seg(g(ep.a)), if g(ep.h) > 0.0006 { seg(g(ep.h)) } else { 0.0 }, seg(g(ep.d)), seg(g(ep.r)));
        let sus = g(ep.s);
        let sus_w = 0.35 * (a + hd + d + r).max(0.5);
        let total = a + hd + d + sus_w + r;
        let sx = (ww - 8) as f32 / total;
        let (bx, by0, by1) = (x + 4, y + hh - 4, y + 4);
        let ypos = |l: f32| by0 as f32 - l * (by0 - by1) as f32;
        draw::set_draw_color(col(ACC));
        draw::set_line_style(LineStyle::Solid, 2);
        draw::begin_line();
        let mut t = 0.0;
        draw::vertex(bx as f64, by0 as f64);
        t += a; draw::vertex((bx as f32 + t * sx) as f64, by1 as f64);
        t += hd; draw::vertex((bx as f32 + t * sx) as f64, by1 as f64);
        for i in 1..=8 {
            let f = i as f32 / 8.0;
            let lvl = sus + (1.0 - sus) * (1.0 - f).powi(2);
            draw::vertex((bx as f32 + (t + d * f) * sx) as f64, ypos(lvl) as f64);
        }
        t += d; t += sus_w; draw::vertex((bx as f32 + t * sx) as f64, ypos(sus) as f64);
        for i in 1..=8 {
            let f = i as f32 / 8.0;
            draw::vertex((bx as f32 + (t + r * f) * sx) as f64, ypos(sus * (1.0 - f).powi(2)) as f64);
        }
        draw::end_line();
        draw::set_line_style(LineStyle::Solid, 0);
        draw::set_draw_color(col(EDGE));
        draw::draw_rect(x, y, ww, hh);
    });
}

fn lfo_view(c: &Ctx, x: i32, y: i32, w: i32, h: i32, l: usize) {
    let mut wd = widget::Widget::new(x, y, w, h, "");
    let sh = c.sh.clone();
    wd.draw(move |w| {
        let (x, y, ww, hh) = (w.x(), w.y(), w.w(), w.h());
        draw::set_draw_color(col(BG));
        draw::draw_rectf(x, y, ww, hh);
        let shape = sh.params.get(LFO[l].shape) as usize;
        draw::set_draw_color(col(ACC));
        draw::set_line_style(LineStyle::Solid, 2);
        draw::begin_line();
        for i in 0..(ww - 8) {
            let ph = i as f32 / (ww - 9) as f32;
            let v = match shape {
                0 => (2.0 * std::f32::consts::PI * ph).sin(),
                1 => 1.0 - 4.0 * (ph - 0.5).abs(),
                2 => 2.0 * ph - 1.0,
                3 => 1.0 - 2.0 * ph,
                4 => if ph < 0.5 { 1.0 } else { -1.0 },
                5 => (4.0 * std::f32::consts::PI * ph).sin() * 0.6 + (9.0 * ph).sin() * 0.3,
                _ => [0.6, -0.4, 0.9, -0.8, 0.2, -0.1][((ph * 6.0) as usize).min(5)],
            };
            draw::vertex((x + 4 + i) as f64, (y + hh / 2) as f64 - (v * (hh as f32 * 0.4)) as f64);
        }
        draw::end_line();
        draw::set_line_style(LineStyle::Solid, 0);
        draw::set_draw_color(col(EDGE));
        draw::draw_rect(x, y, ww, hh);
    });
}

fn scope_view(sh: Arc<Shared>, x: i32, y: i32, w: i32, h: i32) -> widget::Widget {
    let mut wd = widget::Widget::new(x, y, w, h, "");
    wd.draw(move |w| {
        let (x, y, ww, hh) = (w.x(), w.y(), w.w(), w.h());
        draw::set_draw_color(col(BG));
        draw::draw_rectf(x, y, ww, hh);
        let n = (ww - 4).max(8) as usize;
        let total = (n + 220).min(SCOPE_LEN);
        let end = sh.scope_pos.load(std::sync::atomic::Ordering::Relaxed);
        let samples: Vec<f32> = (0..total).map(|i| f32::from_bits(sh.scope[(end + SCOPE_LEN - total + i) % SCOPE_LEN].load(std::sync::atomic::Ordering::Relaxed))).collect();
        let mut start = 0;
        for i in 0..total.saturating_sub(n + 1) { if samples[i] <= 0.0 && samples[i + 1] > 0.0 { start = i; break; } }
        draw::set_draw_color(col(0x23424c));
        draw::draw_line(x, y + hh / 2, x + ww, y + hh / 2);
        draw::set_draw_color(col(ACC));
        draw::begin_line();
        for i in 0..n.min(total - start) {
            let v = samples[start + i].clamp(-1.0, 1.0);
            draw::vertex((x + 2 + i as i32) as f64, (y + hh / 2) as f64 - (v * (hh as f32 * 0.45)) as f64);
        }
        draw::end_line();
        draw::set_draw_color(col(EDGE));
        draw::draw_rect(x, y, ww, hh);
    });
    wd
}

fn meter_view(sh: Arc<Shared>, x: i32, y: i32, w: i32, h: i32) -> widget::Widget {
    let mut wd = widget::Widget::new(x, y, w, h, "");
    wd.draw(move |w| {
        let (x, y, ww, hh) = (w.x(), w.y(), w.w(), w.h());
        draw::set_draw_color(col(BG));
        draw::draw_rectf(x, y, ww, hh);
        let (l, r) = sh.get_peaks();
        for (i, pk) in [l, r].iter().enumerate() {
            let db = 20.0 * pk.max(1e-6).log10();
            let f = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
            let bh = ((hh - 4) as f32 * f) as i32;
            let bx = x + 3 + i as i32 * (ww / 2);
            draw::set_draw_color(col(if db > -1.0 { 0xe03a3a } else if db > -9.0 { ACC2 } else { 0x3ad08a }));
            draw::draw_rectf(bx, y + hh - 2 - bh, ww / 2 - 5, bh);
        }
        draw::set_draw_color(col(EDGE));
        draw::draw_rect(x, y, ww, hh);
    });
    wd
}

// ------------------------------------------------------------------ keyboard + wheels

const KB_LOW: i32 = 36; // C2
const KB_WHITES: i32 = 36; // C2..C7 (35 whites + final C)
const WHITE_IDX: [i32; 12] = [0, 0, 1, 1, 2, 3, 3, 4, 4, 5, 5, 6];
const IS_BLACK: [bool; 12] = [false, true, false, true, false, false, true, false, true, false, true, false];
const KEY_W: i32 = 32;
const BLACK_W: i32 = 20;

fn white_x(n: i32) -> i32 { (((n - KB_LOW) / 12) * 7 + WHITE_IDX[(n % 12) as usize]) * KEY_W }
fn key_rect(n: i32, kh: i32) -> (i32, i32, i32, i32) {
    if IS_BLACK[(n % 12) as usize] { (white_x(n - 1) + KEY_W - BLACK_W / 2, 0, BLACK_W, kh * 6 / 10) } else { (white_x(n), 0, KEY_W, kh) }
}

type Pressed = Rc<RefCell<[u8; 128]>>;

fn press(sh: &Shared, pressed: &Pressed, n: u8, vel: u8) { pressed.borrow_mut()[n as usize] += 1; sh.note_on(n, vel); }
fn release(sh: &Shared, pressed: &Pressed, n: u8) {
    let mut p = pressed.borrow_mut();
    if p[n as usize] > 0 { p[n as usize] -= 1; if p[n as usize] == 0 { sh.note_off(n); } }
}

fn keyboard(sh: Arc<Shared>, pressed: Pressed, x: i32, y: i32, h: i32) -> widget::Widget {
    let mut wd = widget::Widget::new(x, y, KB_WHITES * KEY_W, h, "");
    let (pr, ) = (pressed.clone(), );
    wd.draw(move |w| {
        let (x, y, kh) = (w.x(), w.y(), w.h());
        draw::set_draw_color(col(BG));
        draw::draw_rectf(x, y, w.w(), kh);
        let down = pr.borrow();
        for pass in 0..2 {
            for n in KB_LOW..=(KB_LOW + 60) {
                let blk = IS_BLACK[(n % 12) as usize];
                if (pass == 0) == blk { continue; }
                let (kx, ky, kw, khh) = key_rect(n, kh);
                let on = down[n as usize] > 0;
                draw::set_draw_color(col(if on { ACC } else if blk { 0x0c0e12 } else { 0xdfe3ea }));
                draw::draw_rectf(x + kx, y + ky, kw, khh);
                draw::set_draw_color(col(if blk { 0x2a2f3a } else { 0x8a92a3 }));
                draw::draw_rect(x + kx, y + ky, kw, khh);
                if n % 12 == 0 && !on {
                    draw::set_font(Font::Helvetica, 9);
                    draw::set_draw_color(col(0x5a6272));
                    draw::draw_text2(&format!("C{}", n / 12 - 1), x + kx, y + khh - 14, kw, 12, Align::Center);
                }
            }
        }
    });
    let cur: Rc<Cell<Option<u8>>> = Rc::new(Cell::new(None));
    let hit = move |w: &widget::Widget| -> Option<(u8, u8)> {
        let (mx, my) = (app::event_x() - w.x(), app::event_y() - w.y());
        let kh = w.h();
        for pass in [true, false] {
            for n in KB_LOW..=(KB_LOW + 60) {
                if IS_BLACK[(n % 12) as usize] != pass { continue; }
                let (kx, ky, kw, khh) = key_rect(n, kh);
                if mx >= kx && mx < kx + kw && my >= ky && my < ky + khh {
                    let vel = (45.0 + 80.0 * (my as f32 / khh as f32).clamp(0.0, 1.0)) as u8;
                    return Some((n as u8, vel));
                }
            }
        }
        None
    };
    wd.handle(move |w, ev| match ev {
        Event::Push => {
            if app::event_button() != 1 { return false; }
            if let Some((n, v)) = hit(w) { press(&sh, &pressed, n, v); cur.set(Some(n)); w.redraw(); }
            true
        }
        Event::Drag => {
            if let Some((n, v)) = hit(w) {
                if cur.get() != Some(n) {
                    if let Some(o) = cur.get() { release(&sh, &pressed, o); }
                    press(&sh, &pressed, n, v); cur.set(Some(n)); w.redraw();
                }
            }
            true
        }
        Event::Released => { if let Some(o) = cur.take() { release(&sh, &pressed, o); w.redraw(); } true }
        _ => false,
    });
    wd
}

fn wheel(sh: Arc<Shared>, x: i32, y: i32, w: i32, h: i32, is_bend: bool) {
    let mut wd = widget::Widget::new(x, y, w, h, "");
    let s1 = sh.clone();
    wd.draw(move |w| {
        let (x, y, ww, hh) = (w.x(), w.y(), w.w(), w.h());
        draw::set_draw_color(col(PANEL2));
        draw::draw_rounded_rectf(x, y, ww, hh, 5);
        let n = if is_bend { (s1.get_bend() + 1.0) * 0.5 } else { s1.get_mod() };
        let ty = y + hh - 8 - ((hh - 16) as f32 * n) as i32;
        draw::set_draw_color(col(ACC));
        draw::draw_rounded_rectf(x + 3, ty - 5, ww - 6, 10, 3);
        draw::set_font(Font::Helvetica, 9);
        draw::set_draw_color(col(DIM));
        draw::draw_text2(if is_bend { "PB" } else { "MW" }, x, y + 2, ww, 11, Align::Center);
    });
    wd.handle(move |w, ev| match ev {
        Event::Push | Event::Drag => {
            let n = 1.0 - ((app::event_y() - w.y() - 8) as f32 / (w.h() - 16).max(1) as f32);
            if is_bend { sh.set_bend(n * 2.0 - 1.0); } else { sh.set_mod(n); }
            w.redraw(); true
        }
        Event::Released => { if is_bend { sh.set_bend(0.0); w.redraw(); } true }
        _ => false,
    });
}

// ------------------------------------------------------------------ preset browser

struct Browser {
    sh: Arc<Shared>,
    refresh: Refresh,
    presets: Vec<Preset>,
    cats: Vec<String>,
    cat: usize, // 0 = All
    filtered: Vec<usize>,
    cur: usize,
    cat_choice: menu::Choice,
    preset_choice: menu::Choice,
    win: window::Window,
}

impl Browser {
    fn rebuild_cats(&mut self) {
        self.cats.clear();
        for p in &self.presets { if !self.cats.contains(&p.cat) { self.cats.push(p.cat.clone()); } }
        self.cat_choice.clear();
        self.cat_choice.add_choice("All");
        for c in &self.cats { self.cat_choice.add_choice(&esc(c)); }
        self.cat_choice.set_value(self.cat as i32);
    }
    fn rebuild_list(&mut self) {
        self.filtered = (0..self.presets.len()).filter(|&i| self.cat == 0 || self.presets[i].cat == self.cats[self.cat - 1]).collect();
        self.preset_choice.clear();
        for &i in &self.filtered { self.preset_choice.add_choice(&esc(&self.presets[i].name)); }
        self.preset_choice.set_value(self.filtered.iter().position(|&i| i == self.cur).map(|v| v as i32).unwrap_or(-1));
    }
    fn load(&mut self, idx: usize) {
        if idx >= self.presets.len() { return; }
        self.cur = idx;
        let mut vals = self.presets[idx].vals.clone();
        if self.presets[idx].cat != "Beats" && self.presets[idx].cat != "User" {
            let now = self.sh.params.snapshot();
            for i in 0..PARAM_COUNT { if is_global(i) { vals[i] = now[i]; } }
        }
        self.sh.params.load(&vals);
        self.sh.push(SynthEvent::AllOff);
        if self.cat != 0 && self.presets[idx].cat != self.cats[self.cat - 1] {
            self.cat = self.cats.iter().position(|c| *c == self.presets[idx].cat).map(|i| i + 1).unwrap_or(0);
            self.cat_choice.set_value(self.cat as i32);
            self.rebuild_list();
        } else {
            self.preset_choice.set_value(self.filtered.iter().position(|&i| i == idx).map(|v| v as i32).unwrap_or(-1));
        }
        for r in self.refresh.borrow().iter() { r(); }
        self.win.redraw();
    }
    fn step(&mut self, dir: i32) {
        if self.filtered.is_empty() { return; }
        let pos = self.filtered.iter().position(|&i| i == self.cur).unwrap_or(0) as i32;
        let n = self.filtered.len() as i32;
        let next = self.filtered[((pos + dir).rem_euclid(n)) as usize];
        self.load(next);
    }
    fn add_user(&mut self, p: Preset) {
        self.presets.push(p);
        self.rebuild_cats();
        let idx = self.presets.len() - 1;
        self.cat = self.cats.iter().position(|c| *c == self.presets[idx].cat).map(|i| i + 1).unwrap_or(0);
        self.cat_choice.set_value(self.cat as i32);
        self.rebuild_list();
        self.load(idx);
    }
}


// ------------------------------------------------------------------ drum machine + looper widgets

const DG_NAMEW: i32 = 58; // track-name column
const DG_CW: i32 = 32;    // step cell width
const DG_RH: i32 = 30;    // row height
const DG_HEAD: i32 = 14;  // step-number header

/// 8 x 16 step grid. Click toggles a step, drag paints; the playhead follows the sequencer.
fn drum_grid(c: &Ctx, x: i32, y: i32) -> widget::Widget {
    let mut wd = widget::Widget::new(x, y, DG_NAMEW + 16 * DG_CW, DG_HEAD + 8 * DG_RH, "");
    let sh = c.sh.clone();
    wd.draw(move |w| {
        let playhead = sh.drum_step.load(std::sync::atomic::Ordering::Relaxed);
        draw::set_font(Font::Helvetica, 9);
        for s in 0..16 {
            draw::set_draw_color(col(if s % 4 == 0 { TXT } else { DIM }));
            draw::draw_text2(&format!("{}", s + 1), w.x() + DG_NAMEW + s * DG_CW, w.y(), DG_CW, DG_HEAD, Align::Center);
        }
        for t in 0..8usize {
            let ry = w.y() + DG_HEAD + t as i32 * DG_RH;
            let muted = sh.params.get(DR_M1 + t * 5) >= 0.5;
            draw::set_font(Font::HelveticaBold, 11);
            draw::set_draw_color(col(if muted { 0x4a5163 } else { TXT }));
            draw::draw_text2(crate::drums::NAMES[t], w.x() + 4, ry, DG_NAMEW - 6, DG_RH, Align::Left);
            let mask = sh.params.get(DR_T1 + t * 5) as u32;
            for s in 0..16i32 {
                let rx = w.x() + DG_NAMEW + s * DG_CW;
                let on = mask & (1 << s) != 0;
                let base = if (s / 4) % 2 == 0 { PANEL2 } else { 0x303849 };
                draw::set_draw_color(col(if on { if muted { 0x1a6a76 } else { ACC } } else { base }));
                draw::draw_rounded_rectf(rx + 2, ry + 3, DG_CW - 4, DG_RH - 6, 4);
                if playhead == s as u32 {
                    draw::set_draw_color(col(ACC2));
                    draw::set_line_style(LineStyle::Solid, 2);
                    draw::draw_rect(rx + 2, ry + 3, DG_CW - 4, DG_RH - 6);
                    draw::set_line_style(LineStyle::Solid, 0);
                }
            }
        }
    });
    let sh = c.sh.clone();
    let paint: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));
    wd.handle(move |w, ev| {
        let cell = |w: &widget::Widget| -> Option<(usize, i32)> {
            let (mx, my) = (app::event_x() - w.x() - DG_NAMEW, app::event_y() - w.y() - DG_HEAD);
            if mx < 0 || my < 0 { return None; }
            let (s, t) = (mx / DG_CW, (my / DG_RH) as usize);
            if s >= 16 || t >= 8 { None } else { Some((t, s)) }
        };
        match ev {
            Event::Push | Event::Drag => {
                let Some((t, s)) = cell(w) else { return true };
                let p = DR_T1 + t * 5;
                let m = sh.params.get(p) as u32;
                let want = match paint.get() { Some(v) if ev == Event::Drag => v, _ => { let v = m & (1 << s) == 0; paint.set(Some(v)); v } };
                let nm = if want { m | (1 << s) } else { m & !(1 << s) };
                if nm != m {
                    sh.params.set(p, nm as f32);
                    if want { sh.push(SynthEvent::Drum(t as u8, 110)); }
                    w.redraw();
                }
                true
            }
            Event::Released => { paint.set(None); true }
            _ => false,
        }
    });
    wd
}

/// Looper state + progress bar (read-only, redrawn by the UI timer).
fn loop_view(sh: Arc<Shared>, x: i32, y: i32, w: i32, h: i32) -> widget::Widget {
    let mut wd = widget::Widget::new(x, y, w, h, "");
    wd.draw(move |w| {
        let st = sh.loop_state.load(std::sync::atomic::Ordering::Relaxed);
        let pos = f32::from_bits(sh.loop_pos.load(std::sync::atomic::Ordering::Relaxed)).clamp(0.0, 1.0);
        let (txt, c) = match st { 1 => ("RECORDING", 0xe05050), 2 => ("PLAYING", 0x40d080), 3 => ("OVERDUBBING", ACC2), 4 => ("STOPPED", DIM), _ => ("EMPTY - press REC", DIM) };
        draw::set_draw_color(col(BG));
        draw::draw_rounded_rectf(w.x(), w.y(), w.w(), w.h(), 4);
        let fw = (w.w() as f32 * pos) as i32;
        draw::set_draw_color(col(if st == 1 { 0x6a2a2a } else if st == 3 { 0x6a4a1a } else { 0x1a5a68 }));
        if st != 0 { draw::draw_rectf(w.x(), w.y(), fw, w.h()); }
        for i in 1..16 { draw::set_draw_color(col(EDGE)); draw::draw_line(w.x() + w.w() * i / 16, w.y(), w.x() + w.w() * i / 16, w.y() + if i % 4 == 0 { w.h() } else { w.h() / 3 }); }
        draw::set_font(Font::HelveticaBold, 13);
        draw::set_draw_color(col(c));
        draw::draw_text2(txt, w.x() + 10, w.y(), w.w() - 20, w.h(), Align::Left);
    });
    wd
}

/// Always-visible numeric readout of a parameter (e.g. "120 bpm").
fn readout(c: &Ctx, x: i32, y: i32, w: i32, h: i32, p: usize) -> widget::Widget {
    let mut wd = widget::Widget::new(x, y, w, h, "");
    let sh = c.sh.clone();
    wd.draw(move |w| {
        draw::set_draw_color(col(PANEL));
        draw::draw_rectf(w.x(), w.y(), w.w(), w.h());
        draw::set_font(Font::HelveticaBold, 10);
        draw::set_draw_color(col(ACC2));
        draw::draw_text2(&PARAMS[p].format(sh.params.get(p)), w.x(), w.y(), w.w(), w.h(), Align::Left);
    });
    wd
}

/// Tap-tempo button: averages the last few taps (resets after a 2.5 s pause) and writes BPM.
fn tap_button(c: &Ctx, x: i32, y: i32, w: i32, h: i32) -> button::Button {
    let mut b = button::Button::new(x, y, w, h, "Tap");
    style_button(&mut b);
    let (sh, taps): (Arc<Shared>, Rc<RefCell<Vec<std::time::Instant>>>) = (c.sh.clone(), Rc::new(RefCell::new(Vec::new())));
    b.set_callback(move |_| {
        let now = std::time::Instant::now();
        let mut t = taps.borrow_mut();
        if t.last().map_or(false, |l| now.duration_since(*l).as_secs_f32() > 2.5) { t.clear(); }
        t.push(now);
        if t.len() > 6 { t.remove(0); }
        if t.len() >= 2 {
            let span = now.duration_since(t[0]).as_secs_f32() / (t.len() - 1) as f32;
            if span > 0.0 { sh.params.set(BPM, (60.0 / span).clamp(PARAMS[BPM].min, PARAMS[BPM].max)); }
        }
        app::redraw();
    });
    b
}

fn randomize(sh: &Shared) {
    let mut s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1) | 1;
    let mut r = move || { s ^= s << 13; s ^= s >> 7; s ^= s << 17; (s >> 11) as f32 / (1u64 << 53) as f32 };
    let mut v = default_patch();
    let now = sh.params.snapshot();
    for i in 0..PARAM_COUNT { if is_global(i) { v[i] = now[i]; } }
    let mut set = |k: usize, x: f32| v[k] = PARAMS[k].clamp(x);
    let exp = |lo: f32, hi: f32, t: f32| lo * (hi / lo).powf(t);
    for o in &OSC[..2] {
        set(o.wt, (r() * 16.0).floor()); set(o.pos, r()); set(o.uni, 1.0 + (r() * r() * 6.0).floor());
        set(o.det, 0.1 + r() * 0.4); set(o.rand, if r() > 0.5 { 1.0 } else { 0.0 });
        set(o.wmode, if r() > 0.6 { (r() * 7.0).floor() } else { 0.0 }); set(o.warp, r() * 0.6);
        set(o.oct, if r() > 0.7 { -1.0 } else { 0.0 });
    }
    set(OSC[1].on, if r() > 0.45 { 1.0 } else { 0.0 }); set(OSC[1].fine, (r() - 0.5) * 24.0);
    set(OSC[1].level, 0.4 + r() * 0.3);
    set(SUB_ON, if r() > 0.7 { 1.0 } else { 0.0 });
    set(F_TYPE, [0.0, 1.0, 1.0, 1.0, 4.0, 2.0][(r() * 6.0) as usize % 6]);
    set(F_CUT, exp(250.0, 14000.0, r())); set(F_RES, r() * 0.5); set(F_ENV, (r() - 0.25) * 1.2);
    set(ENV[0].a, exp(0.001, 1.5, r() * r())); set(ENV[0].d, exp(0.2, 3.0, r())); set(ENV[0].s, if r() > 0.4 { 0.6 + r() * 0.4 } else { 0.0 });
    set(ENV[0].r, exp(0.08, 2.5, r()));
    set(ENV[1].d, exp(0.1, 2.0, r())); set(ENV[1].a, exp(0.001, 0.6, r() * r())); set(ENV[1].s, r() * 0.5);
    set(LFO[0].shape, (r() * 5.0).floor()); set(LFO[0].rate, exp(0.1, 12.0, r()));
    set(MODS[0].src, 3.0); set(MODS[0].dst, [1.0, 2.0, 11.0, 14.0][(r() * 4.0) as usize % 4]); set(MODS[0].amt, (r() - 0.3) * 0.9);
    set(R_ON, if r() > 0.4 { 1.0 } else { 0.0 }); set(R_MIX, 0.1 + r() * 0.3); set(R_SIZE, 0.4 + r() * 0.5);
    set(DL_ON, if r() > 0.55 { 1.0 } else { 0.0 }); set(DL_SYNC, 4.0); set(DL_FB, 0.3 + r() * 0.3);
    set(C_ON, if r() > 0.55 { 1.0 } else { 0.0 });
    set(D_ON, if r() > 0.8 { 1.0 } else { 0.0 }); set(D_TYPE, (r() * 4.0).floor()); set(D_DRIVE, r() * 0.5);
    sh.params.load(&v);
}

fn write_atomic(path: &std::path::Path, data: &str) -> std::io::Result<()> {
    use std::io::Write;
    let tmp = path.with_extension("scpreset.tmp");
    { let _ = std::fs::remove_file(&tmp); let mut f = std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp)?; f.write_all(data.as_bytes())?; f.sync_all()?; }
    std::fs::rename(&tmp, path)
}

// ------------------------------------------------------------------ main window

pub fn run(bank: Arc<WaveBank>, list: Vec<Preset>) {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    app::background(18, 20, 26);
    app::background2(27, 31, 40);
    app::foreground(195, 202, 216);
    app::set_visible_focus(false);
    app::set_font_size(11);

    let sh = Shared::new();
    sh.params.load(&list[1.min(list.len() - 1)].vals);
    let audio_res = audio::start(sh.clone(), bank.clone());
    let audio_info = match &audio_res {
        Ok(a) => a.info.clone(),
        Err(e) => { let mut t: String = format!("NO AUDIO ({e})").chars().take(52).collect(); t.push('…'); t }
    };
    let refresh: Refresh = Rc::new(RefCell::new(Vec::new()));
    let c = Ctx { sh: sh.clone(), bank: bank.clone(), refresh: refresh.clone() };
    let pressed: Pressed = Rc::new(RefCell::new([0u8; 128]));

    let mut win = window::Window::new(60, 40, 1240, 816, "");
    win.set_label(&format!("{} {}", APP_ID, APP_VERSION));
    win.set_color(col(BG));
    win.make_resizable(false);
    win.set_callback(|_| { if app::event() == Event::Close { app::quit(); } }); // Esc must not close the synth

    // ---------------------------------------------------- top bar
    {
        let mut logo = label(14, 6, 180, 24, "SAVIORSYNTH", ACC, 18);
        logo.set_label_font(Font::HelveticaBold);
        label(16, 28, 220, 14, &format!("{} {}", APP_ID, APP_VERSION), DIM, 9);
    }
    let mut prev = button::Button::new(236, 12, 26, 24, "@<");
    let mut cat_choice = menu::Choice::new(268, 12, 104, 24, "");
    let mut preset_choice = menu::Choice::new(378, 12, 230, 24, "");
    let mut next = button::Button::new(614, 12, 26, 24, "@>");
    let mut b_save = button::Button::new(654, 12, 54, 24, "Save");
    let mut b_load = button::Button::new(712, 12, 54, 24, "Load");
    let mut b_rand = button::Button::new(770, 12, 64, 24, "Random");
    let mut b_init = button::Button::new(838, 12, 50, 24, "Init");
    for b in [&mut prev, &mut next, &mut b_save, &mut b_load, &mut b_rand, &mut b_init] { style_button(b); }
    style_choice(&mut cat_choice); style_choice(&mut preset_choice);
    let mut b_synth = button::Button::new(900, 12, 84, 24, "SYNTH");
    let mut b_drums = button::Button::new(988, 12, 100, 24, "DRUMS + LOOP");
    style_button(&mut b_synth); style_button(&mut b_drums);
    let mut midi_choice = menu::Choice::new(1096, 14, 136, 22, "");
    style_choice(&mut midi_choice);
    label(1096, 1, 108, 12, "MIDI input", DIM, 9);
    label(236, 38, 650, 12, &format!("Audio: {}", audio_info), if audio_res.is_ok() { DIM } else { 0xe05050 }, 9);

    let g_synth = group::Group::new(0, 50, 1240, 554, "");
    // ---------------------------------------------------- row 1: oscillators, sub/noise, filter, master
    let (r1, r1h) = (50, 236);
    for (o, x) in [(0usize, 8i32), (1, 312)] {
        let p = &OSC[o];
        panel(x, r1, 300, r1h, &format!("OSC {}", if o == 0 { "A" } else { "B" }));
        toggle(&c, x + 252, r1 + 3, 40, 14, p.on, "ON");
        wt_view(&c, x + 8, r1 + 22, 284, 52, o);
        choice(&c, x + 8, r1 + 78, 150, 20, p.wt);
        choice(&c, x + 162, r1 + 78, 130, 20, p.wmode);
        let k = |i: i32| x + 6 + i * 41;
        let (ya, yb) = (r1 + 104, r1 + 164);
        ctl(&c, k(0), ya, 41, 56, p.pos, "Pos", Style::Knob);
        ctl(&c, k(1), ya, 41, 56, p.warp, "Warp", Style::Knob);
        ctl(&c, k(2), ya, 41, 56, p.oct, "Oct", Style::Knob);
        ctl(&c, k(3), ya, 41, 56, p.semi, "Semi", Style::Knob);
        ctl(&c, k(4), ya, 41, 56, p.fine, "Fine", Style::Knob);
        ctl(&c, k(5), ya, 41, 56, p.level, "Level", Style::Knob);
        ctl(&c, k(6), ya, 41, 56, p.pan, "Pan", Style::Knob);
        ctl(&c, k(0), yb, 41, 56, p.uni, "Unison", Style::Knob);
        ctl(&c, k(1), yb, 41, 56, p.det, "Detune", Style::Knob);
        ctl(&c, k(2), yb, 41, 56, p.blend, "Blend", Style::Knob);
        ctl(&c, k(3), yb, 41, 56, p.spread, "Spread", Style::Knob);
        ctl(&c, k(4), yb, 41, 56, p.phase, "Phase", Style::Knob);
        ctl(&c, k(5), yb, 41, 56, p.rand, "Rand", Style::Knob);
    }
    // sub + noise
    {
        let x = 616;
        panel(x, r1, 140, r1h, "SUB");
        toggle(&c, x + 92, r1 + 3, 40, 14, SUB_ON, "ON");
        choice(&c, x + 8, r1 + 22, 124, 20, SUB_SHAPE);
        ctl(&c, x + 8, r1 + 48, 58, 56, SUB_OCT, "Octave", Style::Knob);
        ctl(&c, x + 70, r1 + 48, 58, 56, SUB_LEVEL, "Level", Style::Knob);
        label(x + 8, r1 + 114, 100, 14, "NOISE", ACC, 11).set_label_font(Font::HelveticaBold);
        toggle(&c, x + 92, r1 + 113, 40, 14, NOISE_ON, "ON");
        choice(&c, x + 8, r1 + 134, 124, 20, NOISE_TYPE);
        ctl(&c, x + 36, r1 + 162, 68, 60, NOISE_LEVEL, "Level", Style::Knob);
    }
    // filter
    {
        let x = 760;
        panel(x, r1, 250, r1h, "FILTER");
        toggle(&c, x + 202, r1 + 3, 40, 14, F_ON, "ON");
        choice(&c, x + 9, r1 + 22, 232, 22, F_TYPE);
        label(x + 9, r1 + 50, 60, 14, "Route:", DIM, 10);
        for (i, (p, t)) in [(F_RA, "A"), (F_RB, "B"), (F_RS, "Sub"), (F_RN, "Noise")].iter().enumerate() {
            toggle(&c, x + 52 + i as i32 * 48, r1 + 49, 44, 16, *p, t);
        }
        let k = |i: i32| x + 9 + i * 58;
        ctl(&c, k(0), r1 + 76, 58, 66, F_CUT, "Cutoff", Style::Knob);
        ctl(&c, k(1), r1 + 76, 58, 66, F_RES, "Reso", Style::Knob);
        ctl(&c, k(2), r1 + 76, 58, 66, F_DRIVE, "Drive", Style::Knob);
        ctl(&c, k(3), r1 + 76, 58, 66, F_KEY, "Key Trk", Style::Knob);
        ctl(&c, k(0), r1 + 150, 58, 66, F_ENV, "Env2 Amt", Style::Knob);
        ctl(&c, k(1), r1 + 150, 58, 66, F_MIX, "Mix", Style::Knob);
    }
    // master / output
    let scope = {
        let x = 1014;
        panel(x, r1, 218, r1h, "OUTPUT");
        let scope = scope_view(sh.clone(), x + 8, r1 + 22, 202, 58);
        let k = |i: i32| x + 6 + i * 52;
        ctl(&c, k(0), r1 + 86, 52, 58, MASTER, "Master", Style::Knob);
        ctl(&c, k(1), r1 + 86, 52, 58, GLIDE, "Glide", Style::Knob);
        ctl(&c, k(2), r1 + 86, 52, 58, BEND, "Bend Rng", Style::Knob);
        ctl(&c, k(3), r1 + 86, 52, 58, BPM, "BPM", Style::Knob);
        ctl(&c, k(0), r1 + 150, 52, 58, VOICES, "Voices", Style::Knob);
        choice(&c, x + 62, r1 + 162, 84, 20, MODE);
        let m = meter_view(sh.clone(), x + 154, r1 + 150, 56, 58);
        (scope, m)
    };
    let mut voices_lbl = label(1020, r1 + 212, 100, 16, "Voices: 0", DIM, 10);
    let mut b_panic = button::Button::new(1130, r1 + 210, 80, 20, "Panic");
    style_button(&mut b_panic);

    // ---------------------------------------------------- row 2: envelopes + LFOs
    let r2 = 290;
    for e in 0..3usize {
        let x = 8 + e as i32 * 244;
        panel(x, r2, 240, 150, ["ENV 1  (AMP)", "ENV 2", "ENV 3"][e]);
        env_view(&c, x + 8, r2 + 20, 224, 34, e);
        let ep = &ENV[e];
        for (i, (p, t)) in [(ep.a, "A"), (ep.h, "H"), (ep.d, "D"), (ep.s, "S"), (ep.r, "R")].iter().enumerate() {
            ctl(&c, x + 10 + i as i32 * 44, r2 + 58, 44, 88, *p, t, Style::VSlider);
        }
    }
    for l in 0..4usize {
        let x = 740 + l as i32 * 124;
        panel(x, r2, 120, 150, &format!("LFO {}", l + 1));
        let lp = &LFO[l];
        choice(&c, x + 6, r2 + 20, 108, 18, lp.shape);
        lfo_view(&c, x + 6, r2 + 41, 108, 28, l);
        ctl(&c, x + 6, r2 + 72, 54, 54, lp.rate, "Rate", Style::Knob);
        ctl(&c, x + 60, r2 + 72, 54, 54, lp.fade, "Fade", Style::Knob);
        choice(&c, x + 6, r2 + 128, 66, 18, lp.sync);
        toggle(&c, x + 76, r2 + 128, 38, 18, lp.trig, "Trig");
    }

    // ---------------------------------------------------- row 3: mod matrix + FX
    let r3 = 444;
    panel(8, r3, 560, 156, "MOD MATRIX");
    for i in 0..8usize {
        let (col_i, row_i) = ((i / 4) as i32, (i % 4) as i32);
        let (x, y) = (14 + col_i * 276, r3 + 24 + row_i * 32);
        choice(&c, x, y, 84, 20, MODS[i].src);
        choice(&c, x + 88, y, 100, 20, MODS[i].dst);
        ctl(&c, x + 192, y, 76, 22, MODS[i].amt, "", Style::HSlider);
    }
    {
        let x = 572;
        panel(x, r3, 150, 156, "DISTORTION");
        toggle(&c, x + 102, r3 + 3, 40, 14, D_ON, "ON");
        choice(&c, x + 8, r3 + 22, 134, 20, D_TYPE);
        ctl(&c, x + 14, r3 + 56, 58, 66, D_DRIVE, "Drive", Style::Knob);
        ctl(&c, x + 78, r3 + 56, 58, 66, D_MIX, "Mix", Style::Knob);
    }
    {
        let x = 726;
        panel(x, r3, 150, 156, "CHORUS");
        toggle(&c, x + 102, r3 + 3, 40, 14, C_ON, "ON");
        ctl(&c, x + 6, r3 + 24, 46, 60, C_RATE, "Rate", Style::Knob);
        ctl(&c, x + 52, r3 + 24, 46, 60, C_DEPTH, "Depth", Style::Knob);
        ctl(&c, x + 98, r3 + 24, 46, 60, C_DELAY, "Delay", Style::Knob);
        ctl(&c, x + 52, r3 + 90, 46, 60, C_MIX, "Mix", Style::Knob);
    }
    {
        let x = 880;
        panel(x, r3, 190, 156, "DELAY");
        toggle(&c, x + 142, r3 + 3, 40, 14, DL_ON, "ON");
        choice(&c, x + 8, r3 + 22, 70, 20, DL_SYNC);
        toggle(&c, x + 84, r3 + 23, 70, 18, DL_PP, "Ping-Pong");
        ctl(&c, x + 4, r3 + 52, 46, 66, DL_TIME, "Time", Style::Knob);
        ctl(&c, x + 50, r3 + 52, 46, 66, DL_FB, "Feedbk", Style::Knob);
        ctl(&c, x + 96, r3 + 52, 46, 66, DL_TONE, "Tone", Style::Knob);
        ctl(&c, x + 142, r3 + 52, 46, 66, DL_MIX, "Mix", Style::Knob);
    }
    {
        let x = 1074;
        panel(x, r3, 158, 156, "REVERB");
        toggle(&c, x + 110, r3 + 3, 40, 14, R_ON, "ON");
        ctl(&c, x + 6, r3 + 40, 48, 66, R_SIZE, "Size", Style::Knob);
        ctl(&c, x + 54, r3 + 40, 48, 66, R_DAMP, "Damp", Style::Knob);
        ctl(&c, x + 102, r3 + 40, 48, 66, R_MIX, "Mix", Style::Knob);
    }

    g_synth.end();

    // ---------------------------------------------------- page 2: drum machine + looper
    let mut g_drum = group::Group::new(0, 50, 1240, 554, "");
    let (grid_w, loop_w) = {
        let (x, y) = (8, 50);
        panel(x, y, 770, 338, "DRUM MACHINE   (click or drag to paint steps, 16 steps = one bar)");
        toggle(&c, x + 8, y + 22, 56, 22, DR_RUN, "RUN");
        let mut pat = menu::Choice::new(x + 70, y + 22, 160, 22, "");
        style_choice(&mut pat);
        pat.add_choice("-- load pattern --"); pat.add_choice("(clear all steps)");
        for p in crate::drums::PATTERNS.iter() { pat.add_choice(&esc(p.name)); }
        pat.set_value(0);
        { let sh = sh.clone(); pat.set_callback(move |ch| {
            let v = ch.value();
            if v < 1 { return; }
            let m = if v == 1 { [0u16; crate::drums::TRACKS] } else { crate::drums::pattern_masks((v - 2) as usize) };
            for t in 0..crate::drums::TRACKS { sh.params.set(DR_T1 + t * 5, m[t] as f32); }
            ch.set_value(0); app::redraw();
        }); }
        label(x + 240, y + 26, 40, 14, "SWING", DIM, 9); ctl(&c, x + 276, y + 22, 90, 22, DR_SWING, "", Style::HSlider);
        label(x + 376, y + 26, 40, 14, "LEVEL", DIM, 9); ctl(&c, x + 410, y + 22, 90, 22, DR_VOL, "", Style::HSlider);
        label(x + 510, y + 26, 30, 14, "BPM", DIM, 9); ctl(&c, x + 536, y + 22, 90, 22, BPM, "", Style::HSlider);
        readout(&c, x + 630, y + 24, 56, 18, BPM); tap_button(&c, x + 690, y + 22, 40, 22);
        let gy = y + 52;
        label(x + 586, gy, 40, 12, "MUTE", DIM, 8); label(x + 640, gy, 60, 12, "LEVEL", DIM, 8);
        let grid = drum_grid(&c, x + 8, gy);
        for t in 0..crate::drums::TRACKS {
            let ry = gy + DG_HEAD + t as i32 * DG_RH + 4;
            toggle(&c, x + 586, ry, 40, 22, DR_M1 + t * 5, "M");
            ctl(&c, x + 636, ry, 120, 22, DR_V1 + t * 5, "", Style::HSlider);
        }
        // sound shaping
        let sx = 782;
        panel(sx, y, 450, 338, "DRUM SOUNDS");
        label(sx + 10, y + 26, 30, 14, "KIT", DIM, 9); choice(&c, sx + 40, y + 22, 128, 22, DR_KIT);
        label(sx + 176, y + 26, 40, 14, "ROOM", DIM, 9); ctl(&c, sx + 214, y + 22, 150, 22, DR_ROOM, "", Style::HSlider);
        label(sx + 80, gy, 100, 12, "TUNE", DIM, 8); label(sx + 262, gy, 100, 12, "DECAY", DIM, 8);
        for t in 0..crate::drums::TRACKS {
            let ry = gy + DG_HEAD + t as i32 * DG_RH + 4;
            label(sx + 10, ry + 4, 60, 14, crate::drums::NAMES[t], TXT, 11);
            ctl(&c, sx + 72, ry, 170, 22, DR_TUNE1 + t * 5, "", Style::HSlider);
            ctl(&c, sx + 254, ry, 170, 22, DR_DEC1 + t * 5, "", Style::HSlider);
        }
        // looper
        let ly = 396;
        panel(8, ly, 1224, 150, "LOOPER   (records the live mix: synth + drums)");
        let mut b_rec = button::Button::new(20, ly + 28, 130, 46, "REC / DUB");
        let mut b_play = button::Button::new(156, ly + 28, 130, 46, "PLAY / STOP");
        let mut b_undo = button::Button::new(292, ly + 28, 100, 46, "UNDO DUB");
        let mut b_clear = button::Button::new(398, ly + 28, 100, 46, "CLEAR");
        for (b, cmd) in [(&mut b_rec, 0u8), (&mut b_play, 1), (&mut b_undo, 2), (&mut b_clear, 3)] {
            style_button(b); b.set_label_size(12);
            let sh = sh.clone(); b.set_callback(move |_| sh.push(SynthEvent::Loop(cmd)));
        }
        label(520, ly + 24, 100, 11, "LENGTH", DIM, 8); choice(&c, 520, ly + 38, 110, 22, LP_BARS);
        label(650, ly + 24, 100, 11, "LOOP LEVEL", DIM, 8); ctl(&c, 650, ly + 38, 170, 22, LP_VOL, "", Style::HSlider);
        let lv = loop_view(sh.clone(), 20, ly + 84, 1200, 28);
        label(20, ly + 118, 1200, 26, "REC starts recording at once for the chosen number of bars (length from BPM) and then loops; press REC again to overdub, again to stop overdubbing. In Free length, press REC again to set the loop length. UNDO DUB removes all overdubs. CLEAR empties the loop.", DIM, 9);
        (grid, lv)
    };
    g_drum.end();
    g_drum.hide();

    // ---------------------------------------------------- performance: chords / progressions / arpeggiator
    let r4 = 604;
    let ph = 124;
    let mut chord_lbl = label(500, r4 + 60, 160, 34, "", ACC2, 20);
    {
        let x = 8;
        panel(x, r4, 660, ph, "CHORDS  &  PROGRESSIONS   (play ONE key)");
        let h = |t: &str, px: i32, py: i32| { label(px, py, 100, 11, t, DIM, 8); };
        let mut px = x + 6;
        for (t, p, w) in [("MODE", CH_MODE, 100), ("SCALE", CH_SCALE, 94), ("DEGREE", CH_DEG, 58), ("QUALITY", CH_QUAL, 78), ("VARIATION", CH_EXT, 90), ("INVERSION", CH_INV, 70)] {
            h(t, px, r4 + 17); choice(&c, px, r4 + 29, w, 20, p); px += w + 6;
        }
        toggle(&c, px, r4 + 29, 44, 20, CH_OPEN, "Open"); toggle(&c, px + 48, r4 + 29, 44, 20, CH_BASS, "Bass");
        h("PROGRESSION", x + 6, r4 + 54); choice(&c, x + 6, r4 + 66, 124, 20, PR_PAT);
        h("STEP LENGTH", x + 136, r4 + 54); choice(&c, x + 136, r4 + 66, 74, 20, PR_LEN);
        h("CUSTOM STEPS (when Progression = Custom)", x + 216, r4 + 54);
        for (i, p) in [PR_S1, PR_S2, PR_S3, PR_S4].iter().enumerate() { choice(&c, x + 216 + i as i32 * 62, r4 + 66, 58, 20, *p); }
        h("TEMPO (arp, progression, drums, looper)", x + 6, r4 + 91);
        ctl(&c, x + 6, r4 + 100, 200, 20, BPM, "", Style::HSlider);
        readout(&c, x + 212, r4 + 101, 64, 18, BPM); tap_button(&c, x + 282, r4 + 100, 44, 20);
    }
    {
        let x = 672;
        panel(x, r4, 560, ph, "ARPEGGIATOR  +  HUMANIZE");
        let h = |t: &str, px: i32, py: i32| { label(px, py, 100, 11, t, DIM, 8); };
        toggle(&c, x + 6, r4 + 29, 44, 20, AR_ON, "ON");
        let mut px = x + 56;
        for (t, p, w) in [("PATTERN", AR_PAT, 112), ("RATE", AR_RATE, 58), ("OCTAVES", AR_OCT, 66), ("INVERSION CYCLE", AR_INVM, 100)] {
            h(t, px, r4 + 17); choice(&c, px, r4 + 29, w, 20, p); px += w + 6;
        }
        toggle(&c, px, r4 + 29, 52, 20, AR_LATCH, "Latch");
        for (i, (t, p)) in [("GATE", AR_GATE), ("SWING", AR_SWING), ("ACCENT", AR_ACC)].iter().enumerate() {
            let sx = x + 8 + i as i32 * 132;
            h(t, sx, r4 + 54); ctl(&c, sx, r4 + 64, 126, 22, *p, "", Style::HSlider);
        }
        h("BPM", x + 408, r4 + 54); ctl(&c, x + 408, r4 + 64, 90, 22, BPM, "", Style::HSlider);
        tap_button(&c, x + 502, r4 + 64, 48, 22);
        label(x + 8, r4 + 98, 70, 14, "HUMANIZE", ACC2, 10);
        for (i, (t, p)) in [("TIME", HUM_TIME), ("VELOCITY", HUM_VEL), ("GATE", HUM_GATE)].iter().enumerate() {
            let sx = x + 78 + i as i32 * 160;
            label(sx, r4 + 99, 50, 12, t, DIM, 8); ctl(&c, sx + 42, r4 + 94, 104, 22, *p, "", Style::HSlider);
        }
    }
    chord_lbl.set_label_font(Font::HelveticaBold);

    // ---------------------------------------------------- keyboard
    let r5 = r4 + ph + 4;
    wheel(sh.clone(), 8, r5, 28, 76, true);
    wheel(sh.clone(), 40, r5, 28, 76, false);
    let mut kb = keyboard(sh.clone(), pressed.clone(), 76, r5, 76);
    let octave = Rc::new(Cell::new(48i32));
    win.end();

    // page switching
    {
        let (mut gs, mut gd, mut bs, mut bd) = (g_synth.clone(), g_drum.clone(), b_synth.clone(), b_drums.clone());
        let mut show = move |drums: bool| {
            if drums { gs.hide(); gd.show(); } else { gd.hide(); gs.show(); }
            bs.set_color(col(if drums { PANEL } else { ACC })); bd.set_color(col(if drums { ACC } else { PANEL }));
            bs.redraw(); bd.redraw(); app::redraw();
        };
        let mut s1 = show.clone(); b_synth.set_callback(move |_| s1(false));
        let mut s2 = show.clone(); b_drums.set_callback(move |_| s2(true));
        show(false);
    }

    // ---------------------------------------------------- preset browser wiring
    let browser = Rc::new(RefCell::new(Browser {
        sh: sh.clone(), refresh: refresh.clone(), presets: list, cats: vec![], cat: 0, filtered: vec![], cur: 1,
        cat_choice: cat_choice.clone(), preset_choice: preset_choice.clone(), win: win.clone(),
    }));
    {
        let mut b = browser.borrow_mut();
        b.rebuild_cats(); b.rebuild_list();
        let idx = b.cur.min(b.presets.len() - 1);
        b.load(idx);
    }
    { let b = browser.clone(); cat_choice.set_callback(move |ch| { let mut b = b.borrow_mut(); b.cat = ch.value().max(0) as usize; b.rebuild_list(); let first = b.filtered.first().copied(); if let Some(i) = first { b.load(i); } }); }
    { let b = browser.clone(); preset_choice.set_callback(move |ch| { let mut b = b.borrow_mut(); let v = ch.value(); if v >= 0 { if let Some(&i) = b.filtered.get(v as usize) { b.load(i); } } }); }
    { let b = browser.clone(); prev.set_callback(move |_| b.borrow_mut().step(-1)); }
    { let b = browser.clone(); next.set_callback(move |_| b.borrow_mut().step(1)); }
    { let (b, sh) = (browser.clone(), sh.clone()); b_rand.set_callback(move |_| { randomize(&sh); let b = b.borrow(); for r in b.refresh.borrow().iter() { r(); } b.win.clone().redraw(); }); }
    { let b = browser.clone(); b_init.set_callback(move |_| { let mut b = b.borrow_mut(); if let Some(i) = b.presets.iter().position(|p| p.name == "Init Saw") { b.load(i); } }); }
    { let sh = sh.clone(); b_panic.set_callback(move |_| { sh.push(SynthEvent::AllOff); sh.set_bend(0.0); }); }
    {
        let (b, sh) = (browser.clone(), sh.clone());
        b_save.set_callback(move |_| {
            let cur_name = { let b = b.borrow(); b.presets.get(b.cur).map(|p| p.name.clone()).unwrap_or_default() };
            let Some(name) = dialog::input_default("Preset name:", &cur_name) else { return };
            let name = presets::sanitize_name(&name);
            let mut fc = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseSaveFile);
            fc.set_option(dialog::NativeFileChooserOptions::SaveAsConfirm);
            fc.set_filter("Presets\t*.scpreset");
            fc.set_preset_file(&format!("{}.scpreset", crate::offline::file_stem(&name)));
            let _ = fc.show();
            let mut path = fc.filename();
            if path.as_os_str().is_empty() { return; }
            if path.extension().is_none() { path.set_extension("scpreset"); }
            let p = Preset { name, cat: "User".into(), vals: sh.params.snapshot() };
            if let Err(e) = write_atomic(&path, &presets::to_text(&p, APP_ID)) { dialog::alert_default(&format!("Could not save preset:\n{e}")); }
        });
    }
    {
        let b = browser.clone();
        b_load.set_callback(move |_| {
            let mut fc = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseFile);
            fc.set_filter("Presets\t*.scpreset");
            let _ = fc.show();
            let path = fc.filename();
            if path.as_os_str().is_empty() { return; }
            let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "User".into());
            let text = match std::fs::metadata(&path) {
                Ok(m) if m.len() > 200_000 => { dialog::alert_default("Preset file is too large."); return; }
                Ok(_) => match std::fs::read_to_string(&path) { Ok(t) => t, Err(e) => { dialog::alert_default(&format!("Could not read preset:\n{e}")); return; } },
                Err(e) => { dialog::alert_default(&format!("Could not open preset:\n{e}")); return; }
            };
            match presets::from_text(&text, &presets::sanitize_name(&stem)) {
                Ok(p) => b.borrow_mut().add_user(p),
                Err(e) => dialog::alert_default(&format!("Not a valid preset file: {e}")),
            }
        });
    }

    // ---------------------------------------------------- MIDI
    let midi_ports = midi::list_ports();
    midi_choice.add_choice("(none)");
    for p in &midi_ports { midi_choice.add_choice(&esc(p)); }
    midi_choice.set_value(0);
    let midi_conn: Rc<RefCell<Option<midi::MidiConn>>> = Rc::new(RefCell::new(None));
    {
        let (sh, mc) = (sh.clone(), midi_conn.clone());
        midi_choice.set_callback(move |ch| {
            *mc.borrow_mut() = None;
            let v = ch.value();
            if v > 0 {
                match midi::connect((v - 1) as usize, sh.clone()) {
                    Ok(conn) => *mc.borrow_mut() = Some(conn),
                    Err(e) => dialog::alert_default(&format!("MIDI: {e}")),
                }
            }
        });
    }
    if let Some(i) = (!midi_ports.is_empty()).then_some(0usize) {
        if let Ok(conn) = midi::connect(i, sh.clone()) { *midi_conn.borrow_mut() = Some(conn); midi_choice.set_value(1); }
    }

    // ---------------------------------------------------- computer keyboard
    {
        let (sh, pr, oct, mut kbw) = (sh.clone(), pressed.clone(), octave.clone(), kb.clone());
        let held: Rc<RefCell<HashMap<char, u8>>> = Rc::new(RefCell::new(HashMap::new()));
        const KEYS: &str = "awsedftgyhujkolp;";
        let mut winc = win.clone();
        win.handle(move |_, ev| {
            match ev {
                Event::KeyDown | Event::Shortcut => {
                    if app::event_state().intersects(EventState::Ctrl | EventState::Alt | EventState::Command) { return false; }
                    let Some(ch) = app::event_key().to_char().map(|c| c.to_ascii_lowercase()) else { return false };
                    if ch == 'z' || ch == 'x' {
                        oct.set((oct.get() + if ch == 'x' { 12 } else { -12 }).clamp(12, 84));
                        winc.set_label(&format!("{} {}   [keyboard base: C{}]", APP_ID, APP_VERSION, oct.get() / 12 - 1));
                        return true;
                    }
                    if let Some(i) = KEYS.find(ch) {
                        if held.borrow().contains_key(&ch) { return true; } // auto-repeat
                        let n = (oct.get() + i as i32).clamp(0, 127) as u8;
                        held.borrow_mut().insert(ch, n);
                        press(&sh, &pr, n, 100);
                        kbw.redraw();
                        return true;
                    }
                    false
                }
                Event::KeyUp => {
                    let Some(ch) = app::event_key().to_char().map(|c| c.to_ascii_lowercase()) else { return false };
                    if let Some(n) = held.borrow_mut().remove(&ch) { release(&sh, &pr, n); kbw.redraw(); return true; }
                    false
                }
                Event::Unfocus | Event::Hide => {
                    for (_, n) in held.borrow_mut().drain() { release(&sh, &pr, n); }
                    false
                }
                _ => false,
            }
        });
    }

    // ---------------------------------------------------- UI refresh timer (scope, meter, voice count)
    {
        let (mut scope_w, mut meter_w) = (scope.0.clone(), scope.1.clone());
        let (sh, mut kbw) = (sh.clone(), kb.clone());
        let (mut gridw, mut loopw) = (grid_w.clone(), loop_w.clone());
        let last = Rc::new(Cell::new(u32::MAX));
        let mut chord_w = chord_lbl.clone();
        let last_chord = Rc::new(RefCell::new(String::new()));
        app::add_timeout3(0.04, move |h| {
            let r = sh.chord_root.load(std::sync::atomic::Ordering::Relaxed);
            let name = if r == u32::MAX { String::new() } else { crate::perform::chord_name(r as i32, sh.chord_mask.load(std::sync::atomic::Ordering::Relaxed)) };
            if *last_chord.borrow() != name { chord_w.set_label(&name); chord_w.redraw(); *last_chord.borrow_mut() = name; }
            scope_w.redraw(); meter_w.redraw(); kbw.redraw();
            if gridw.visible() { gridw.redraw(); loopw.redraw(); }
            let n = sh.voices.load(std::sync::atomic::Ordering::Relaxed);
            if n != last.get() { last.set(n); voices_lbl.set_label(&format!("Voices: {}", n)); voices_lbl.redraw(); }
            app::repeat_timeout3(0.04, h);
        });
    }

    win.show();
    let _ = &mut kb;
    let _ = app.run();
    drop(audio_res);
}
