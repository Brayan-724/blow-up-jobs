#![expect(dead_code, reason = "lib in progress")]

macro_rules! type_iter {
    (<$T:ident $(, $lf:lifetime)*> once ($($arg:ident: $ty:ty),*$(,)?) $(-> $ret:ty)? $body:block) => {{
        struct Funnel<$($lf,)*> {
            $($arg: $ty),*
        }

        impl<$($lf,)*> TypeIteratorOnce for Funnel<$($lf,)*> {
            type Item = type_iter!(@ret-ty $($ret)?);

            fn once<$T: $crate::ui::popup::Popup>(self) -> Self::Item {
                let Self {$($arg),*} = self;
                $body
            }
        }

        Funnel { $($arg),* }
    }};

    (<$T:ident $(, $lf:lifetime)*> once async ($($arg:ident: $ty:ty),*$(,)?) $(-> $ret:ty)? $body:block) => {{
        struct Funnel<$($lf,)*> {
            $($arg: $ty),*
        }

        impl<$($lf,)*> AsyncTypeIteratorOnce for Funnel<$($lf,)*> {
            type Item = type_iter!(@ret-ty $($ret)?);

            async fn once<$T: $crate::ui::popup::Popup>(self) -> Self::Item {
                let Self {$($arg),*} = self;
                $body
            }
        }

        Funnel { $($arg),* }
    }};

    (@ret-ty) => {()};
    (@ret-ty $ret:ty) => {$ret};
}

pub mod common;
pub mod intro_overlay;
pub mod job;
pub mod popup;
pub mod sidebar;

use std::marker::PhantomData;
use std::ops;
use std::rc::Rc;

use crossterm::event::{Event, KeyEvent, MouseEvent};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Margin, Offset, Rect, Size};
use ratatui::widgets::{StatefulWidget, Widget};

use crate::variadicts::{all_tuples_repeated, dual_permutation, indexed_slice};

#[allow(unused_imports)]
pub mod prelude {
    pub use super::*;
    pub use ratatui::Frame;
    pub use ratatui::buffer::*;
    pub use ratatui::layout::*;
    pub use ratatui::style::*;
    pub use ratatui::symbols::*;
    pub use ratatui::text::*;
    pub use ratatui::widgets::*;

    pub use crossterm::event::*;

    // Overwrite ratatui::Layout
    pub use super::Layout;
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Action {
    #[default]
    Noop,
    /// Don't operate but takes the tick
    Intercept,
    Quit,
    Tick,
}

impl std::ops::FromResidual<Action> for Action {
    fn from_residual(residual: Action) -> Self {
        residual
    }
}

impl std::ops::Try for Action {
    type Output = ();
    type Residual = Action;

    fn from_output(_: Self::Output) -> Self {
        Self::Noop
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Action::Noop => std::ops::ControlFlow::Continue(()),
            _ => std::ops::ControlFlow::Break(self),
        }
    }
}

#[expect(unused_variables, reason = "default templating")]
pub trait Component {
    type State;

    fn on_mount(state: &mut Self::State) {}
    fn on_destroy(state: &mut Self::State) {}

    async fn handle_event(state: &mut Self::State, event: Event) -> Action {
        match event {
            Event::Key(key_event) => Self::handle_key_events(state, key_event).await?,
            Event::Mouse(mouse_event) => Self::handle_mouse_events(state, mouse_event).await?,
            Event::Resize(_, _) => Action::Tick?,
            _ => {}
        };

        Self::propagate_event(state, event).await
    }

    async fn propagate_event(state: &mut Self::State, event: Event) -> Action {
        Action::Noop
    }

    async fn handle_key_events(state: &mut Self::State, key: KeyEvent) -> Action {
        Action::Noop
    }

    async fn handle_mouse_events(state: &mut Self::State, mouse: MouseEvent) -> Action {
        Action::Noop
    }

    fn draw(state: &mut Self::State, frame: &mut Frame, area: Rect);
}

pub type ComponentDraw<'a, S> = for<'s, 'f> fn(&'a mut S, &'s mut Frame<'f>, Rect);

pub struct Layout<const N: usize, T> {
    inner: ratatui::layout::Layout,
    _marker: PhantomData<T>,
}

impl<const N: usize, T: ContraintBundle<N>> Layout<N, T> {
    pub fn horizontal(constraints: T) -> Self {
        Layout {
            inner: ratatui::layout::Layout::horizontal(constraints.into_vec()),
            _marker: Default::default(),
        }
    }

    pub fn vertical(constraints: T) -> Self {
        Layout {
            inner: ratatui::layout::Layout::vertical(constraints.into_vec()),
            _marker: Default::default(),
        }
    }

