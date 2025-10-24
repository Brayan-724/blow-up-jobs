#![expect(dead_code, reason = "lib in progress")]

/// Create type iterators as closures. Allowing to get non-dyn-safe info from traits.
///
/// Syntax:
/// ```ignore
/// // Common syntax (not valid)
/// type_iter!(<T: TRAIT> () {});
/// type_iter!(<T: TRAIT> (arg: type...) {});
/// type_iter!(<T: TRAIT> () -> RET {});
/// type_iter!(<T: TRAIT, 'lifetimes...> () {});
/// // Sync find iterator
/// type_iter!(CONSTRAINT once CLOSURE);
/// // Async find iterator
/// type_iter!(CONSTRAINT once async CLOSURE);
/// ```
///
/// Example:
/// ```
/// // Create a common trait
/// trait Entity {
///     const NAME: &'static str;
/// }
///
/// // Define structs used for iterator
/// struct Attacker;
/// struct Victim;
///
/// // Implement common trait on target structs
/// impl Entity for Attacker {
///     const NAME: &'static str = "The attacker";
/// }
///
/// impl Entity for Victim {
///     const NAME: &'static str = "The victim";
/// }
///
/// // Create iterator
/// let iter = type_iter!(<T: Entity> once () -> &'static str {
///     T::NAME
/// });
///
/// // Select runtime target
/// let target = TypeId::of::<Attacker>();
///
/// // Find desired value
/// type Targets = (Attacker, Victim);
/// let target_name = Targets::find(target, iter);
///
/// assert_eq!(target_name, "The attacker");
/// ```
macro_rules! type_iter {
    (<$T:ident $(: $Trait:ty)? $(, $lf:lifetime)*> once ($($arg:ident: $ty:ty),*$(,)?) $(-> $ret:ty)? $body:block) => {{
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

    (<$T:ident $(: $Trait:ty)? $(, $lf:lifetime)*> once async ($($arg:ident: $ty:ty),*$(,)?) $(-> $ret:ty)? $body:block) => {{
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

use crate::variadicts::{all_tuples_repeated, dual_combination, indexed_slice};

#[doc(hidden)]
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

/// Action result per tick.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Action {
    /// Don't operate, lowest priority
    #[default]
    Noop,
    /// Don't operate but takes the tick
    Intercept,
    /// Execute draw tick
    Tick,
    /// Close app or popup
    Quit,
}

impl std::ops::BitOr for Action {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match self {
            Self::Noop => rhs,
            _ => self,
        }
    }
}

impl std::ops::BitOrAssign for Action {
    fn bitor_assign(&mut self, rhs: Self) {
        if *self == Action::Noop {
            *self = rhs;
        }
    }
}

impl std::ops::FromResidual<Action> for Action {
    fn from_residual(residual: Action) -> Self {
        residual
    }
}

impl std::ops::Try for Action {
    type Output = ();
    type Residual = Action;

    fn from_output((): Self::Output) -> Self {
        Self::Noop
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Action::Noop => std::ops::ControlFlow::Continue(()),
            _ => std::ops::ControlFlow::Break(self),
        }
    }
}

/// A component is a struct that can handle user events and draw with
/// an attached state.
///
/// Example:
/// ```
///# use prelude::*;
/// struct GlobalState {
///     counter: usize,
/// }
///
/// struct Counter;
///
/// impl Component for Counter {
///     type State = GlobalState;
///
///     async fn handle_key_events(state: &mut Self::State, key: KeyEvent) -> Action {
///         match key.code {
///             KeyCode::Char('+') => {
///                 state.counter = state.saturating_add(1);
///                 Action::Tick
///             }
///             KeyCode::Char('-') => {
///                 state.counter = state.saturating_sub(1);
///                 Action::Tick
///             }
///             _ => Action::Noop,
///         }
///     }
///
///     fn draw(state: &mut Self::State, frame: &mut Frame, area: Rect) {
///         state.counter.to_string().to_text().render(area, frame.buffer_mut());
///     }
/// }
/// ```
#[expect(unused_variables, reason = "default templating")]
pub trait Component {
    type State;

    /// Called when component will start a render lifecycle
    fn on_mount(state: &mut Self::State) {}
    /// Called when component will end a render lifecycle
    fn on_destroy(state: &mut Self::State) {}

    /// Entry point for all user events
    async fn handle_event(state: &mut Self::State, event: Event) -> Action {
        match event {
            Event::Key(key_event) => Self::handle_key_events(state, key_event).await?,
            Event::Mouse(mouse_event) => Self::handle_mouse_events(state, mouse_event).await?,
            Event::Resize(_, _) => Action::Tick?,
            _ => {}
        }

        Self::propagate_event(state, event).await
    }

    /// If the event was not handled, then propagate to other components.
    ///
    /// > This method may be replaced by [crate::type_bundle]
    async fn propagate_event(state: &mut Self::State, event: Event) -> Action {
        Action::Noop
    }

    /// Handle user key events
    async fn handle_key_events(state: &mut Self::State, key: KeyEvent) -> Action {
        Action::Noop
    }

    /// Handle user mouse events
    async fn handle_mouse_events(state: &mut Self::State, mouse: MouseEvent) -> Action {
        Action::Noop
    }

    /// Draw the component with attached state
    fn draw(state: &mut Self::State, frame: &mut Frame, area: Rect);
}

/// Function signature of [`Component::draw`]
pub type ComponentDraw<'a, S> = for<'s, 'f> fn(&'a mut S, &'s mut Frame<'f>, Rect);

/// Wrapper for [`ratatui::layout::Layout`] that add generics for compile-time info about layout
pub struct Layout<const N: usize, T> {
    inner: ratatui::layout::Layout,
    _marker: PhantomData<T>,
}

impl<const N: usize, T: ContraintBundle<N>> Layout<N, T> {
    pub fn horizontal(constraints: T) -> Self {
        Layout {
            inner: ratatui::layout::Layout::horizontal(constraints.into_vec()),
            _marker: PhantomData,
        }
    }

    pub fn vertical(constraints: T) -> Self {
        Layout {
            inner: ratatui::layout::Layout::vertical(constraints.into_vec()),
            _marker: PhantomData,
        }
    }

    /// The `flex` method  allows you to specify the flex behavior of the layout.
    ///
    /// # Arguments
    ///
    /// * `flex`: A [`Flex`] enum value that represents the flex behavior of the layout. It can be
    ///   one of the following:
    ///   - [`Flex::Legacy`]: The last item is stretched to fill the excess space.
    ///   - [`Flex::Start`]: The items are aligned to the start of the layout.
    ///   - [`Flex::Center`]: The items are aligned to the center of the layout.
    ///   - [`Flex::End`]: The items are aligned to the end of the layout.
    ///   - [`Flex::SpaceAround`]: The items are evenly distributed with equal space around them.
    ///   - [`Flex::SpaceBetween`]: The items are evenly distributed with equal space between them.
    ///
    /// # Examples
    ///
    /// In this example, the items in the layout will be aligned to the start.
    ///
    /// ```rust
    /// use crate::ui::prelude::{Constraint::*, Flex, Layout};
    ///
    /// let layout = Layout::horizontal([Length(20), Length(20), Length(20)]).flex(Flex::Start);
    /// ```
    ///
    /// In this example, the items in the layout will be stretched equally to fill the available
    /// space.
    ///
    /// ```rust
    /// use crate::ui::prelude::{Constraint::*, Flex, Layout};
    ///
    /// let layout = Layout::horizontal([Length(20), Length(20), Length(20)]).flex(Flex::Legacy);
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn flex(mut self, flex: Flex) -> Self {
        self.inner = self.inner.flex(flex);
        self
    }

    // TODO: doc
    pub fn split(&self, area: Rect) -> T::Out {
        T::from_rects(self.inner.split(area))
    }
}

/// see [`crate::type_bundle`]
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

/// see [`crate::type_bundle`]
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

/// Common trait for all kind of data that can be draw and has
/// compile-time info about its state.
/// It uses `Marker` to implement collidable types.
pub trait Drawable<'a, Marker> {
    type State = ();
    const STATEFUL: bool = false;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect);
}

