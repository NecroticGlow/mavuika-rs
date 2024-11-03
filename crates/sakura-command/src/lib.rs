use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use sakura_data::excel::monster_excel_config_collection;
use sakura_entity::{
    common::{GrowCurveConfigType, Level, Visible, LifeState}, // 移除 EntityCounter 的导入
    monster::{MonsterBundle, MonsterID},
    transform::{Transform, Vector3},
    util::to_protocol_entity_id,
    ProtEntityType,
};
use sakura_persistence::Players;
use sakura_scene::ScenePlayerJumpEvent;
use rand::RngCore;
use std::fs; // 引入 std::fs
use tracing::{debug, instrument};
use util::create_fight_properties_by_monster_config;
use bevy_ecs::system::Resource; // 添加 Resource

mod util;

#[derive(Resource)] // 确保 LuaShellSettings 是一个资源
pub struct LuaShellSettings {
    pub startup_payloads: Vec<Box<[u8]>>, // 定义 LuaShellSettings 结构体
}

pub struct CommandPlugin;

impl Plugin for CommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DebugCommandEvent>()
            .add_systems(Update, debug_command_handler);
    }
}

#[derive(Event)]
pub struct DebugCommandEvent {
    pub executor_uid: u32,
    pub kind: CommandKind,
}

#[derive(Debug)]
pub enum CommandKind {
    SpawnMonster {
        monster_id: Option<String>, // 改为 Option<String> 以支持字母输入
        position: (f32, f32),
    },
    QuickTravel {
        position: (f32, Option<f32>, f32),
    },
}

#[instrument(skip_all)]
pub fn debug_command_handler(
    mut events: EventReader<DebugCommandEvent>,
    mut commands: Commands,
    mut entity_counter: ResMut<EntityCounter>,
    players: Res<Players>,
    mut jump_events: EventWriter<ScenePlayerJumpEvent>,
    mut lua_shell_settings: ResMut<LuaShellSettings>, // 直接使用资源
) {
    for command in events.read() {
        debug!(
            "executor_uid: {}, kind: {:?}",
            command.executor_uid, command.kind
        );

        let player = players.get(command.executor_uid);

        match &command.kind { // 使用 &command.kind 来借用而不是移动
            CommandKind::SpawnMonster {
                monster_id,
                position,
            } => {
                // 判断输入的内容
                if let Some(input) = monster_id {
                    // 检查输入是否为数字
                    if input.chars().all(char::is_numeric) {
                        // 处理数字输入以生成怪物
                        let monster_id = input.parse::<u32>().unwrap_or_else(|_| {
                            [20010101, 20010302, 20010502, 20010803, 20011002]
                                [rand::thread_rng().next_u32() as usize % 5]
                        });

                        let Some(config) =
                            monster_excel_config_collection::iter().find(|cfg| cfg.id == monster_id)
                        else {
                            debug!("monster config for id {monster_id} not found");
                            continue;
                        };

                        let level = 90;
                        let mut fight_properties = create_fight_properties_by_monster_config(config);
                        for grow_curve in config.prop_grow_curves.iter() {
                            fight_properties.apply_grow_curve(level, grow_curve, GrowCurveConfigType::Monster);
                        }
                        fight_properties.apply_base_values();

                        commands.spawn(MonsterBundle {
                            monster_id: MonsterID(monster_id),
                            entity_id: to_protocol_entity_id(ProtEntityType::Monster, entity_counter.next()),
                            level: Level(level),
                            transform: Transform {
                                position: (
                                    position.0,
                                    player.world_position.position.1 + 10.0,
                                    position.1,
                                ).into(),
                                rotation: Vector3::default(),
                            },
                            fight_properties,
                            life_state: LifeState::default(), // 确保 LifeState 有 default 方法
                        }).insert(Visible);
                    } else {
                        // 处理字母输入以读取文件
                        let filename = format!("assets/luashell/{}.bin", input);
                        let payload = fs::read(&filename)
                            .unwrap_or_else(|_| {
                                debug!("Failed to read file: {}", filename);
                                Vec::new()
                            });

                        lua_shell_settings.startup_payloads = vec![payload.into_boxed_slice()]; // 更新资源
                    }
                } else {
                    // 处理没有输入的情况
                    debug!("No monster ID or input provided");
                }
            }
            CommandKind::QuickTravel { position } => {
                let destination =
                    Vector3::from((position.0, position.1.unwrap_or(2000.0), position.2));
                jump_events.send(ScenePlayerJumpEvent(command.executor_uid, destination));
            }
        }
    }
}

// 确保 EntityCounter 结构体及其实现
#[derive(Resource)] // 确保 EntityCounter 是一个资源
pub struct EntityCounter {
    current_id: u32,
}

impl EntityCounter {
    pub fn new() -> Self {
        Self { current_id: 0 }
    }

    pub fn next(&mut self) -> u32 {
        self.current_id += 1; // 递增当前ID
        self.current_id
    }
}
