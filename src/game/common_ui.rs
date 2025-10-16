use std::{marker::PhantomData, ops::DerefMut};

use bevy::{
    ecs::{
        schedule::ScheduleConfigs,
        system::{ScheduleSystem, StaticSystemParam, SystemParam, SystemParamItem},
    },
    prelude::*,
};

use super::colors::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};

// https://www.reddit.com/r/bevy/comments/18xgcm8/comment/kg5jmwp/
// The extra generic here lets me implement for each component
pub trait ButtonInteraction<C: Component>: SystemParam {
    fn interact(&mut self, _: &C);
    fn verify(&mut self, _: &C) -> bool {
        true
    }
}
pub trait SpinnerButtonInteraction<C: Component>: SystemParam {
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
pub struct SpinnerParmeter<'w, 's, C: Component, T: SystemParam + 'static>(
    Query<'w, 's, &'static C>,
    StaticSystemParam<'w, 's, T>,
);

#[derive(Debug, Component, Clone, Copy)]
pub struct Up<C>(Entity, PhantomData<C>);
impl<C, T> ButtonInteraction<Up<C>> for SpinnerParmeter<'_, '_, C, T>
where
    C: Component,
    T: SystemParam,
    for<'w, 's> T::Item<'w, 's>: SpinnerButtonInteraction<C>,
{
    fn interact(&mut self, c: &Up<C>) {
        if let Ok(c) = self.0.get(c.0) {
            self.1.increment(c);
        }
    }

    fn verify(&mut self, c: &Up<C>) -> bool {
        if let Ok(c) = self.0.get(c.0) {
            self.1.deref_mut().can_increment(c)
        } else {
            false
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub struct Down<C>(Entity, PhantomData<C>);
impl<C, T> ButtonInteraction<Down<C>> for SpinnerParmeter<'_, '_, C, T>
where
    C: Component,
    T: SystemParam,
    for<'w, 's> T::Item<'w, 's>: SpinnerButtonInteraction<C>,
{
    fn interact(&mut self, c: &Down<C>) {
        if let Ok(c) = self.0.get(c.0) {
            self.1.deref_mut().decrement(c);
        }
    }

    fn verify(&mut self, c: &Down<C>) -> bool {
        if let Ok(c) = self.0.get(c.0) {
            self.1.deref_mut().can_decrement(c)
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

                    *background_color = PRESSED_BUTTON.into();
                }
                Interaction::Hovered => *background_color = HOVERED_BUTTON.into(),
                Interaction::None => *background_color = NORMAL_BUTTON.into(),
            }
        } else {
            // TODO: have disable button color
            *background_color = NORMAL_BUTTON.into();
        }
    }
}
pub fn spinner_buttons_interactions<C, T: SystemParam + 'static>() -> ScheduleConfigs<ScheduleSystem>
where
    C: Component,
    for<'w, 's> T::Item<'w, 's>: SpinnerButtonInteraction<C>,
{
    (
        button_system_with_generic::<Up<C>, SpinnerParmeter<'_, '_, C, T>>,
        button_system_with_generic::<Down<C>, SpinnerParmeter<'_, '_, C, T>>,
    )
        .into_configs()
}

pub fn spinner_bundle<U: Component>(entity: Entity, label: impl Bundle) -> impl Bundle
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
        BorderColor::all(Color::BLACK),
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
                Node {
                    display: Display::Grid,
                    grid_template_columns: vec![GridTrack::auto(), GridTrack::auto()],
                    ..default()
                },
                children![
                    (
                        Value(entity, PhantomData::<U>),
                        Text::new(String::new()),
                        Node {
                            display: Display::Grid,
                            ..default()
                        },
                    ),
                    label
                ]
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