    pub fn flex(mut self, flex: Flex) -> Self {
        self.inner = self.inner.flex(flex);
        self
    }

    pub fn split(&self, area: Rect) -> T::Out {
        T::from_rects(self.inner.split(area))
    }
}

pub trait ContraintBundle<const N: usize> {
    type Out: RectsBundle<N>;

    fn into_vec(self) -> Vec<Constraint>;
    fn from_rects(rects: Rc<[Rect]>) -> Self::Out;
}

macro_rules! impl_contraint_bundle {
    ($n:tt, $(()),*) => {
        impl ContraintBundle<$n> for [Constraint; $n] {
            type Out = [Rect; $n];

            fn into_vec(self) -> Vec<Constraint> {
                self.to_vec()
            }

            fn from_rects(rects: Rc<[Rect]>) -> Self::Out {
                Self::Out::from_rects(rects)
            }
        }
    };
}

all_tuples_repeated!(impl_contraint_bundle, 1, 10, ());

pub trait RectsBundle<const N: usize> {
    fn from_rects(rects: Rc<[Rect]>) -> Self;
}

macro_rules! impl_rects_bundle {
    ($n:tt, $(()),*) => {
        impl RectsBundle<$n> for [Rect; $n] {
            fn from_rects(rects: Rc<[Rect]>) -> Self {
                indexed_slice!(rects, $n)
            }
        }
    };
}

all_tuples_repeated!(impl_rects_bundle, 1, 10, ());

pub trait Drawable<'a, Marker> {
    type State = ();
    const STATEFUL: bool = false;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect);
}

impl<'a> Drawable<'a, ()> for () {
    type State = ();
    const STATEFUL: bool = false;

    fn draw(self, _: Self::State, _: &mut Frame, _: Rect) {}
}

impl<'a, S: 'a + Widget> Drawable<'a, fn(S) -> bool> for S {
    type State = ();
    const STATEFUL: bool = false;

    fn draw(self, _: Self::State, frame: &mut Frame, area: Rect) {
        self.render(area, frame.buffer_mut());
    }
}

impl<'a, S: 'a + StatefulWidget> Drawable<'a, fn(&'a S)> for S {
    type State = &'a mut S::State;
    const STATEFUL: bool = true;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        self.render(area, frame.buffer_mut(), state);
    }
}

impl<'a, F> Drawable<'a, fn(Rect, &mut Buffer)> for F
where
    F: 'a + FnOnce(Rect, &mut Buffer),
{
    fn draw(self, _: Self::State, frame: &mut Frame, area: Rect) {
        self(area, frame.buffer_mut())
    }
}

impl<'a, 's, S: 's, F> Drawable<'a, fn(&'s S, Rect, &mut Buffer)> for F
where
    F: 'a + FnOnce(&'s S, Rect, &mut Buffer),
{
    type State = &'s S;
    const STATEFUL: bool = true;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        self(state, area, frame.buffer_mut())
    }
}

impl<'a, 's, S: 's, F> Drawable<'a, fn(&'s mut S, Rect, &mut Buffer)> for F
where
    F: 'a + FnOnce(&'s mut S, Rect, &mut Buffer),
{
    type State = &'s mut S;
    const STATEFUL: bool = true;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        self(state, area, frame.buffer_mut())
    }
}

impl<'a, F> Drawable<'a, fn(&mut Frame<'_>, Rect)> for F
where
    F: 'a + FnOnce(&mut Frame, Rect),
{
    fn draw(self, _: Self::State, frame: &mut Frame, area: Rect) {
        self(frame, area)
    }
}

impl<'a, 's, S: 's, F> Drawable<'a, fn(&'s S, &mut Frame<'_>, Rect)> for F
where
    F: 'a + FnOnce(&'s S, &mut Frame, Rect),
{
    type State = &'s S;
    const STATEFUL: bool = true;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        self(state, frame, area)
    }
}

impl<'a, 's, S: 's> Drawable<'static, &'s ComponentDraw<'s, S>> for ComponentDraw<'s, S> {
    type State = &'s mut S;
    const STATEFUL: bool = true;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        self(state, frame, area)
    }
}

impl<'a, 's, S: 's, F> Drawable<'a, fn(&'s mut S, &mut Frame<'_>, Rect)> for F
where
    F: 'a + FnOnce(&'s mut S, &mut Frame, Rect),
{
    type State = &'s mut S;
    const STATEFUL: bool = true;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        self(state, frame, area)
    }
}

pub trait FrameExt {
    fn draw<'a, M, D: Drawable<'a, M>>(&mut self, drawable: D, area: Rect, state: D::State);
    fn draw_stateless<'a, M, D: Drawable<'a, M, State = ()>>(&mut self, drawable: D, area: Rect);
}

