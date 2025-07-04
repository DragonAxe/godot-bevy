use bevy::prelude::*;

use bevy_asset_loader::asset_collection::AssetCollection;
use godot::{
    builtin::{StringName, Vector2},
    classes::{AnimatedSprite2D, Input, Node2D},
};
use godot_bevy::{plugins::core::PhysicsDelta, prelude::*};

use crate::{nodes::player::Player as GodotPlayerNode, GameState};

#[derive(AssetCollection, Resource, Debug)]
pub struct PlayerAssets {
    #[asset(path = "scenes/player.tscn")]
    player_scene: Handle<GodotResource>,
}
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Loading), spawn_player)
            .add_systems(Update, player_on_ready)
            .add_systems(
                Update,
                check_player_death.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                PhysicsUpdate,
                move_player.run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnEnter(GameState::Countdown), setup_player)
            .add_systems(
                PhysicsUpdate,
                move_player.run_if(in_state(GameState::Countdown)),
            );
    }
}

#[derive(Debug, Component)]
pub struct Player {
    speed: f32,
}

#[derive(Debug, Component)]
struct PlayerInitialized;

fn spawn_player(mut commands: Commands, assets: Res<PlayerAssets>) {
    commands
        .spawn_empty()
        .insert(GodotScene::from_handle(assets.player_scene.clone()))
        .insert(Player { speed: 0.0 });
}

fn player_on_ready(
    mut commands: Commands,
    mut player: Query<
        (Entity, &mut Player, &mut GodotNodeHandle),
        (With<Player>, Without<PlayerInitialized>),
    >,
) -> Result {
    if let Ok((entity, mut player_data, mut player)) = player.single_mut() {
        let mut player = player.get::<GodotPlayerNode>();
        player.set_visible(false);
        player_data.speed = player.bind().get_speed();

        // Mark as initialized
        commands.entity(entity).insert(PlayerInitialized);
    }

    Ok(())
}

fn setup_player(
    mut player: Query<(&mut GodotNodeHandle, &mut Transform2D), With<Player>>,
    mut entities: Query<(&Name, &mut GodotNodeHandle), Without<Player>>,
) -> Result {
    if let Ok((mut player, mut transform)) = player.single_mut() {
        let mut player = player.get::<GodotPlayerNode>();
        player.set_visible(true);

        let start_position = entities
            .iter_mut()
            .find_entity_by_name("StartPosition")
            .unwrap()
            .get::<Node2D>()
            .get_position();
        transform.as_godot_mut().origin = start_position;
    }

    Ok(())
}

fn move_player(
    mut player: Query<(&Player, &mut GodotNodeHandle, &mut Transform2D)>,
    physics_delta: Res<PhysicsDelta>,
) -> Result {
    if let Ok((player_data, mut player, mut transform)) = player.single_mut() {
        let player = player.get::<GodotPlayerNode>();
        let screen_size = player.get_viewport_rect().size;
        let mut velocity = Vector2::ZERO;

        if Input::singleton().is_action_pressed("move_right") {
            velocity.x += 1.0;
        }

        if Input::singleton().is_action_pressed("move_left") {
            velocity.x -= 1.0;
        }

        if Input::singleton().is_action_pressed("move_down") {
            velocity.y += 1.0;
        }

        if Input::singleton().is_action_pressed("move_up") {
            velocity.y -= 1.0;
        }

        let mut sprite = player.get_node_as::<AnimatedSprite2D>("AnimatedSprite2D");

        if velocity.length() > 0.0 {
            velocity = velocity.normalized() * player_data.speed;
            sprite.play();

            if velocity.x != 0.0 {
                sprite.set_animation(&StringName::from("walk"));
                sprite.set_flip_v(false);
                sprite.set_flip_h(velocity.x < 0.0);
            } else if velocity.y != 0.0 {
                sprite.set_animation(&StringName::from("up"));
                sprite.set_flip_v(velocity.y > 0.0);
            }
        } else {
            sprite.stop();
        }

        let mut godot_transform = transform.as_godot_mut();
        godot_transform.origin += velocity * physics_delta.delta_seconds;
        godot_transform.origin.x = f32::min(f32::max(0.0, godot_transform.origin.x), screen_size.x);
        godot_transform.origin.y = f32::min(f32::max(0.0, godot_transform.origin.y), screen_size.y);
    }

    Ok(())
}

fn check_player_death(
    mut player: Query<(&mut GodotNodeHandle, &Collisions), With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Ok((mut player, collisions)) = player.single_mut() {
        if collisions.colliding().is_empty() {
            return;
        }

        player.get::<Node2D>().set_visible(false);
        next_state.set(GameState::GameOver);
    }
}
