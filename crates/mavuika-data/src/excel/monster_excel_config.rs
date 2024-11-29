use mavuika_data_derive::FromBinary;

use super::common::PropGrowCurve;

#[derive(Debug, FromBinary)]
pub struct HpDropConfig {
    pub drop_id: u32,
    pub hp_percent: f32,
}

#[derive(Debug, FromBinary)]
pub struct MonsterExcelConfig {
    pub unk_1: u32,
    pub monster_name: String,
    pub r#type: i32,
    pub unk_2: u32,
    pub security_level: i32,
    pub script_data_path_hash: u64,
    pub server_script: String,
    pub combat_config_hash: u64,
    pub affix: Vec<u32>,
    pub ai: String,
    pub is_ai_hash_check: bool,
    pub equips: Vec<u32>,
    pub can_swim: bool,
    pub hp_drops: Vec<HpDropConfig>,
    pub kill_drop_id: u32,
    pub is_scene_reward: bool,
    pub vision_level: i32,
    pub is_invisible_reset: bool,
    pub exclude_weathers: String,
    pub feature_tag_group_id: u32,
    pub mp_prop_id: u32,
    pub skin: String,
    pub describe_id: u32,
    pub safety_check: bool,
    pub combat_bgm_level: u32,
    pub entity_budget_level: u32,
    pub radar_hint_id: u32,
    pub hide_name_in_element_view: bool,
    pub unk_3: u32,
    pub unk_4: u32,
    pub hp_base: f32,
    pub attack_base: f32,
    pub defense_base: f32,
    pub critical: f32,
    pub anti_critical: f32,
    pub critical_hurt: f32,
    pub fire_sub_hurt: f32,
    pub grass_sub_hurt: f32,
    pub water_sub_hurt: f32,
    pub elec_sub_hurt: f32,
    pub wind_sub_hurt: f32,
    pub ice_sub_hurt: f32,
    pub rock_sub_hurt: f32,
    pub fire_add_hurt: f32,
    pub grass_add_hurt: f32,
    pub water_add_hurt: f32,
    pub elec_add_hurt: f32,
    pub wind_add_hurt: f32,
    pub ice_add_hurt: f32,
    pub rock_add_hurt: f32,
    pub prop_grow_curves: Vec<PropGrowCurve>,
    pub element_mastery: f32,
    pub physical_sub_hurt: f32,
    pub physical_add_hurt: f32,
    pub prefab_path_ragdoll_hash: u64,
    pub deformation_mesh_path_hash: u64,
    pub id: u32,
    pub name_text_map_hash: u32,
    pub prefab_path_hash: u64,
    pub prefab_path_remote_hash: u64,
    pub controller_path_hash: u64,
    pub controller_path_remote_hash: u64,
    pub camp_id: u32,
    pub lod_pattern_name: String,
}