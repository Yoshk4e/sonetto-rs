use serde::Serialize;

// TODO: replace all code: .. msg: .. with
// this struct, using serde(flatten)
// #[derive(Serialize, Default)]
// pub struct CodeMsg {
//     pub code: u16,
//     pub msg: String,
// }

#[allow(dead_code)]
#[repr(u8)]
#[derive(Serialize, Default)]
pub enum AccountType {
    #[default]
    Email = 10,
    Bluepoch = 13, // not sure
    Steam = 14,    // Steam = 15,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GamePatchVersionRsp {
    pub latest_version: String,
    pub app_status: u16,
    pub login_uri: String,
    pub login_uri_bak: String,
    pub force_update: u8,
}

impl GamePatchVersionRsp {
    pub fn with_version(version: &str) -> Self {
        Self {
            latest_version: String::from(version),
            ..Default::default()
        }
    }
}

#[derive(Serialize, Default)]
pub struct GameConfigRsp {
    pub code: u16,
    pub msg: String,
    pub data: GameConfigRspData,
}

impl GameConfigRsp {
    pub fn with_timestamp(timestamp: u128) -> Self {
        Self {
            data: GameConfigRspData {
                server_timestamp: timestamp,
                sync_batch_size: 30,
                sync_interval: 30,
                disable_event_list: vec![
                    String::from("summon_client"),
                    String::from("ta_app_click"),
                    String::from("ta_app_crash"),
                    String::from("ta_app_view"),
                ],
            },
            code: 0,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Default)]
pub struct GameConfigRspData {
    pub server_timestamp: u128,
    pub sync_batch_size: i32,
    pub sync_interval: i32,
    pub disable_event_list: Vec<String>,
}

// {"code":200,"msg":"成功","data":{"shootFaceConfig":null}}
#[derive(Serialize, Default)]
pub struct GameNoticecpConfigRsp {
    pub code: u16,
    pub msg: String,
    pub data: GameNoticecpConfigRspData,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GameNoticecpConfigRspData {
    // not sure
    pub shoot_face_config: Option<[u8; 0]>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JspLoginRsp {
    pub account_tags: String,
    pub area_id: i32,
    pub is_admin: bool,
    pub result_code: u16,
    pub session_id: String,
    pub user_name: String,
    pub zone_info: ZoneInfo,
}

#[derive(Serialize, Default)]
pub struct ZoneInfo {
    pub default: bool,
    pub id: i32,
    pub name: String,
    pub prefix: String,
    pub state: u16,
}

impl ZoneInfo {
    pub fn zone_four() -> Self {
        Self {
            name: String::from("GL"),
            id: 4,
            state: 1,
            default: true,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JspLoadZoneRsp {
    pub last_login_zone_id: i32,
    pub recommend_zone_id: i32,
    pub result_code: u16,
    pub user_infos: Vec<ZoneUserInfo>,
    pub zone_infos: Vec<ZoneInfo>,
}

#[derive(Serialize, Default)]
pub struct ZoneUserInfo {
    pub id: u64,
    pub level: i32,
    pub name: String,
    pub portrait: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JspStartGameRsp {
    pub bak_ip: String,
    pub bak_pid: i32,
    pub bak_port: u16,
    pub ip: String,
    pub is_admin: bool,
    pub pid: i32,
    pub port: u16,
    pub result_code: u16,
    pub state: u16,
    pub tips: String,
}

impl Default for JspStartGameRsp {
    fn default() -> Self {
        Self {
            bak_ip: String::new(),
            bak_pid: 1,
            bak_port: 0,
            ip: String::new(),
            is_admin: false,
            pid: 1,
            port: 0,
            result_code: 0,
            state: 0,
            tips: String::from(
                "Maintenance time: 05:00 - 10:00, Apr.24th (UTC-5). For more info, please check Notice or our official X page.",
            ),
        }
    }
}

#[derive(Serialize, Default)]
pub struct AccountSdkInitRsp {
    pub code: u16,
    pub msg: String,
    pub data: AccountSdkInitRspData,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AccountSdkInitRspData {
    pub login_account_types: Option<Vec<i32>>, // Option<Vec<AccountType>>,
    pub user_center_items: Option<Vec<UserCenterItem>>,
    pub only_mail: Option<bool>,
    pub game_channel: Option<GameChannel>,
    pub biz_switch: Option<BizSwitch>,
    pub is_download_service: Option<bool>,
    pub is_show_stop_service_baffle: Option<bool>,
    pub is_ignore_file_missing: Option<bool>,
    pub is_open_c_m_p: Option<bool>,
    pub show_buttons: Option<ShowButtons>,
    pub is_unsupport_change_volume: bool,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserCenterItem {
    pub r#type: i32,
    pub lab_title: String,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GameChannel {
    pub game_id: i32,
    pub channel_id: i32,
    pub cp_name: String,
    pub app_id: String,
    pub app_key: String,
    pub call_interval: i32,
    pub relogin_interval: i32,
    pub relogin_times: i32,
    pub is_record_debug: bool,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BizSwitch {
    pub open_real_name_window: bool,
    pub force_real_name_auth: bool,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ShowButtons {
    pub notice: bool,
}

// LoginMail
// AutoLogin
// etc...
#[derive(Serialize, Default)]
pub struct AccountLoginRsp {
    pub code: u16,
    pub msg: String,
    pub data: AccountLoginRspData,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AccountLoginRspData {
    pub token: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub user_id: u64,
    pub account_type: AccountType,
    pub registration_account_type: i32,
    pub account: String,
    pub real_name_info: RealNameInfo,
    pub need_activate: bool,
    pub cipher_mark: bool,
    pub first_join: bool,
    pub account_tags: String,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RealNameInfo {
    pub need_real_name: bool,
    pub real_name_status: bool,
    pub age: u8,
    pub adult: bool,
}

#[derive(Serialize, Default)]
pub struct AccountLoginConfigRsp {
    pub code: u16,
    pub msg: String,
    pub data: AccountLoginConfigRspData,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AccountLoginConfigRspData {
    pub af_whitelist: bool,
}

#[derive(Serialize, Default)]
pub struct AccountBindListRsp {
    pub code: u16,
    pub msg: String,
    pub data: Vec<AccountBindListRspData>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AccountBindListRspData {
    pub user_id: u64,
    pub account: String,
    pub account_type: AccountType,
}

#[derive(Serialize, Default)]
pub struct AccountLoginVerifyRsp {
    pub code: u16,
    pub msg: String,
    pub data: AccountLoginVerifyRspData,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AccountLoginVerifyRspData {
    pub user_info: VerifyUserInfo,
    pub session_id: String,
    pub token: String,
    pub expires_in: i64,
    pub refresh_token: String,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VerifyUserInfo {
    pub channel_id: i32,
    pub user_id: String,
    pub real_name_status: bool,
    pub age: i32,
    pub adult: bool,
    pub first_join: bool,
    pub account_tags: String,
    pub bind_account_type_list: Vec<String>,
    pub first_join_time: String,
    pub register_time: String,
    pub is_pay_account: bool,
}

#[allow(dead_code)]
impl VerifyUserInfo {
    pub fn user_id(uid: u64) -> String {
        format!("200_{}", uid)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummonQueryRsp {
    pub code: u16,
    pub msg: String,
    pub data: SummonQueryRspData,
}

impl SummonQueryRsp {
    pub fn summons(data: SummonQueryRspData) -> Self {
        SummonQueryRsp {
            code: 200,
            msg: "成功".to_string(),
            data: data,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummonQueryRspData {
    pub page_data: Vec<PageDatum>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageDatum {
    pub summon_type: String,
    pub lucky_bag_ids: Vec<Option<serde_json::Value>>,
    pub create_time: String,
    pub pool_id: i64,
    pub gain_ids: Vec<i64>,
    pub pool_type: i64,
    pub pool_name: PoolName,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PoolName {
    #[serde(rename = "子夜独角戏")]
    Empty,
    #[serde(rename = "修缮往日")]
    Fluffy,
    #[serde(rename = "殓骨悼词")]
    PoolName,
    #[serde(rename = "石心瓦解时")]
    Purple,
}

impl PoolName {
    pub fn from_db(name: &str) -> Self {
        match name {
            "子夜独角戏" => PoolName::Empty,
            "修缮往日" => PoolName::Fluffy,
            "殓骨悼词" => PoolName::PoolName,
            "石心瓦解时" => PoolName::Purple,
            _ => PoolName::Empty, // safe fallback
        }
    }
}
