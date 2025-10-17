mod edit;
mod new_job;
mod rename;

use std::any::TypeId;

use crate::animation::AnimationTicker;
use crate::app::App;
use crate::ui::common::AnimatedIsland;
use crate::ui::prelude::*;

pub use edit::EditPopup;
pub use new_job::NewJobPopup;
pub use rename::RenamePopup;

#[derive(Default)]
pub struct SharedPopupState<Popups> {
    pub anim: AnimationTicker,
    active_popup: Option<TypeId>,
    quitting: bool,
    marker: PhantomData<Popups>,
}

impl<Popups: PopupBundle> SharedPopupState<Popups> {
    pub fn update(state: &mut App) {
        if state.popup.anim.stopped() && state.popup.quitting {
            if let Some(popup) = state.popup.active_popup.take() {
                let iter = type_iter!(<T, 'a> once (state: &'a mut App) {
                    T::on_mount(state);
                });

                _ = Popups::find(popup, iter);
            }

            state.popup.quitting = false;
            state.popup.active_popup = None;
            state.popup.anim.reverse();
        }
    }

    pub fn open<T: Popup + 'static>(state: &mut App) {
        if state.popup.active_popup.is_some() || state.popup.quitting {
            // TODO: Warn about action not possible
            return;
        }

        if !Popups::get::<T>() {
            // TODO: Warn about popup isn't part of the component
            return;
        }

        state.popup.active_popup = Some(TypeId::of::<T>());
        state.popup.anim.len = T::DURATION;
        state.popup.anim.start();

        T::on_mount(state);
    }

    pub fn close(&mut self) {
        if self.active_popup.is_none() || self.quitting {
            // TODO: Warn about action not possible
            return;
        }

        self.anim.reverse();
        self.anim.start();
        self.quitting = true;
    }
}

impl<Popups: PopupBundle> Component for SharedPopupState<Popups> {
    type State = App;

    async fn handle_event(state: &mut Self::State, event: Event) -> Action {
        if state.popup.quitting {
            return Action::Intercept;
        }

        let Some(popup) = state.popup.active_popup else {
            return Action::Noop;
        };

        let auto_close_event = type_iter!(<T: Popup> once () -> bool {
            T::AUTO_CLOSE_EVENT
        });

        let Some(auto_close_event) = Popups::find(popup, auto_close_event) else {
            // TODO: Warn about popup not found
            return Action::Noop;
        };

        let action = if auto_close_event {
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q') | KeyCode::Esc,
                    ..
                }) => Action::Quit,
                _ => Action::Noop,
            }
        } else {
            Action::Noop
        };

        let action = if action == Action::Noop {
            let iter = type_iter!(<T, 'a> once async (state: &'a mut App, event: Event) -> Action {
                <T as Component>::handle_event(state, event).await
            });

            let Some(action) = Popups::find_async(popup, iter).await else {
                // TODO: Warn about popup not found
                return Action::Noop;
            };

            action
        } else {
            action
        };

        match action {
            Action::Noop => Action::Intercept,
            Action::Quit => {
                state.popup.anim.reverse();
                state.popup.anim.start();
                state.popup.quitting = true;
                Action::Tick
            }
            _ => action,
        }
    }

    fn draw(state: &mut Self::State, frame: &mut Frame, area: Rect) {
        let Some(popup) = state.popup.active_popup else {
            return;
        };

        let iter = type_iter!(<T, 'a, 'f, 'fi> once (state: &'a mut App, frame: &'f mut Frame<'fi>, area: Rect) {
            let island = PopupBuilder::new::<T>();
            let island = T::build(island, state, area);
            island.draw(state, frame);
        });

        Popups::find(popup, iter);
    }
}

pub struct PopupBuilder<'a> {
    area: Rect,
    island: AnimatedIsland<'a, ComponentDraw<'a, App>, ComponentDraw<'a, App>>,
}

impl<'a> PopupBuilder<'a> {
    pub fn new<'i, P: Popup>() -> PopupBuilder<'i> {
        PopupBuilder {
            area: Rect::ZERO,
            island: AnimatedIsland::new(P::draw),
        }
    }

    pub fn reserve(mut self, area: Rect) -> Self {
        self.area = area;
        self
    }

    pub fn direction(mut self, side: Side) -> Self {
        self.island = self.island.direction(side);
        self
    }

    pub fn border_style(mut self, style: impl Into<Style>) -> Self {
        self.island = self.island.border_style(style);
        self
    }

    fn draw(self, app: &'a mut App, frame: &mut Frame) {
        self.island
            .draw((app.popup.anim.tick(), app), frame, self.area);
    }
}

pub trait Popup: Component<State = App> + Sized {
    const DURATION: usize = 20;
    const AUTO_CLOSE_EVENT: bool = true;

    fn build<'a: 'app, 'app>(
        island: PopupBuilder<'a>,
        app: &'app mut App,
        area: Rect,
    ) -> PopupBuilder<'a>;
}

pub trait PopupBundle {
    fn get<T: 'static>() -> bool;

    fn find<It: TypeIteratorOnce>(target: TypeId, iter: It) -> Option<It::Item>;
    async fn find_async<It: AsyncTypeIteratorOnce>(target: TypeId, iter: It) -> Option<It::Item>;
}

impl_variadics::impl_variadics!(
    1..10 "T*" => {
        impl<#(#T0: Popup + 'static,)*> PopupBundle for (#(#T0,)*) {
            fn get<T: 'static>() -> bool {
                #(
                    #[allow(non_snake_case)]
                    let #T0: TypeId = TypeId::of::<#T0>();
                )*

                let _ty = TypeId::of::<T>();

                #[allow(clippy::nonminimal_bool)]
                { false #(|| _ty == #T0)* }
            }

            fn find<It: TypeIteratorOnce>(target: TypeId, _iter: It) -> Option<It::Item>
            {
                #(
                    #[allow(non_snake_case)]
                    let #T0: TypeId = TypeId::of::<#T0>();
                )*

                match target {
                    #(ty if ty == #T0 => Some(_iter.once::<#T0>()),)*
                    _ => None
                }
            }

            async fn find_async<It: AsyncTypeIteratorOnce>(target: TypeId, _iter: It) -> Option<It::Item>
            {
                #(
                    #[allow(non_snake_case)]
                    let #T0: TypeId = TypeId::of::<#T0>();
                )*

                match target {
                    #(ty if ty == #T0 => Some(_iter.once::<#T0>().await),)*
                    _ => None
                }
            }
        }
    };
);

pub fn action_buttons<const N: usize>(
    buttons: [(&'static str, Color); N],
    area: Rect,
    buf: &mut Buffer,
) {
    let mut constraints = [Constraint::Length(0); N];

    for i in 0..N {
        const BUTTON_PADDING: u16 = 6;
        constraints[i] = Constraint::Length(buttons[i].0.len() as u16 + BUTTON_PADDING);
    }

    let area = ratatui::layout::Layout::horizontal(constraints)
        .flex(Flex::End)
        .split(area);

    for (i, (item, color)) in buttons.into_iter().enumerate() {
        Text::raw(item)
            .centered()
            .bold()
            .render(common::round_button(color, area[i], buf), buf);
    }
}
