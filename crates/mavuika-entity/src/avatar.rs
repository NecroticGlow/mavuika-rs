use std::collections::HashMap;

use bevy_ecs::{prelude::*, query::QueryData};
use mavuika_message::output::MessageOutput;
use mavuika_persistence::{
    player_information::{AvatarInformation, ItemInformation},
    Players,
};
use mavuika_proto::{AvatarChangeCostumeNotify, AvatarChangeTraceEffectNotify, SceneEntityInfo};

use crate::{
    int_prop_pair,
    transform::Transform,
    weapon::{WeaponQueryReadOnly, WeaponQueryReadOnlyItem},
};

use super::{ability::Ability, common::*};

#[derive(Component)]
pub struct Equipment {
    pub weapon: Entity,
}

#[derive(Component)]
pub struct AvatarAppearance {
    pub flycloak_id: u32,
    pub costume_id: u32,
    pub trace_effect_id: u32,
}

#[derive(Event)]
pub struct AvatarEquipChangeEvent {
    pub player_uid: u32,
    pub avatar_guid: u64,
    pub weapon_guid: u64,
}

pub enum AvatarAppearanceChange {
    Costume(u32),
    TraceEffect(u32),
}

#[derive(Event)]
pub struct AvatarAppearanceChangeEvent {
    pub player_uid: u32,
    pub avatar_guid: u64,
    pub change: AvatarAppearanceChange,
}

#[derive(Component)]
pub struct AvatarID(pub u32);

#[derive(Component)]
pub struct ControlPeer(pub u32);

#[derive(Component)]
pub struct SkillDepot(pub u32);

#[derive(Component)]
pub struct BornTime(pub u32);

#[derive(Component, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexInSceneTeam(pub u8);

#[derive(Component)]
pub struct CurrentPlayerAvatarMarker;

#[derive(Component)]
pub struct SkillLevelMap(pub HashMap<u32, u32>);

#[derive(Component)]
pub struct InherentProudSkillList(pub Vec<u32>);

#[derive(Bundle)]
pub struct AvatarBundle {
    pub avatar_id: AvatarID,
    pub entity_id: ProtocolEntityID,
    pub guid: Guid,
    pub level: Level,
    pub break_level: BreakLevel,
    pub control_peer: ControlPeer,
    pub skill_depot: SkillDepot,
    pub equipment: Equipment,
    pub appearance: AvatarAppearance,
    pub transform: Transform,
    pub owner_player_uid: OwnerPlayerUID,
    pub fight_properties: FightProperties,
    pub life_state: LifeState,
    pub ability: Ability,
    pub born_time: BornTime,
    pub index_in_scene_team: IndexInSceneTeam,
    pub skill_level_map: SkillLevelMap,
    pub inherent_proud_skill_list: InherentProudSkillList,
}

#[derive(QueryData)]
pub struct AvatarQueryReadOnly {
    pub avatar_id: &'static AvatarID,
    pub entity_id: &'static ProtocolEntityID,
    pub guid: &'static Guid,
    pub level: &'static Level,
    pub break_level: &'static BreakLevel,
    pub control_peer: &'static ControlPeer,
    pub skill_depot: &'static SkillDepot,
    pub equipment: &'static Equipment,
    pub appearance: &'static AvatarAppearance,
    pub transform: &'static Transform,
    pub owner_player_uid: &'static OwnerPlayerUID,
    pub fight_properties: &'static FightProperties,
    pub life_state: &'static LifeState,
    pub ability: &'static Ability,
    pub born_time: &'static BornTime,
    pub index_in_scene_team: &'static IndexInSceneTeam,
    pub skill_level_map: &'static SkillLevelMap,
    pub inherent_proud_skill_list: &'static InherentProudSkillList,
}

pub fn update_avatar_appearance(
    mut events: EventReader<AvatarAppearanceChangeEvent>,
    mut avatars: Query<(&Guid, &mut AvatarAppearance)>,
) {
    for event in events.read() {
        if let Some((_, mut appearance)) =
            avatars.iter_mut().find(|(g, _)| g.0 == event.avatar_guid)
        {
            match event.change {
                AvatarAppearanceChange::Costume(costume_id) => {
                    appearance.costume_id = costume_id;
                }
                AvatarAppearanceChange::TraceEffect(trace_effect_id) => {
                    appearance.trace_effect_id = trace_effect_id;
                }
            }
        }
    }
}

