use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChampionBuildProfile {
    archetype: String,
    role: String,
    skill_priority: String,
    tip: String,
}

#[derive(Clone, Debug)]
pub struct BuildRecommendation {
    pub archetype: String,
    pub role: String,
    pub skill_priority: String,
    pub skill_order: Vec<String>,
    pub tip: String,
    pub starting_items: Vec<u32>,
    pub boots: u32,
    pub core_items: Vec<u32>,
    pub situational_items: Vec<u32>,
    pub primary_tree: u32,
    pub secondary_tree: u32,
    pub rune_ids: Vec<u32>,
    pub spell_one: i64,
    pub spell_two: i64,
}

static CHAMPION_PROFILES: LazyLock<HashMap<String, ChampionBuildProfile>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../assets/champion_build_profiles.json")).unwrap_or_default()
});

static ITEM_NAMES: LazyLock<HashMap<u32, String>> = LazyLock::new(|| {
    let root: Value =
        serde_json::from_str(include_str!("../assets/data/item.json")).unwrap_or(Value::Null);
    root.get("data")
        .and_then(Value::as_object)
        .map(|items| {
            items
                .iter()
                .filter_map(|(id, item)| {
                    let id = id.parse().ok()?;
                    let name = item.get("name")?.as_str()?.trim();
                    (!name.is_empty()).then(|| (id, name.to_owned()))
                })
                .collect()
        })
        .unwrap_or_default()
});

pub fn item_name(id: u32) -> String {
    ITEM_NAMES
        .get(&id)
        .cloned()
        .unwrap_or_else(|| format!("Item {id}"))
}

pub fn item_available(id: u32) -> bool {
    ITEM_NAMES.contains_key(&id)
}

pub fn recommend(
    key: &str,
    name: &str,
    roles: &[String],
    requested_role: Option<&str>,
) -> BuildRecommendation {
    let normalized_key = normalize(key);
    let normalized_name = normalize(name);
    let profile = CHAMPION_PROFILES
        .iter()
        .find(|(candidate, _)| {
            let candidate = normalize(candidate);
            candidate == normalized_key || candidate == normalized_name
        })
        .map(|(_, profile)| profile);
    let fallback_role = roles
        .iter()
        .find(|role| valid_role(role))
        .map(|role| role.to_ascii_uppercase())
        .unwrap_or_else(|| "MID".to_string());
    let role = requested_role
        .filter(|role| valid_role(role))
        .map(str::to_ascii_uppercase)
        .unwrap_or_else(|| {
            profile
                .map(|profile| profile.role.to_ascii_uppercase())
                .unwrap_or(fallback_role)
        });
    let archetype = profile
        .map(|profile| profile.archetype.as_str())
        .unwrap_or("DIVER_FIGHTER");
    let skill_priority = profile
        .map(|profile| profile.skill_priority.as_str())
        .unwrap_or("Q > E > W")
        .to_owned();
    let tip = profile
        .map(|profile| profile.tip.as_str())
        .unwrap_or("Focus on safe trades and your champion's core power spikes.")
        .to_owned();

    let (
        starting_items,
        boots,
        core_items,
        situational_items,
        primary_tree,
        secondary_tree,
        _keystone,
        rune_ids,
        spell_one,
        spell_two,
    ) = preset(archetype, &role);
    BuildRecommendation {
        archetype: archetype.to_owned(),
        role,
        skill_order: skill_order(&skill_priority),
        skill_priority,
        tip,
        starting_items: available(starting_items),
        boots,
        core_items: available(core_items),
        situational_items: available(situational_items),
        primary_tree,
        secondary_tree,
        rune_ids,
        spell_one,
        spell_two,
    }
}

pub fn rune_page(champion: &str, build: &BuildRecommendation) -> Value {
    serde_json::json!({
        "name": format!("RiftCore {champion} {}", build.role),
        "primaryStyleId": build.primary_tree,
        "subStyleId": build.secondary_tree,
        "selectedPerkIds": build.rune_ids,
        "current": true
    })
}

