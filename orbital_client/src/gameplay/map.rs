use std::sync::Arc;

use cgmath::{Bounded, InnerSpace, MetricSpace, VectorSpace};
use rand::rngs::ThreadRng;
use rand::Rng;
use serde::__private::ser::constrain;
use serde::{Deserialize, Serialize};

use crate::graphics::renderer::{Camera, RenderState, TextHAlignment, TextVAlignment};
use crate::graphics::ui::*;
use crate::graphics::window::{FrameState, KeyCode};
use crate::{types::*, State};

use super::game_state::Tick;

const SUNSET_CORAL: Vec4 = Vec4::new(0.749, 0.290, 0.184, 1.0);
const WARM_TERRACOTTA: Vec4 = Vec4::new(0.843, 0.463, 0.263, 1.0);
const PEACHY_SAND: Vec4 = Vec4::new(0.929, 0.831, 0.666, 1.0);
const BLUSH_SALMON: Vec4 = Vec4::new(0.894, 0.651, 0.447, 1.0);
const RUSTIC_BRICK: Vec4 = Vec4::new(0.721, 0.435, 0.314, 1.0);
const DEEP_CHESTNUT: Vec4 = Vec4::new(0.451, 0.243, 0.224, 1.0);
const DARK_UMBER: Vec4 = Vec4::new(0.243, 0.152, 0.192, 1.0);
const CRIMSON_FIRE: Vec4 = Vec4::new(0.635, 0.149, 0.200, 1.0);
const VIBRANT_TOMATO: Vec4 = Vec4::new(0.894, 0.231, 0.267, 1.0);
const ZESTY_ORANGE: Vec4 = Vec4::new(0.967, 0.463, 0.133, 1.0);
const SUNNY_MARIGOLD: Vec4 = Vec4::new(0.996, 0.682, 0.204, 1.0);
const BRIGHT_DAFFODIL: Vec4 = Vec4::new(0.996, 0.905, 0.380, 1.0);
const FRESH_MEADOW: Vec4 = Vec4::new(0.388, 0.780, 0.302, 1.0);
const LUSH_FOREST: Vec4 = Vec4::new(0.243, 0.537, 0.282, 1.0);
const DEEP_EMERALD: Vec4 = Vec4::new(0.149, 0.361, 0.259, 1.0);
const OCEAN_MYSTERY: Vec4 = Vec4::new(0.098, 0.235, 0.243, 1.0);
const MIDNIGHT_BLUE: Vec4 = Vec4::new(0.071, 0.306, 0.537, 1.0);
const SKY_AZURE: Vec4 = Vec4::new(0.000, 0.600, 0.859, 1.0);
const AQUA_SURF: Vec4 = Vec4::new(0.173, 0.910, 0.960, 1.0);
const PURE_WHITE: Vec4 = Vec4::new(1.000, 1.000, 1.000, 1.0);
const SOFT_LAVENDER: Vec4 = Vec4::new(0.753, 0.796, 0.863, 1.0);
const STORMY_SLATE: Vec4 = Vec4::new(0.545, 0.608, 0.702, 1.0);
const MUTED_INDIGO: Vec4 = Vec4::new(0.353, 0.412, 0.533, 1.0);
const TWILIGHT_NAVY: Vec4 = Vec4::new(0.227, 0.267, 0.400, 1.0);
const CHARCOAL_NIGHT: Vec4 = Vec4::new(0.149, 0.169, 0.267, 1.0);
const DEEP_ECLIPSE: Vec4 = Vec4::new(0.094, 0.078, 0.145, 1.0);
const ELECTRIC_FUCHSIA: Vec4 = Vec4::new(1.000, 0.000, 0.267, 1.0);
const MYSTIC_PLUM: Vec4 = Vec4::new(0.408, 0.224, 0.424, 1.0);
const RADIANT_ORCHID: Vec4 = Vec4::new(0.710, 0.329, 0.533, 1.0);
const CORAL_PINK: Vec4 = Vec4::new(0.965, 0.459, 0.478, 1.0);
const PASTEL_ROSE: Vec4 = Vec4::new(0.910, 0.718, 0.584, 1.0);
const EARTHY_ADOBE: Vec4 = Vec4::new(0.761, 0.522, 0.412, 1.0);

