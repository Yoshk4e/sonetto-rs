use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::{friends, player_infos::get_player_info_data};
use prost::Message;
use sonettobuf::{
    ChatMsg, ChatMsgPush, CmdId, DeleteOfflineMsgReply, FriendInfo, GetApplyListReply,
    GetBlacklistReply, GetFriendInfoListReply, GetRecommendedFriendsReply, LoadFriendInfosReply,
    SendMsgReply, SendMsgRequest,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_send_msg(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = SendMsgRequest::decode(&req.data[..])?;
    tracing::info!("Received SendMsgRequest: {:?}", request);

    let input = request.content.clone().unwrap_or_default();

    let (player_id, username, portrait, level) = {
        let conn = ctx.lock().await;
        let pool = conn.state.db.clone();
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let player_data = get_player_info_data(&pool, player_id)
            .await?
            .ok_or(AppError::InvalidRequest)?;
        (
            player_id as u64,
            player_data.user_info.username.clone(),
            player_data.player_info.portrait as u32,
            player_data.user_info.level as u32,
        )
    };

    let now = common::time::ServerTime::now_ms() as u64;
    let mut messages = Vec::new();

    // game expects the message to be in a specific order or else it fails
    // also since this message is for bot usage only we only make it exists in the current session
    // ig resets every time you log in
    let (player_msg_id, bot_msg_id) = {
        let mut conn = ctx.lock().await;
        let base = conn.bot_msg_counter;
        conn.bot_msg_counter += 2;
        (base, base + 1)
    };

    messages.push(ChatMsg {
        msg_id: Some(player_msg_id),
        sender_id: Some(player_id),
        channel_type: request.channel_type,
        sender_name: Some(username),
        portrait: Some(portrait),
        content: request.content.clone(),
        send_time: Some(now),
        level: Some(level),
        recipient_id: Some(1337),
        msg_type: request.msg_type,
        ext_data: request.ext_data.clone(),
    });

    if let Some(bot_text) = handle_command(ctx.clone(), &input).await {
        messages.push(ChatMsg {
            msg_id: Some(bot_msg_id),
            sender_id: Some(1337),
            channel_type: request.channel_type,
            sender_name: Some("Sonetto Bot".to_string()),
            portrait: Some(171805),
            content: Some(bot_text),
            send_time: Some(now),
            level: Some(80),
            recipient_id: Some(player_id),
            msg_type: Some(0),
            ext_data: Some(String::new()),
        });
    }

    {
        let mut conn = ctx.lock().await;
        conn.notify(CmdId::ChatMsgPushCmd, ChatMsgPush { msg: messages })
            .await?;
    }

    let reply = SendMsgReply {
        channel_type: request.channel_type,
        message: None,
        content: request.content,
        msg_type: request.msg_type,
        ext_data: request.ext_data,
    };

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::SendMsgCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}

pub async fn on_load_friend_infos(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (friend_ids, blacklist_ids) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        let friends = friends::get_friend_ids(&conn.state.db, player_id).await?;
        let blacklist = friends::get_blacklist_ids(&conn.state.db, player_id).await?;

        (friends, blacklist)
    };

    let reply = LoadFriendInfosReply {
        friend_ids,
        black_list_ids: blacklist_ids,
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::LoadFriendInfosCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_delete_offline_msg(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let reply = DeleteOfflineMsgReply {};

    // Honestly not sure if this is needed here but leave it for now
    let push = ChatMsgPush { msg: vec![] };

    {
        let mut conn = ctx.lock().await;
        conn.notify(CmdId::ChatMsgPushCmd, push).await?;
    }

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::DeleteOfflineMsgCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}

pub async fn on_get_friend_info_list(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let player_id = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        player_id as u64
    };

    let friends = vec![FriendInfo {
        user_id: Some(1337),
        level: Some(80),
        time: None, //setting a time actually makes you appear offline
        name: Some("Sonetto Bot".to_string()),
        portrait: Some(171805),
        desc: Some("".to_string()),
        infos: vec![], //this is where you friends hero profile would appear but we don't need that for a bot
        bg: None,      // ig this is the background
    }];

    let reply = GetFriendInfoListReply { info: friends };

    {
        let mut conn = ctx.lock().await;
        if !conn.bot_welcome_sent {
            let id = conn.bot_msg_counter;
            conn.bot_msg_counter += 1;
            send_bot_welcome(&mut conn, player_id, id).await?;
        }
    }

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::GetFriendInfoListCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}

pub async fn send_bot_welcome(
    ctx: &mut ConnectionContext,
    player_id: u64,
    msg_id: u64,
) -> Result<(), AppError> {
    if ctx.bot_welcome_sent {
        return Ok(());
    }

    let msg = ChatMsg {
        msg_id: Some(msg_id),
        sender_id: Some(1337),
        channel_type: Some(1),
        sender_name: Some("Sonetto Bot".to_string()),
        portrait: Some(171805),
        content: Some(
            "Welcome to Sonetto-rs\nGM commands available\nType: /help for commands".to_string(),
        ),
        send_time: None,
        level: Some(80),
        recipient_id: Some(player_id),
        msg_type: Some(0),
        ext_data: Some(String::new()),
    };

    ctx.notify(
        CmdId::ChatMsgPushCmd,
        sonettobuf::ChatMsgPush { msg: vec![msg] },
    )
    .await?;

    ctx.bot_welcome_sent = true;
    Ok(())
}

pub async fn handle_command(ctx: Arc<Mutex<ConnectionContext>>, input: &str) -> Option<String> {
    if !input.starts_with("/") {
        return None;
    }

    match crate::handlers::gm::execute_command(ctx, input).await {
        Ok(response) => Some(response),
        Err(e) => Some(format!("Error: {:?}", e)),
    }
}

pub async fn on_get_recommended_friends(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let now = common::time::ServerTime::now_ms();
    let friends = vec![FriendInfo {
        user_id: Some(1337),
        level: Some(80),
        time: Some(now as u64),
        name: Some("Sonetto Bot".to_string()),
        portrait: Some(171603),
        desc: Some("Server commands sent here".to_string()),
        infos: vec![],
        bg: None,
    }];

    let reply = GetRecommendedFriendsReply {
        info: friends,
        message: Some("Sonetto Bot".to_string()),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetRecommendedFriendsCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_get_blacklist(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let reply = GetBlacklistReply { info: vec![] };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetBlacklistCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_get_apply_list(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let reply = GetApplyListReply { info: vec![] };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetApplyListCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
