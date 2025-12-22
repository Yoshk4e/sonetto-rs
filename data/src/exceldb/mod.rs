// Auto-generated module declarations

pub mod antique;
pub mod battle;
pub mod bgm_switch;
pub mod bonus;
pub mod chapter;
pub mod character;
pub mod character_destiny;
pub mod character_level;
pub mod character_rank;
pub mod character_talent;
pub mod character_voice;
pub mod cloth_level;
pub mod currency;
pub mod episode;
pub mod equip;
pub mod equip_skill;
pub mod equip_strengthen;
pub mod guide;
pub mod insight_item;
pub mod item;
pub mod monster;
pub mod monster_skill_template;
pub mod monster_template;
pub mod open;
pub mod power_item;
pub mod skill;
pub mod skill_ex_level;
pub mod skill_passive_level;
pub mod skin;
pub mod summon;
pub mod summon_pool;
pub mod talent_scheme;

use std::sync::OnceLock;
use anyhow::Result;

pub struct GameDB {
    pub antique: antique::AntiqueTable,
    pub battle: battle::BattleTable,
    pub bgm_switch: bgm_switch::BgmSwitchTable,
    pub bonus: bonus::BonusTable,
    pub chapter: chapter::ChapterTable,
    pub character: character::CharacterTable,
    pub character_destiny: character_destiny::CharacterDestinyTable,
    pub character_level: character_level::CharacterLevelTable,
    pub character_rank: character_rank::CharacterRankTable,
    pub character_talent: character_talent::CharacterTalentTable,
    pub character_voice: character_voice::CharacterVoiceTable,
    pub cloth_level: cloth_level::ClothLevelTable,
    pub currency: currency::CurrencyTable,
    pub episode: episode::EpisodeTable,
    pub equip: equip::EquipTable,
    pub equip_skill: equip_skill::EquipSkillTable,
    pub equip_strengthen: equip_strengthen::EquipStrengthenTable,
    pub guide: guide::GuideTable,
    pub insight_item: insight_item::InsightItemTable,
    pub item: item::ItemTable,
    pub monster: monster::MonsterTable,
    pub monster_skill_template: monster_skill_template::MonsterSkillTemplateTable,
    pub monster_template: monster_template::MonsterTemplateTable,
    pub open: open::OpenTable,
    pub power_item: power_item::PowerItemTable,
    pub skill: skill::SkillTable,
    pub skill_ex_level: skill_ex_level::SkillExLevelTable,
    pub skill_passive_level: skill_passive_level::SkillPassiveLevelTable,
    pub skin: skin::SkinTable,
    pub summon: summon::SummonTable,
    pub summon_pool: summon_pool::SummonPoolTable,
    pub talent_scheme: talent_scheme::TalentSchemeTable,
}