pub fn notify_avatar_appearance_change(
    mut events: EventReader<AvatarAppearanceChangeEvent>,
    avatars: Query<AvatarQueryReadOnly>,
    weapons: Query<WeaponQueryReadOnly>,
    message_output: Res<MessageOutput>,
    players: Res<Players>,
) {
    for event in events.read() {
        if let Some(avatar_data) = avatars
            .iter()
            .find(|avatar_data| avatar_data.guid.0 == event.avatar_guid)
        {
            let weapon_data = weapons.get(avatar_data.equipment.weapon).unwrap();
            let entity_info = Some(build_avatar_entity_info(&avatar_data, &weapon_data));

            match event.change {
                AvatarAppearanceChange::Costume(_) => {
                    message_output.send_to_all(AvatarChangeCostumeNotify { entity_info })
                }
                AvatarAppearanceChange::TraceEffect(_) => {
                    message_output.send_to_all(AvatarChangeTraceEffectNotify { entity_info })
                }
            }
        }
        // that's disgusting, notify required even if avatar is not on scene
        // even though it contains SceneEntityInfo
        else {
            let player = players.get(event.player_uid);

            let avatar = player
                .avatar_module
                .avatar_map
                .get(&event.avatar_guid)
                .unwrap();
            let weapon = player.item_map.get(&avatar.weapon_guid).unwrap();

            let entity_info = Some(build_fake_avatar_entity_info(avatar, weapon));
            match event.change {
                AvatarAppearanceChange::Costume(_) => {
                    message_output.send(event.player_uid, AvatarChangeCostumeNotify { entity_info })
                }
                AvatarAppearanceChange::TraceEffect(_) => message_output.send(
                    event.player_uid,
                    AvatarChangeTraceEffectNotify { entity_info },
                ),
            }
        }
    }
}

pub fn notify_appear_avatar_entities(
    appear_avatars: Query<AvatarQueryReadOnly, (Added<Visible>, Without<ToBeRemovedMarker>)>,
    weapons: Query<WeaponQueryReadOnly>,
    message_output: Res<MessageOutput>,
) {
    use mavuika_proto::*;

    message_output.send_to_all(SceneEntityAppearNotify {
        appear_type: VisionType::Meet.into(),
        param: 0,
        entity_list: appear_avatars
            .iter()
            .map(|avatar_data| {
                let weapon_data = weapons.get(avatar_data.equipment.weapon).unwrap();
                build_avatar_entity_info(&avatar_data, &weapon_data)
            })
            .collect(),
    });
}

pub fn run_if_avatar_entities_appeared(
    appear_avatars: Query<AvatarQueryReadOnly, (Added<Visible>, Without<ToBeRemovedMarker>)>,
) -> bool {
    !appear_avatars.is_empty()
}

fn build_fake_avatar_entity_info(
    avatar: &AvatarInformation,
    weapon: &ItemInformation,
) -> SceneEntityInfo {
    use mavuika_proto::*;

    let ItemInformation::Weapon {
        weapon_id,
        level,
        promote_level,
        affix_map,
        ..
    } = weapon;

    SceneEntityInfo {
        entity_type: ProtEntityType::Avatar.into(),
        entity_id: 0,
        entity: Some(scene_entity_info::Entity::Avatar(SceneAvatarInfo {
            uid: (avatar.guid >> 32) as u32,
            avatar_id: avatar.avatar_id,
            guid: avatar.guid,
            equip_id_list: vec![*weapon_id],
            skill_depot_id: avatar.skill_depot_id,
            talent_id_list: vec![],
            weapon: Some(SceneWeaponInfo {
                guid: avatar.weapon_guid,
                item_id: *weapon_id,
                level: *level,
                promote_level: *promote_level,
                affix_map: affix_map.clone(),
                ..Default::default()
            }),
            reliquary_list: Vec::with_capacity(0),
            core_proud_skill_level: 0,
            inherent_proud_skill_list: avatar.inherent_proud_skill_list.clone(),
            skill_level_map: avatar.skill_level_map.clone(),
            proud_skill_extra_level_map: HashMap::with_capacity(0),
            server_buff_list: Vec::with_capacity(0),
            team_resonance_list: Vec::with_capacity(0),
            wearing_flycloak_id: avatar.wearing_flycloak_id,
            born_time: avatar.born_time,
            costume_id: avatar.costume_id,
            trace_effect_id: avatar.trace_effect_id,
            cur_vehicle_info: None,
            excel_info: Some(AvatarExcelInfo::default()),
            anim_hash: 0,
            ..Default::default()
        })),
        ..Default::default()
    }
}