fn normalize(input: &str) -> String {
    input
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn valid_role(role: &str) -> bool {
    matches!(
        role.to_ascii_uppercase().as_str(),
        "TOP" | "JUNGLE" | "MID" | "ADC" | "SUPPORT"
    )
}

fn available(items: Vec<u32>) -> Vec<u32> {
    items.into_iter().filter(|id| item_available(*id)).collect()
}

fn skill_order(priority: &str) -> Vec<String> {
    let skills: Vec<&str> = priority.split('>').map(str::trim).collect();
    let (first, second, third) = match skills.as_slice() {
        [first, second, third, ..] => (*first, *second, *third),
        _ => ("Q", "E", "W"),
    };
    let mut order = vec![first, second, third];
    let (mut first_count, mut second_count) = (1, 1);
    for level in 4..=18 {
        if [6, 11, 16].contains(&level) {
            order.push("R");
        } else if first_count < 5 {
            order.push(first);
            first_count += 1;
        } else if second_count < 5 {
            order.push(second);
            second_count += 1;
        } else {
            order.push(third);
        }
    }
    order.into_iter().map(str::to_owned).collect()
}

fn preset(
    archetype: &str,
    role: &str,
) -> (
    Vec<u32>,
    u32,
    Vec<u32>,
    Vec<u32>,
    u32,
    u32,
    u32,
    Vec<u32>,
    i64,
    i64,
) {
    let (
        start,
        boots,
        core,
        situational,
        primary,
        secondary,
        keystone,
        perks,
        spell_one,
        spell_two,
    ) = match archetype {
        "BURST_MAGE" => (
            vec![1056, 2003],
            3020,
            vec![3003, 4645, 3089],
            vec![3157, 3135, 3102],
            8100,
            8200,
            8112,
            vec![8112, 8139, 8140, 8106, 8226, 8210, 5008, 5008, 5001],
            4,
            if role == "MID" { 14 } else { 12 },
        ),
        "BATTLE_MAGE" => (
            vec![1056, 2003],
            3020,
            vec![6657, 6653, 4633],
            vec![3157, 3137, 3102],
            8200,
            8300,
            8230,
            vec![8230, 8226, 8210, 8236, 8345, 8347, 5008, 5008, 5001],
            4,
            12,
        ),
        "AP_ASSASSIN" => (
            vec![1056, 2003],
            3020,
            vec![3100, 4646, 4645],
            vec![3157, 3089, 3135],
            8100,
            8000,
            8112,
            vec![8112, 8143, 8140, 8105, 9111, 8014, 5008, 5008, 5001],
            4,
            14,
        ),
        "AD_ASSASSIN" => (
            vec![1055, 2003],
            3158,
            vec![6698, 6697, 3036],
            vec![3814, 3142, 3026],
            8100,
            8200,
            8112,
            vec![8112, 8143, 8140, 8106, 8210, 8237, 5008, 5008, 5001],
            4,
            if role == "JUNGLE" { 11 } else { 14 },
        ),
        "CRIT_MARKSMAN" => (
            vec![1055, 2003],
            3006,
            vec![6672, 3031, 3036],
            vec![3094, 3072, 6673],
            8000,
            8200,
            8008,
            vec![8008, 9101, 9104, 8014, 8210, 8236, 5005, 5008, 5001],
            4,
            7,
        ),
        "ONHIT_MARKSMAN" => (
            vec![1055, 2003],
            3006,
            vec![3153, 3124, 3302],
            vec![3091, 6665, 6673],
            8000,
            8400,
            8005,
            vec![8005, 9111, 9104, 8299, 8429, 8451, 5005, 5008, 5001],
            4,
            6,
        ),
        "JUGGERNAUT" => (
            vec![1055, 2003],
            3047,
            vec![6631, 3071, 3053],
            vec![3742, 4401, 3075],
            8000,
            8400,
            8010,
            vec![8010, 9111, 9105, 8299, 8429, 8451, 5008, 5008, 5001],
            4,
            6,
        ),
        "SKIRMISHER" => (
            vec![1055, 2003],
            3006,
            vec![3153, 3031, 6673],
            vec![6333, 3156, 3026],
            8000,
            8400,
            8008,
            vec![8008, 9111, 9104, 8299, 8429, 8451, 5005, 5008, 5001],
            4,
            14,
        ),
        "TANK" => (
            vec![1054, 2003],
            3047,
            vec![3084, 3068, 3075],
            vec![2504, 6664, 2502],
            8400,
            8000,
            8437,
            vec![8437, 8446, 8429, 8451, 9111, 9105, 5008, 5001, 5011],
            4,
            if role == "JUNGLE" { 11 } else { 12 },
        ),
        "ENCHANTER" => (
            vec![3865, 2003],
            3158,
            vec![3869, 6617, 6616],
            vec![3504, 3107, 3222],
            8200,
            8400,
            8214,
            vec![8214, 8226, 8210, 8237, 8463, 8453, 5008, 5008, 5001],
            4,
            14,
        ),
        "ENGAGE_SUPPORT" => (
            vec![3865, 2003],
            3009,
            vec![3870, 2503, 3190],
            vec![3109, 3075, 3050],
            8400,
            8300,
            8439,
            vec![8439, 8463, 8473, 8451, 8306, 8347, 5007, 5001, 5011],
            4,
            14,
        ),
        "AP_SUPPORT" => (
            vec![3865, 2003],
            3020,
            vec![6653, 3116, 3157],
            vec![3165, 3137, 3102],
            8200,
            8100,
            8229,
            vec![8229, 8226, 8210, 8237, 8140, 8106, 5008, 5008, 5001],
            4,
            14,
        ),
        _ => (
            if role == "JUNGLE" {
                vec![1102, 2031]
            } else {
                vec![1055, 2003]
            },
            3047,
            vec![6694, 6692, 3071],
            vec![3053, 3156, 3026],
            8000,
            8400,
            8010,
            vec![8010, 9111, 9104, 8299, 8473, 8451, 5008, 5008, 5001],
            4,
            if role == "JUNGLE" { 11 } else { 12 },
        ),
    };
    (
        start,
        boots,
        core,
        situational,
        primary,
        secondary,
        keystone,
        perks,
        spell_one,
        spell_two,
    )
}
