use crate::error::AppError;
use crate::state::ConnectionContext;
use crate::utils::inventory::{add_currencies, add_items};
use crate::utils::push;
use database::db::{game, user};
use database::models::game::heros::UserHeroModel;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CommandContext {
    pub ctx: Arc<Mutex<ConnectionContext>>,
    pub user_id: i64,
    pub args: Vec<String>,
}

pub async fn execute_command(
    ctx: Arc<Mutex<ConnectionContext>>,
    input: &str,
) -> Result<String, AppError> {
    let input = input.trim();
    if !input.starts_with("/") {
        return Err(AppError::InvalidRequest);
    }

    let parts: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
    if parts.is_empty() {
        return Ok("Invalid command".to_string());
    }

    let cmd = &parts[0];
    let args = parts[1..].to_vec();

    let user_id = ctx.lock().await.player_id.ok_or(AppError::NotLoggedIn)?;

    let cmd_ctx = CommandContext {
        ctx: ctx.clone(),
        user_id,
        args,
    };

    match cmd.as_str() {
        "/help" => Ok(get_help()),
        "/item" => cmd_item(cmd_ctx).await,
        "/currency" => cmd_currency(cmd_ctx).await,
        "/level" => cmd_level(cmd_ctx).await,
        "/hero" => cmd_hero(cmd_ctx).await,
        "/equip" => cmd_equip(cmd_ctx).await,
        _ => Ok(format!("Unknown command: {}", cmd)),
    }
}

fn get_help() -> String {
    r#"Available GM Commands:
/help - Show this help
/item <id> <amount> - Add items
/currency <id> <amount> - Add currency
/level <level> - Set player level
/hero <id> - Add hero
/equip <id> <amount> - Add equipment"#
        .to_string()
}

async fn cmd_item(ctx: CommandContext) -> Result<String, AppError> {
    if ctx.args.len() < 2 {
        return Ok("Usage: /item <id> <amount>".to_string());
    }

    let item_id: u32 = match ctx.args[0].parse() {
        Ok(id) => id,
        Err(_) => return Ok(format!("Invalid item ID: {}", ctx.args[0])),
    };

    let amount: i32 = match ctx.args[1].parse() {
        Ok(amt) => amt,
        Err(_) => return Ok(format!("Invalid amount: {}", ctx.args[1])),
    };

    let game_data = data::exceldb::get();
    if game_data.item.get(item_id as i32).is_none() {
        return Ok(format!("Invalid item ID: {}", item_id));
    }

    let db = ctx.ctx.lock().await.state.db.clone();

    add_items(&db, ctx.user_id, &[(item_id, amount)]).await?;

    push::send_item_change_push(ctx.ctx.clone(), ctx.user_id, vec![item_id], vec![], vec![])
        .await?;

    let material_changes = vec![(1, item_id, amount)];
    push::send_material_change_push(ctx.ctx.clone(), material_changes, None).await?;

    Ok(format!("Added {} of item {}", amount, item_id))
}

async fn cmd_currency(ctx: CommandContext) -> Result<String, AppError> {
    if ctx.args.len() < 2 {
        return Ok("Usage: /currency <id> <amount>".to_string());
    }

    let currency_id: i32 = match ctx.args[0].parse() {
        Ok(id) => id,
        Err(_) => return Ok(format!("Invalid currency ID: {}", ctx.args[0])),
    };

    let amount: i32 = match ctx.args[1].parse() {
        Ok(amt) => amt,
        Err(_) => return Ok(format!("Invalid amount: {}", ctx.args[1])),
    };

    let db = ctx.ctx.lock().await.state.db.clone();

    add_currencies(&db, ctx.user_id, &[(currency_id, amount)]).await?;

    push::send_currency_change_push(ctx.ctx.clone(), ctx.user_id, vec![(currency_id, amount)])
        .await?;

    let material_changes = vec![(2, currency_id as u32, amount)];
    push::send_material_change_push(ctx.ctx.clone(), material_changes, None).await?;

    Ok(format!("Added {} of currency {}", amount, currency_id))
}

async fn cmd_level(ctx: CommandContext) -> Result<String, AppError> {
    if ctx.args.is_empty() {
        return Ok("Usage: /level <level>".to_string());
    }

    let level: i32 = match ctx.args[0].parse() {
        Ok(lvl) => lvl,
        Err(_) => return Ok(format!("Invalid level: {}", ctx.args[0])),
    };

    if level < 1 || level > 80 {
        return Ok("Level must be between 1 and 80".to_string());
    }

    let db = ctx.ctx.lock().await.state.db.clone();

    user::account::update_user_level(&db, ctx.user_id, level).await?;

    Ok(format!("Set level to {}", level))
}

async fn cmd_hero(ctx: CommandContext) -> Result<String, AppError> {
    if ctx.args.is_empty() {
        return Ok("Usage: /hero <id>".to_string());
    }

    let hero_id: i32 = match ctx.args[0].parse() {
        Ok(id) => id,
        Err(_) => return Ok(format!("Invalid hero ID: {}", ctx.args[0])),
    };

    let game_data = data::exceldb::get();
    if !game_data.character.iter().any(|c| c.id == hero_id) {
        return Ok(format!("Invalid hero ID: {}", hero_id));
    }

    let pool = ctx.ctx.lock().await.state.db.clone();

    let hero = UserHeroModel::new(ctx.user_id, pool);

    if hero.has_hero(hero_id).await? {
        return Ok(format!("You already have hero {}", hero_id));
    }

    hero.create_hero(hero_id).await?;

    Ok(format!("Added hero {}", hero_id))
}

async fn cmd_equip(ctx: CommandContext) -> Result<String, AppError> {
    if ctx.args.len() < 2 {
        return Ok("Usage: /equip <id> <amount>".to_string());
    }

    let equip_id: i32 = match ctx.args[0].parse() {
        Ok(id) => id,
        Err(_) => return Ok(format!("Invalid equipment ID: {}", ctx.args[0])),
    };

    let amount: i32 = match ctx.args[1].parse() {
        Ok(amt) => amt,
        Err(_) => return Ok(format!("Invalid amount: {}", ctx.args[1])),
    };

    let game_data = data::exceldb::get();
    if game_data.equip.get(equip_id).is_none() {
        return Ok(format!("Invalid equipment ID: {}", equip_id));
    }

    let db = ctx.ctx.lock().await.state.db.clone();

    let equip_uids: Vec<i64> =
        if equip_id == 1002 || equip_id == 1003 || equip_id == 1004 || equip_id == 1005 {
            game::equipment::update_equipment_count(&db, ctx.user_id, equip_id, amount).await?
        } else {
            game::equipment::add_equipments(&db, ctx.user_id, &[(equip_id, amount)]).await?
        };

    push::send_equip_update_push_by_uid(ctx.ctx.clone(), ctx.user_id, &equip_uids).await?;

    let material_changes = vec![(9, equip_id as u32, amount)];
    push::send_material_change_push(ctx.ctx.clone(), material_changes, None).await?;

    Ok(format!("Added {} of equipment {}", amount, equip_id))
}
