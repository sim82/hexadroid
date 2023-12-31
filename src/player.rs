use crate::{prelude::*, ship::ShipMarker};
use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerMarker;

#[derive(Event)]
pub struct PlayerTakeover {
    pub player: Entity,
    pub target: Entity,
}

fn player_takeover_system(
    mut commands: Commands,
    mut events: EventReader<PlayerTakeover>,
    ai_query: Query<Entity, With<AiMarker>>,
    child_query: Query<&Children>,
    // mut ship_query: Query<(&mut ShipInput, &mut AttackRequest)>,
    parent_query: Query<&Parent>,
    ship_query: Query<Entity, With<ShipMarker>>,
) {
    for PlayerTakeover { player, target } in events.read() {
        if let Ok(parent) = parent_query.get(*player) {
            // if let Ok((mut ship_input, mut attack_request)) = ship_query.get_mut(parent.get()) {
            //     *ship_input = default();
            //     *attack_request = default();
            // }
            if ship_query.contains(parent.get()) {
                commands.entity(parent.get()).remove_children(&[*player]);
                commands.entity(parent.get()).despawn_recursive(); //insert(Despawn::ThisFrame);
                info!("despawn ship");
            }
        }
        let Ok(children) = child_query.get(*target) else {
            continue;
        };

        for child in children.iter() {
            if !ai_query.contains(*child) {
                continue;
            }
            commands.entity(*child).despawn_recursive();
        }
        commands.entity(*target).clear_children().add_child(*player);
    }
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerTakeover>()
            .add_systems(PostUpdate, player_takeover_system);
    }
}