#[derive(Serialize, Deserialize, Debug)]
pub struct MapFile {}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Team {
    A,
    B,
}

impl Team {
    pub fn color(&self) -> Vec4 {
        match self {
            Team::A => SKY_AZURE,
            Team::B => ELECTRIC_FUCHSIA,
        }
    }

    pub fn opposite(&self) -> Team {
        match self {
            Team::A => Team::B,
            Team::B => Team::A,
        }
    }
}

#[derive(Clone)]
pub struct PlayerActionAttack {
    sources: Vec<i32>, // index into world, this wont work if you add worlds !! Sligh hacky of doing it is to use the position instead
    target: i32, // index into world, this wont work if you add worlds !! Sligh hacky of doing it is to use the position instead
}

#[derive(Clone)]
pub enum PlayerAction {
    None,
    Attack(PlayerActionAttack),
}

pub struct ClientState {
    camera: Camera,
    team: Team,
    next_action: PlayerAction,
}

pub struct Squadron {
    world_pos: Vec2,
    ship_count: i32,
    team: Team,
    source_world: i32,
    dest_world: i32,
    distance_in_ticks: i32,
    travel_in_ticks: i32,
}

pub enum PlanetSize {
    Small,
    Medium,
    Large,
}

impl PlanetSize {
    pub fn size(&self) -> i32 {
        match self {
            PlanetSize::Small => 8,
            PlanetSize::Medium => 12,
            PlanetSize::Large => 19,
        }
    }

    pub fn random(rng: &mut ThreadRng) -> PlanetSize {
        match rng.gen_range(0..3) {
            0 => PlanetSize::Small,
            1 => PlanetSize::Medium,
            2 => PlanetSize::Large,
            _ => panic!("Invalid random number"),
        }
    }
}

pub struct World {
    pos: Vec2i,
    ship_count: i32,
    size: PlanetSize,
    team: Team,
}

pub struct GameMap {
    size: Vec2i,
    worlds: Vec<World>,
    selected_worlds: Vec<i32>,
    squadrons: Vec<Squadron>,
    client: ClientState,
    start_mouse_pos: Vec2,
    end_mouse_pos: Vec2,
    is_dragging: bool,
    ui: UIMaster,
}

impl GameMap {
    pub fn new() -> Self {
        Self {
            size: Vec2i::new(0, 0),
            worlds: Vec::new(),
            selected_worlds: Vec::new(),
            squadrons: Vec::new(),
            client: ClientState {
                camera: Camera::new(0, 0),
                team: Team::A,
                next_action: PlayerAction::None,
            },
            start_mouse_pos: Vec2::new(0.0, 0.0),
            end_mouse_pos: Vec2::new(0.0, 0.0),
            is_dragging: false,
            ui: UIMaster::new(),
        }
    }

    pub fn main_menu(size: Vec2i, rs: &mut RenderState) -> Self {
        Self {
            size: size,
            worlds: Vec::new(),
            selected_worlds: Vec::new(),
            squadrons: Vec::new(),
            client: ClientState {
                camera: Camera::new(0, 0),
                team: Team::A,
                next_action: PlayerAction::None,
            },
            start_mouse_pos: Vec2::new(0.0, 0.0),
            end_mouse_pos: Vec2::new(0.0, 0.0),
            is_dragging: false,
            ui: GameMap::ui_main_menu(),
        }
    }

    pub fn ui_main_menu() -> UIMaster {
        let mut ui = UIMaster::new();
        let mut stack = Box::new(UIStackPaneContainer::new_vertical());
        stack.add_child(Box::new(UIButton::new("Play")));
        stack.add_child(Box::new(UIButton::new("Options")));
        stack.add_child(Box::new(UIButton::new("Exit")));

        ui.add_child(
            stack,
            UIBlockContainerContraints {
                x_constraint: UIBlockContainerXConstraint::CENTER,
                y_constraint: UIBlockContainerYConstraint::CENTER,
            },
        );

        ui
    }