// Unit type does nothing
impl Drawable<'_, ()> for () {
    fn draw(self, (): Self::State, _: &mut Frame, _: Rect) {}
}

// Ratatui native widgets
impl<'a, S: 'a + Widget> Drawable<'a, fn(S) -> bool> for S {
    fn draw(self, (): Self::State, frame: &mut Frame, area: Rect) {
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

// Different kinds of functions
impl<'a, F> Drawable<'a, fn(Rect, &mut Buffer)> for F
where
    F: 'a + FnOnce(Rect, &mut Buffer),
{
    fn draw(self, (): Self::State, frame: &mut Frame, area: Rect) {
        self(area, frame.buffer_mut());
    }
}

impl<'a, 's, S: 's, F> Drawable<'a, fn(&'s S, Rect, &mut Buffer)> for F
where
    F: 'a + FnOnce(&'s S, Rect, &mut Buffer),
{
    type State = &'s S;
    const STATEFUL: bool = true;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        self(state, area, frame.buffer_mut());
    }
}

impl<'a, 's, S: 's, F> Drawable<'a, fn(&'s mut S, Rect, &mut Buffer)> for F
where
    F: 'a + FnOnce(&'s mut S, Rect, &mut Buffer),
{
    type State = &'s mut S;
    const STATEFUL: bool = true;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        self(state, area, frame.buffer_mut());
    }
}