fn build_avatar_entity_info(
    avatar_data: &AvatarQueryReadOnlyItem,
    weapon_data: &WeaponQueryReadOnlyItem,
) -> SceneEntityInfo {
    use mavuika_proto::*;

    SceneEntityInfo {
        entity_type: ProtEntityType::Avatar.into(),
        entity_id: avatar_data.entity_id.0,
        name: String::new(),
        motion_info: Some(MotionInfo {
            pos: Some(avatar_data.transform.position.into()),
            rot: Some(avatar_data.transform.rotation.into()),
            speed: Some(Vector::default()),
            ..Default::default()
        }),
        prop_list: vec![
            int_prop_pair!(PROP_LEVEL, avatar_data.level.0),
            int_prop_pair!(PROP_BREAK_LEVEL, avatar_data.break_level.0),
        ],
        fight_prop_list: avatar_data
            .fight_properties
            .0
            .iter()
            .map(|(k, v)| FightPropPair {
                prop_type: *k as u32,
                prop_value: *v,
            })
            .collect(),
        life_state: *avatar_data.life_state as u32,
        animator_para_list: vec![AnimatorParameterValueInfoPair {
            name_id: 0,
            animator_para: Some(AnimatorParameterValueInfo::default()),
        }],
        last_move_scene_time_ms: 0,
        last_move_reliable_seq: 0,
        entity_client_data: Some(EntityClientData::default()),
        entity_environment_info_list: Vec::with_capacity(0),
        entity_authority_info: Some(EntityAuthorityInfo {
            ability_info: Some(AbilitySyncStateInfo::default()),
            born_pos: Some(Vector::default()),
            client_extra_info: Some(EntityClientExtraInfo {
                skill_anchor_position: Some(Vector::default()),
            }),
            ..Default::default()
        }),
        tag_list: Vec::with_capacity(0),
        server_buff_list: Vec::with_capacity(0),
        entity: Some(scene_entity_info::Entity::Avatar(SceneAvatarInfo {
            uid: avatar_data.owner_player_uid.0,
            avatar_id: avatar_data.avatar_id.0,
            guid: avatar_data.guid.0,
            peer_id: avatar_data.control_peer.0,
            equip_id_list: vec![weapon_data.weapon_id.0],
            skill_depot_id: avatar_data.skill_depot.0,
            talent_id_list: vec![],
            weapon: Some(SceneWeaponInfo {
                guid: weapon_data.guid.0,
                entity_id: weapon_data.entity_id.0,
                gadget_id: weapon_data.gadget_id.0,
                item_id: weapon_data.weapon_id.0,
                level: weapon_data.level.0,
                promote_level: weapon_data.promote_level.0,
                affix_map: weapon_data.affix_map.0.clone(),
                ability_info: Some(AbilitySyncStateInfo::default()),
                renderer_changed_info: Some(EntityRendererChangedInfo::default()),
            }),
            reliquary_list: Vec::with_capacity(0),
            core_proud_skill_level: 0,
            inherent_proud_skill_list: avatar_data.inherent_proud_skill_list.0.clone(),
            skill_level_map: avatar_data.skill_level_map.0.clone(),
            proud_skill_extra_level_map: HashMap::with_capacity(0),
            server_buff_list: Vec::with_capacity(0),
            team_resonance_list: Vec::with_capacity(0),
            wearing_flycloak_id: avatar_data.appearance.flycloak_id,
            born_time: avatar_data.born_time.0,
            costume_id: avatar_data.appearance.costume_id,
            trace_effect_id: avatar_data.appearance.trace_effect_id,
            cur_vehicle_info: None,
            excel_info: Some(AvatarExcelInfo::default()),
            anim_hash: 0,
        })),
    }
}