    pub fn new_random(size: Vec2i, rs: &mut RenderState) -> Self {
        let mut rng = rand::thread_rng();
        let world_count = rng.gen_range(4..18);

        let mut worlds = Vec::with_capacity(world_count);
        let half_size = size / 2;

        for _ in 0..world_count {
            let ship_count = rng.gen_range(10..31);
            let team = if rng.gen_bool(0.5) { Team::A } else { Team::B };
            worlds.push(World {
                pos: Vec2i::new(
                    rng.gen_range(-half_size.x..half_size.x),
                    rng.gen_range(-half_size.y..half_size.y),
                ),
                size: PlanetSize::random(&mut rng),
                ship_count,
                team,
            });
        }

        Self {
            size: size,
            worlds: worlds,
            selected_worlds: Vec::new(),
            squadrons: Vec::new(),
            client: ClientState {
                camera: Camera::new(0, 0),
                team: Team::A,
                next_action: PlayerAction::None,
            },
            start_mouse_pos: Vec2::new(0.0, 0.0),
            end_mouse_pos: Vec2::new(0.0, 0.0),
            is_dragging: false,
            ui: GameMap::test_ui_stuffies(),
        }
    }

    pub fn ui_network_test() -> UIMaster {
        let mut ui = UIMaster::new();
        let mut left_block = Box::new(UIBlockContainer::new_from_percent(0.5, 1.0));

   
        left_block.add_child(
            Box::new(UIButton::new("Left")),
            UIBlockContainerContraints {
                x_constraint: UIBlockContainerXConstraint::CENTER,
                y_constraint: UIBlockContainerYConstraint::CENTER,
            },
        );

        let mut right_block = Box::new(UIBlockContainer::new_from_percent(0.5, 1.0));
        right_block.add_child(
            Box::new(UIButton::new("Right")),
            UIBlockContainerContraints {
                x_constraint: UIBlockContainerXConstraint::CENTER,
                y_constraint: UIBlockContainerYConstraint::CENTER,
            },
        );

        ui.add_child(
            left_block,
            UIBlockContainerContraints {
                x_constraint: UIBlockContainerXConstraint::LEFT,
                y_constraint: UIBlockContainerYConstraint::CENTER,
            },
        );

        ui.add_child(
            right_block,
            UIBlockContainerContraints {
                x_constraint: UIBlockContainerXConstraint::RIGHT,
                y_constraint: UIBlockContainerYConstraint::CENTER,
            },
        );

        ui
    }

    pub fn test_ui_stuffies() -> UIMaster {
        let mut ui = UIMaster::new();

        let mut left_stack = Box::new(UIStackPaneContainer::new_vertical());
        let s = "File";
        fn on_click_file(state : &mut State) {
            println!("File clicked");
        }

        left_stack.add_child(Box::new(UIButton::new_callback(
            "File",
            Box::new(on_click_file),
        )));
        left_stack.add_child(Box::new(UIButton::new("Edit")));
        left_stack.add_child(Box::new(UIButton::new("View")));

        let mut center_stack = Box::new(UIStackPaneContainer::new());
        center_stack.add_child(Box::new(UIButton::new("Play")));
        center_stack.add_child(Box::new(UIButton::new("Pause")));
        center_stack.add_child(Box::new(UIButton::new("Stop")));

        let mut right_stack = Box::new(UIStackPaneContainer::new());
        right_stack.add_child(Box::new(UIButton::new("Exit")));
        right_stack.add_child(Box::new(UIButton::new("About")));
        right_stack.add_child(Box::new(UIButton::new("Help")));

        ui.add_child(
            left_stack,
            UIBlockContainerContraints {
                x_constraint: UIBlockContainerXConstraint::LEFT,
                y_constraint: UIBlockContainerYConstraint::TOP,
            },
        );

        ui.add_child(
            center_stack,
            UIBlockContainerContraints {
                x_constraint: UIBlockContainerXConstraint::CENTER,
                y_constraint: UIBlockContainerYConstraint::TOP,
            },
        );

        ui.add_child(
            right_stack,
            UIBlockContainerContraints {
                x_constraint: UIBlockContainerXConstraint::RIGHT,
                y_constraint: UIBlockContainerYConstraint::TOP,
            },
        );

        ui
    }