impl<'a, F> Drawable<'a, fn(&mut Frame<'_>, Rect)> for F
where
    F: 'a + FnOnce(&mut Frame, Rect),
{
    fn draw(self, (): Self::State, frame: &mut Frame, area: Rect) {
        self(frame, area);
    }
}

impl<'a, 's, S: 's, F> Drawable<'a, fn(&'s S, &mut Frame<'_>, Rect)> for F
where
    F: 'a + FnOnce(&'s S, &mut Frame, Rect),
{
    type State = &'s S;
    const STATEFUL: bool = true;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        self(state, frame, area);
    }
}

impl<'a, 's, S: 's, F> Drawable<'a, fn(&'s mut S, &mut Frame<'_>, Rect)> for F
where
    F: 'a + FnOnce(&'s mut S, &mut Frame, Rect),
{
    type State = &'s mut S;
    const STATEFUL: bool = true;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        self(state, frame, area);
    }
}

impl<'s, S: 's> Drawable<'static, &'s ComponentDraw<'s, S>> for ComponentDraw<'s, S> {
    type State = &'s mut S;
    const STATEFUL: bool = true;

    fn draw(self, state: Self::State, frame: &mut Frame, area: Rect) {
        self(state, frame, area);
    }
}

// Ratatui extensions

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
    fn reduce_offset(self, size: impl Into<Size>) -> Self;
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

    fn reduce_offset(mut self, size: impl Into<Size>) -> Self {
        let size: Size = size.into();
        self.x = self.x.saturating_add(size.width);
        self.y = self.y.saturating_add(size.height);
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
    fn both(value: i32) -> Self;
}

impl OffsetExt for Offset {
    fn x(value: i32) -> Self {
        Self { x: value, y: 0 }
    }

    fn y(value: i32) -> Self {
        Self { y: value, x: 0 }
    }

    fn both(value: i32) -> Self {
        Self { y: value, x: value }
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

/// Value that can be operated as an arithmetic number
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

/// Cast any number to any other number
pub trait Cast<T> {
    fn cast(self) -> T;
}

macro_rules! impl_cast {
    ($a:tt, $b:tt) => {
        impl_cast!(@ $a, $b);
        impl_cast!(@ $b, $a);
    };

    (@ (u, $a:ty), (i, $b:ty)) => {
        impl Cast<$b> for $a {
            fn cast(self) -> $b {
                self.cast_signed() as $b
            }
        }
    };

    (@ (i, $a:ty), (u, $b:ty)) => {
        impl Cast<$b> for $a {
            fn cast(self) -> $b {
                self.max(0).unsigned_abs() as $b
            }
        }
    };

    (@ (f, $a:ty), (f, $b:ty)) => {
        impl Cast<$b> for $a {
            fn cast(self) -> $b {
                self as $b
            }
        }
    };

    (@ (f, $a:ty), ($_:ident, $b:ty)) => {
        impl Cast<$b> for $a {
            fn cast(self) -> $b {
                self.trunc() as $b
            }
        }
    };

    (@ ($_:ident, $a:ty), ($__:ident, $b:ty)) => {
        impl Cast<$b> for $a {
            fn cast(self) -> $b {
                self as $b
            }
        }
    };
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
const _: () = {
    #[rustfmt::skip]
    dual_combination!(impl_cast, [
        (u, usize), (u, u8), (u, u16), (u, u32), (u, u64),
        (i, isize), (i, i8), (i, i16), (i, i32), (i, i64),
        (f, f32  ), (f, f64),
    ]);
};

pub trait Casted: Sized {
    fn casted<T>(self) -> T
    where
        Self: Cast<T>,
    {
        self.cast()
    }
}

impl<T> Casted for T {}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

/// Find callback for [`crate::type_bundle`]
pub trait TypeIteratorOnce {
    type Item;

    fn once<T: popup::Popup>(self) -> Self::Item;
}

/// Asyncronous find callback for [`crate::type_bundle`]
pub trait AsyncTypeIteratorOnce {
    type Item;

    async fn once<T: popup::Popup>(self) -> Self::Item;
}