impl GameDB {
    pub fn load(data_dir: &str) -> Result<Self> {
        let antique = antique::AntiqueTable::load(
            &format!("{data_dir}/antique.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load antique.json: {}", e))?;
        let battle = battle::BattleTable::load(
            &format!("{data_dir}/battle.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load battle.json: {}", e))?;
        let bgm_switch = bgm_switch::BgmSwitchTable::load(
            &format!("{data_dir}/bgm_switch.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load bgm_switch.json: {}", e))?;
        let bonus = bonus::BonusTable::load(
            &format!("{data_dir}/bonus.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load bonus.json: {}", e))?;
        let chapter = chapter::ChapterTable::load(
            &format!("{data_dir}/chapter.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load chapter.json: {}", e))?;
        let character = character::CharacterTable::load(
            &format!("{data_dir}/character.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load character.json: {}", e))?;
        let character_destiny = character_destiny::CharacterDestinyTable::load(
            &format!("{data_dir}/character_destiny.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load character_destiny.json: {}", e))?;
        let character_level = character_level::CharacterLevelTable::load(
            &format!("{data_dir}/character_level.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load character_level.json: {}", e))?;
        let character_rank = character_rank::CharacterRankTable::load(
            &format!("{data_dir}/character_rank.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load character_rank.json: {}", e))?;
        let character_talent = character_talent::CharacterTalentTable::load(
            &format!("{data_dir}/character_talent.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load character_talent.json: {}", e))?;
        let character_voice = character_voice::CharacterVoiceTable::load(
            &format!("{data_dir}/character_voice.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load character_voice.json: {}", e))?;
        let cloth_level = cloth_level::ClothLevelTable::load(
            &format!("{data_dir}/cloth_level.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load cloth_level.json: {}", e))?;
        let currency = currency::CurrencyTable::load(
            &format!("{data_dir}/currency.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load currency.json: {}", e))?;
        let episode = episode::EpisodeTable::load(
            &format!("{data_dir}/episode.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load episode.json: {}", e))?;
        let equip = equip::EquipTable::load(
            &format!("{data_dir}/equip.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load equip.json: {}", e))?;
        let equip_skill = equip_skill::EquipSkillTable::load(
            &format!("{data_dir}/equip_skill.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load equip_skill.json: {}", e))?;
        let equip_strengthen = equip_strengthen::EquipStrengthenTable::load(
            &format!("{data_dir}/equip_strengthen.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load equip_strengthen.json: {}", e))?;
        let guide = guide::GuideTable::load(
            &format!("{data_dir}/guide.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load guide.json: {}", e))?;
        let insight_item = insight_item::InsightItemTable::load(
            &format!("{data_dir}/insight_item.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load insight_item.json: {}", e))?;
        let item = item::ItemTable::load(
            &format!("{data_dir}/item.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load item.json: {}", e))?;
        let monster = monster::MonsterTable::load(
            &format!("{data_dir}/monster.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load monster.json: {}", e))?;
        let monster_skill_template = monster_skill_template::MonsterSkillTemplateTable::load(
            &format!("{data_dir}/monster_skill_template.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load monster_skill_template.json: {}", e))?;
        let monster_template = monster_template::MonsterTemplateTable::load(
            &format!("{data_dir}/monster_template.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load monster_template.json: {}", e))?;
        let open = open::OpenTable::load(
            &format!("{data_dir}/open.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load open.json: {}", e))?;
        let power_item = power_item::PowerItemTable::load(
            &format!("{data_dir}/power_item.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load power_item.json: {}", e))?;
        let skill = skill::SkillTable::load(
            &format!("{data_dir}/skill.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load skill.json: {}", e))?;
        let skill_ex_level = skill_ex_level::SkillExLevelTable::load(
            &format!("{data_dir}/skill_ex_level.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load skill_ex_level.json: {}", e))?;
        let skill_passive_level = skill_passive_level::SkillPassiveLevelTable::load(
            &format!("{data_dir}/skill_passive_level.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load skill_passive_level.json: {}", e))?;
        let skin = skin::SkinTable::load(
            &format!("{data_dir}/skin.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load skin.json: {}", e))?;
        let summon = summon::SummonTable::load(
            &format!("{data_dir}/summon.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load summon.json: {}", e))?;
        let summon_pool = summon_pool::SummonPoolTable::load(
            &format!("{data_dir}/summon_pool.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load summon_pool.json: {}", e))?;
        let talent_scheme = talent_scheme::TalentSchemeTable::load(
            &format!("{data_dir}/talent_scheme.json")
        ).map_err(|e| anyhow::anyhow!("Failed to load talent_scheme.json: {}", e))?;

        Ok(Self {
            antique,
            battle,
            bgm_switch,
            bonus,
            chapter,
            character,
            character_destiny,
            character_level,
            character_rank,
            character_talent,
            character_voice,
            cloth_level,
            currency,
            episode,
            equip,
            equip_skill,
            equip_strengthen,
            guide,
            insight_item,
            item,
            monster,
            monster_skill_template,
            monster_template,
            open,
            power_item,
            skill,
            skill_ex_level,
            skill_passive_level,
            skin,
            summon,
            summon_pool,
            talent_scheme,
        })
    }

    pub fn global() -> &'static GameDB {
        static DB: OnceLock<GameDB> = OnceLock::new();
        DB.get_or_init(|| {
            Self::load("data").expect("Failed to load game database")
        })
    }
}

static GAME_DATA: OnceLock<GameDB> = OnceLock::new();

pub fn init(data_dir: &str) -> Result<()> {
    let db = GameDB::load(data_dir)?;
    GAME_DATA.set(db)
        .map_err(|_| anyhow::anyhow!("Game data already initialized"))
}

#[inline]
pub fn get() -> &'static GameDB {
    GAME_DATA.get().expect("Game data not initialized. Call init() first.")
}

#[inline]
pub fn try_get() -> Option<&'static GameDB> {
    GAME_DATA.get()
}