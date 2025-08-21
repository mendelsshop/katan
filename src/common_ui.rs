use std::{marker::PhantomData, ops::DerefMut};

use bevy::{
    ecs::{
        schedule::ScheduleConfigs,
        system::{ScheduleSystem, StaticSystemParam, SystemParam, SystemParamItem},
    },
    prelude::*,
};

use crate::{
    GameState,
    colors::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON},
    resources_management::{TradingResources, TradingResourcesSlider},
};

// https://www.reddit.com/r/bevy/comments/18xgcm8/comment/kg5jmwp/
// The extra generic here lets me implement for each component
pub trait ButtonInteraction<C: Component>: SystemParam {
    fn interact(&mut self, _: &C);
    fn verify(&mut self, _: &C) -> bool {
        true
    }
}
pub trait SliderButtonInteraction<C: Component>: SystemParam {
    fn increment(&mut self, _: &C);
    fn can_increment(&mut self, _: &C) -> bool {
        true
    }

    fn decrement(&mut self, _: &C);
    fn can_decrement(&mut self, _: &C) -> bool {
        true
    }
}
#[derive(SystemParam)]
pub struct SliderParmeter<
    'w,
    's,
    C: Component,
    T: SystemParam + SliderButtonInteraction<C> + 'static,
>(Query<'w, 's, &'static C>, StaticSystemParam<'w, 's, T>);

#[derive(Debug, Component, Clone, Copy)]
pub struct Up<C>(Entity, PhantomData<C>);
impl<'w, 's, C, T> ButtonInteraction<Up<C>> for SliderParmeter<'w, 's, C, T>
where
    C: Component,
    T: SliderButtonInteraction<C> + SystemParam<Item<'w, 's> = T> + SliderButtonInteraction<C>,
{
    fn interact(&mut self, c: &Up<C>) {
        if let Ok(c) = self.0.get(c.0) {
            self.1.increment(&c)
        }
    }

    fn verify(&mut self, c: &Up<C>) -> bool {
        if let Ok(c) = self.0.get(c.0) {
            self.1.deref_mut().can_increment(&c)
        } else {
            false
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub struct Down<C>(Entity, PhantomData<C>);
impl<
    C: Component,
    T: SliderButtonInteraction<C>
        + for<'w, 's> SystemParam<Item<'w, 's> = T>
        + SliderButtonInteraction<C>,
> ButtonInteraction<Down<C>> for SliderParmeter<'_, '_, C, T>
{
    fn interact(&mut self, c: &Down<C>) {
        if let Ok(c) = self.0.get(c.0) {
            self.1.deref_mut().increment(&c)
        }
    }

    fn verify(&mut self, c: &Down<C>) -> bool {
        if let Ok(c) = self.0.get(c.0) {
            self.1.deref_mut().can_increment(&c)
        } else {
            false
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub struct Value<C>(pub Entity, PhantomData<C>);
pub fn button_system_with_generic<C: Component, T: SystemParam>(
    button_query: Query<'_, '_, (&Interaction, &mut BackgroundColor, &C), (Changed<Interaction>,)>,
    mut param: SystemParamItem<'_, '_, T>,
) where
    for<'w, 's> T::Item<'w, 's>: ButtonInteraction<C>,
{
    for (interaction, mut background_color, c) in button_query {
        if param.verify(c) {
            match *interaction {
                Interaction::Pressed => {
                    param.interact(c);

                    *background_color = PRESSED_BUTTON.into()
                }
                Interaction::Hovered => *background_color = HOVERED_BUTTON.into(),
                Interaction::None => *background_color = NORMAL_BUTTON.into(),
            }
        } else {
            // TODO: have disable button color
            *background_color = NORMAL_BUTTON.into()
        }
    }
}
pub fn slider_buttons_interactions<C, T1, T2>() -> ScheduleConfigs<ScheduleSystem>
where
    C: Component,
    for<'w, 's> T1: SystemParam<Item<'w, 's> = T1>
        + SliderButtonInteraction<C>
        + 'static
        + ButtonInteraction<Up<C>>,
    for<'w, 's> T2: SystemParam<Item<'w, 's> = T2>
        + SliderButtonInteraction<C>
        + 'static
        + ButtonInteraction<Down<C>>,
{
    (
        button_system_with_generic::<Up<C>, SliderParmeter<'_, '_, C, T1>>,
        button_system_with_generic::<Down<C>, SliderParmeter<'_, '_, C, T2>>,
    )
        .into_configs()
}

pub fn slider_bundle<U: Component>(entity: Entity) -> impl Bundle
where
{
    (
        Node {
            display: Display::Grid,
            margin: UiRect::all(Val::Px(3.0)),
            grid_template_rows: vec![GridTrack::auto(), GridTrack::auto(), GridTrack::auto()],
            border: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        BorderColor(Color::BLACK),
        children![
            (
                Up(entity, PhantomData::<U>),
                Node {
                    display: Display::Grid,
                    margin: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                Button,
                BackgroundColor(NORMAL_BUTTON),
                Text::new("+".to_string()),
            ),
            (
                Value(entity, PhantomData::<U>),
                Node {
                    justify_self: JustifySelf::Center,
                    display: Display::Grid,
                    ..default()
                },
            ),
            (
                Down(entity, PhantomData::<U>),
                Node {
                    display: Display::Grid,
                    margin: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                Button,
                BackgroundColor(NORMAL_BUTTON),
                Text::new("-".to_string()),
            )
        ],
    )
}
// #[derive(Debug, Component, Clone, Copy)]
// pub struct PlayButton;
// impl ButtonInteraction<PlayButton> for ResMut<'_, NextState<GameState>> {
//     fn interact(&mut self, f: &PlayButton) {
//         // if let Some(Interaction::Pressed) = interaction {
//         //     self.set(AppState::Game);
//         // }
//     }
// }