    pub fn get_next_action(&mut self) -> PlayerAction {
        let c = self.client.next_action.clone();
        self.client.next_action = PlayerAction::None;
        return c;
    }

    pub fn handle_player_move(&mut self, player_action: &PlayerAction, player_team: Team) {
        match player_action {
            PlayerAction::None => {}
            PlayerAction::Attack(attack) => {
                for source in &attack.sources {
                    let ship_count = self.worlds[*source as usize].ship_count;
                    let source_world = &self.worlds[*source as usize];
                    let target_world = &self.worlds[attack.target as usize];

                    let dir = ivec_to_vec(target_world.pos - source_world.pos).normalize();
                    let pos = ivec_to_vec(source_world.pos);
                    let distance = magnitude_i32(source_world.pos - target_world.pos);
                    self.squadrons.push(Squadron {
                        world_pos: pos,
                        ship_count: ship_count,
                        team: player_team,
                        source_world: *source,
                        dest_world: attack.target,
                        distance_in_ticks: distance,
                        travel_in_ticks: 0,
                    });

                    self.worlds[*source as usize].ship_count = 0;
                }
            }
        }
    }

    pub fn tick(&mut self, tick: &Tick) {
        self.handle_player_move(&tick.player_a_move, Team::A);
        self.handle_player_move(&tick.player_b_move, Team::B);

        if tick.tick_number % 30 == 0 {
            for world in &mut self.worlds {
                world.ship_count += 1;
            }
        }

        for squadron in &mut self.squadrons {
            squadron.travel_in_ticks += 1;
            if squadron.travel_in_ticks == squadron.distance_in_ticks {
                let dest_world = &mut self.worlds[squadron.dest_world as usize];

                if dest_world.team == squadron.team {
                    dest_world.ship_count += squadron.ship_count;
                } else {
                    dest_world.ship_count -= squadron.ship_count;
                }

                if dest_world.ship_count < 0 {
                    dest_world.team = squadron.team;
                    dest_world.ship_count = -dest_world.ship_count;
                }
            }
        }

        self.squadrons
            .retain(|squadron| squadron.travel_in_ticks < squadron.distance_in_ticks)
    }

    pub fn get_world_under_point(&self, point: Vec2) -> Option<i32> {
        for (i, world) in self.worlds.iter().enumerate() {
            let world_pos = ivec_to_vec(world.pos);
            let world_size = world.size.size() as f32;

            if is_point_on_circle(world_pos, world_size, point) {
                return Some(i as i32);
            }
        }

        None
    }

    pub fn get_worlds_under_box(&self, min: Vec2, max: Vec2) -> Vec<i32> {
        let mut result = Vec::new();

        for (i, world) in self.worlds.iter().enumerate() {
            let world_pos = ivec_to_vec(world.pos);
            let world_size = world.size.size() as f32;

            if is_box_overlapping_circle(min, max, world_pos, world_size) {
                result.push(i as i32);
            }
        }

        result
    }