impl FrameExt for Frame<'_> {
    fn draw<'a, M, D: Drawable<'a, M>>(&mut self, drawable: D, area: Rect, state: D::State) {
        drawable.draw(state, self, area);
    }
    fn draw_stateless<'a, M, D: Drawable<'a, M, State = ()>>(&mut self, drawable: D, area: Rect) {
        drawable.draw((), self, area);
    }
}

pub trait RectExt: Sized {
    fn reduce(self, size: impl Into<Size>) -> Self;
    fn outline(self, size: impl Into<Size>) -> Self;
    fn centered(self, size: impl Into<Size>) -> Self;
    fn set_height(self, value: u16) -> Self;
    fn set_width(self, value: u16) -> Self;
    fn inner_x(self, value: i32) -> Self;
    fn inner_y(self, value: i32) -> Self;
}

impl RectExt for Rect {
    fn reduce(mut self, size: impl Into<Size>) -> Self {
        let size: Size = size.into();
        self.width = self.width.saturating_sub(size.width);
        self.height = self.height.saturating_sub(size.height);

        self
    }

    fn outline(mut self, size: impl Into<Size>) -> Self {
        let size: Size = size.into();
        self.x = self.x.saturating_sub(size.width);
        self.y = self.y.saturating_sub(size.height);
        self.width = self.width.saturating_add(size.width.saturating_mul(2));
        self.height = self.height.saturating_add(size.height.saturating_mul(2));

        self
    }

    fn centered(mut self, size: impl Into<Size>) -> Self {
        let orig = self;

        let size: Size = size.into();

        self.x = self.x.saturating_add(self.width / 2 - size.width / 2);
        self.y = self.y.saturating_add(self.height / 2 - size.height / 2);
        self.width = size.width;
        self.height = size.height;

        self.intersection(orig)
    }

    fn set_height(mut self, value: u16) -> Self {
        self.height = value;
        self
    }

    fn set_width(mut self, value: u16) -> Self {
        self.width = value;
        self
    }

    fn inner_x(self, value: i32) -> Self {
        self.offset(Offset::x(value))
            .set_width(self.width.saturating_sub_signed(value as i16))
    }

    fn inner_y(self, value: i32) -> Self {
        self.offset(Offset::y(value))
            .set_height(self.height.saturating_sub_signed(value as i16))
    }
}

pub trait OffsetExt: Sized {
    fn x(value: i32) -> Self;
    fn y(value: i32) -> Self;
}

impl OffsetExt for Offset {
    fn x(value: i32) -> Self {
        Self { x: value, y: 0 }
    }

    fn y(value: i32) -> Self {
        Self { y: value, x: 0 }
    }
}

pub trait MarginExt: Sized {
    fn horizontal(value: u16) -> Self;
    fn vertical(value: u16) -> Self;
    fn both(value: u16) -> Self;
}

impl MarginExt for Margin {
    fn horizontal(value: u16) -> Self {
        Self {
            horizontal: value,
            vertical: 0,
        }
    }

    fn vertical(value: u16) -> Self {
        Self {
            vertical: value,
            horizontal: 0,
        }
    }

    fn both(value: u16) -> Self {
        Self {
            horizontal: value,
            vertical: value,
        }
    }
}

pub trait Arithmetic<T>:
    Sized
    + ops::Add<Self, Output = T>
    + ops::Sub<Self, Output = T>
    + ops::Mul<Self, Output = T>
    + ops::Div<Self, Output = T>
    + PartialOrd<Self>
{
}

impl<T, S> Arithmetic<T> for S where
    S: ops::Add<Self, Output = T>
        + ops::Sub<Self, Output = T>
        + ops::Mul<Self, Output = T>
        + ops::Div<Self, Output = T>
        + PartialOrd
{
}

pub trait Cast<T> {
    fn cast(self) -> T;
}

macro_rules! impl_cast {
    ($a:ty, $b:ty) => {
        impl_cast!(@ $a, $b);
        impl_cast!(@ $b, $a);
    };

    (@ $a:ty, $b:ty) => {
        impl Cast<$b> for $a {
            fn cast(self) -> $b {
                self as $b
            }
        }
    };
}

#[rustfmt::skip]
dual_permutation!(impl_cast, [
    usize, u8, u16, u32,
    isize, i8, i16, i32,
    f32, f64,
]);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

pub trait TypeIteratorOnce {
    type Item;

    fn once<T: popup::Popup>(self) -> Self::Item;
}

pub trait AsyncTypeIteratorOnce {
    type Item;

    async fn once<T: popup::Popup>(self) -> Self::Item;
}
