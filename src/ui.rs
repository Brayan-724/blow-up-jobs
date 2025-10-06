#![expect(dead_code, reason = "lib in progress")]

pub mod common;
pub mod intro_overlay;
pub mod job;
pub mod sidebar;

use std::marker::PhantomData;
use std::rc::Rc;

use crossterm::event::{Event, KeyEvent, MouseEvent};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Margin, Offset, Rect, Size};

use crate::variadicts::{all_tuples_repeated, indexed_slice};

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

    // Overwrite ratatui::Layout
    pub use super::Layout;
}

#[derive(Clone, Copy, Default)]
pub enum Action {
    #[default]
    Noop,
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

#[expect(unused_variables)]
pub trait Component {
    type State;

    async fn handle_event(state: &mut Self::State, event: Event) -> Action {
        match event {
            Event::Key(key_event) => Self::handle_key_events(state, key_event).await?,
            Event::Mouse(mouse_event) => Self::handle_mouse_events(state, mouse_event).await?,
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

pub trait Drawable<Marker> {
    type State = ();

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect);
}

impl<F> Drawable<fn(Rect, &mut Buffer)> for F
where
    F: 'static + FnOnce(Rect, &mut Buffer),
{
    fn draw(self, _: Self::State, frame: &mut Frame, area: Rect) {
        self(area, frame.buffer_mut())
    }
}

impl<'s, S: 's, F> Drawable<fn(&'s S, Rect, &mut Buffer)> for F
where
    F: FnOnce(&'s S, Rect, &mut Buffer),
{
    type State = &'s S;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        self(state, area, frame.buffer_mut())
    }
}

impl<'s, S: 's, F> Drawable<fn(&'s mut S, &mut Frame<'_>, Rect)> for F
where
    F: FnOnce(&'s mut S, &mut Frame, Rect),
{
    type State = &'s mut S;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        self(state, frame, area)
    }
}

pub trait FrameExt {
    fn draw<M, D: Drawable<M>>(&mut self, drawable: D, area: Rect, state: D::State);
    fn draw_stateless<M, D: Drawable<M, State = ()>>(&mut self, drawable: D, area: Rect);
}

impl FrameExt for Frame<'_> {
    fn draw<M, D: Drawable<M>>(&mut self, drawable: D, area: Rect, state: D::State) {
        drawable.draw(state, self, area);
    }
    fn draw_stateless<M, D: Drawable<M, State = ()>>(&mut self, drawable: D, area: Rect) {
        drawable.draw((), self, area);
    }
}

pub trait RectExt: Sized {
    fn reduce(self, size: impl Into<Size>) -> Self;
    fn set_height(self, value: u16) -> Self;
    fn set_width(self, value: u16) -> Self;
    fn offset_x(self, value: i32) -> Self;
    fn offset_y(self, value: i32) -> Self;
}

impl RectExt for Rect {
    fn reduce(mut self, size: impl Into<Size>) -> Self {
        let size: Size = size.into();
        self.width -= size.width;
        self.height -= size.height;

        self
    }

    fn set_height(mut self, value: u16) -> Self {
        self.height = value;
        self
    }

    fn set_width(mut self, value: u16) -> Self {
        self.width = value;
        self
    }

    fn offset_x(self, value: i32) -> Self {
        self.offset(Offset::x(value))
    }

    fn offset_y(self, value: i32) -> Self {
        self.offset(Offset::y(value))
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