    pub fn frame_update_and_render(&mut self, state: &mut State) {
        let rs = &mut state.rs;
        let fs = &mut state.fs;

        rs.camera_begin(self.client.camera);
        rs.draw_sprite("planet00", Vec2::new(0.0, 0.0));
        let mouse_pos_world =
            rs.screen_pos_to_world_pos(&self.client.camera, fs.mouse_pos);

        if fs.is_mouse_just_pressed(0) {
            self.start_mouse_pos = mouse_pos_world;
            self.is_dragging = false;
        }

        if fs.is_mouse_pressed(0) {
            self.end_mouse_pos = mouse_pos_world;
            if (self.start_mouse_pos - self.end_mouse_pos).magnitude() > 0.1 {
                self.is_dragging = true;
            }
        }

        if fs.is_mouse_just_released(0) {
            if self.is_dragging {
                let min_x = self.start_mouse_pos.x.min(self.end_mouse_pos.x);
                let max_x = self.start_mouse_pos.x.max(self.end_mouse_pos.x);
                let min_y = self.start_mouse_pos.y.min(self.end_mouse_pos.y);
                let max_y = self.start_mouse_pos.y.max(self.end_mouse_pos.y);

                let min = Vec2::new(min_x, min_y);
                let max = Vec2::new(max_x, max_y);

                let mut new_selected_worlds = self.get_worlds_under_box(min, max);

                new_selected_worlds.retain(|world_index| {
                    self.worlds[*world_index as usize].team == self.client.team
                });

                if fs.is_key_pressed(KeyCode::LeftShift) {
                    self.selected_worlds.append(&mut new_selected_worlds);
                } else {
                    self.selected_worlds = new_selected_worlds.clone();
                }

                self.is_dragging = false;
            } else {
                match self.get_world_under_point(mouse_pos_world) {
                    Some(world_index) => {
                        let world = &mut self.worlds[world_index as usize];
                        if world.team == self.client.team {
                            if fs.is_key_pressed(KeyCode::LeftShift) {
                                self.selected_worlds.push(world_index);
                            } else {
                                self.selected_worlds.clear();
                                self.selected_worlds.push(world_index);
                            }
                        } else {
                            let attack = PlayerActionAttack {
                                sources: self.selected_worlds.clone(),
                                target: world_index,
                            };

                            self.client.next_action = PlayerAction::Attack(attack);
                        }
                    }
                    None => {
                        self.selected_worlds.clear();
                    }
                }
            }
        }

        for (world_index, world) in self.worlds.iter_mut().enumerate() {
            let world_pos = Vec2::new(world.pos.x as f32, world.pos.y as f32);

            if self.selected_worlds.contains(&(world_index as i32)) {
                rs.draw_circle(world.size.size() as f32 + 1.0, world_pos)
                    .with_color(PURE_WHITE);
            }

            rs.draw_circle(world.size.size() as f32, world_pos)
                .with_color(world.team.color());
            rs.draw_text(world.ship_count.to_string().as_str(), world_pos)
                .with_color(PURE_WHITE)
                .with_horizontal_alignment(TextHAlignment::Center)
                .with_vertical_alignment(TextVAlignment::Center);
        }

        for squadron in &mut self.squadrons {
            let t = (squadron.travel_in_ticks as f32) / (squadron.distance_in_ticks as f32);
            let source_world_pos = ivec_to_vec(self.worlds[squadron.source_world as usize].pos);
            let target_world_pos = ivec_to_vec(self.worlds[squadron.dest_world as usize].pos);
            let pos = source_world_pos + (target_world_pos - source_world_pos) * t;
            squadron.world_pos = squadron.world_pos.lerp(pos, 0.40);
            rs.draw_circle(5.0, squadron.world_pos)
                .with_color(PURE_WHITE);
            //rs.draw_circle(5.0, pos)
            //    .with_color(Vec4::new(1.0, 0.0, 0.0, 0.5));
        }

        if fs.is_mouse_pressed(0) {
            let min_x = self.start_mouse_pos.x.min(self.end_mouse_pos.x);
            let max_x = self.start_mouse_pos.x.max(self.end_mouse_pos.x);
            let min_y = self.start_mouse_pos.y.min(self.end_mouse_pos.y);
            let max_y = self.start_mouse_pos.y.max(self.end_mouse_pos.y);

            let min = Vec2::new(min_x, min_y);
            let max = Vec2::new(max_x, max_y);

            rs.draw_rect_min_max(min, max)
                .with_color(Vec4::new(1.0, 1.0, 1.0, 0.5));
        }

        rs.camera_end();

        self.ui.update_and_render(state);
    }

    pub fn load(&mut self, map_file: &MapFile) {}
}
