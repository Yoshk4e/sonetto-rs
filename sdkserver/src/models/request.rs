use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct GameC2SSessionReq {
    pub timestamp: String,
    pub device_os_version: String,
    pub device_model: String,
    pub app_version: String,
    pub device_ids: Vec<DeviceId>,
    pub request_id: String,
    pub limit_ad_tracking: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct DeviceId {
    #[serde(rename = "type")]
    pub device_id_type: String,
    pub value: String,
}

#[derive(Deserialize)]
pub struct GamePatchVersionReq {
    // query
    pub version: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub network_name: String,
    pub device_id: String,
    pub cnadid: String,
    pub oa_id: String,
    pub android_id: String,
    pub imsi: String,
    pub imei: String,
    pub uuid: String,
    pub device_name: String,
    pub device_manufacturer: String,
    pub os_type: i64,
    pub os_version: String,
    pub api_level: String,
    pub language: String,
    pub display_width: String,
    pub display_height: String,
    pub hardware: String,
    pub build_name: String,
    pub distinct_id: String,
    pub anonymous_id: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPackageInfo {
    pub app_package_name: String,
    pub app_version: i64,
    pub app_version_name: String,
    pub game_id: i64,
    pub game_code: String,
    pub game_name: String,
    pub channel_id: String,
    pub sub_channel_id: String,
    pub app_install_time: String,
    pub app_update_time: String,
    pub app_signature: String,
    pub sdk_version: String,
    pub channel_version: String,
    pub ad_fid: String,
    pub gclid: String,
    pub data_app_id: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountLoginMailReq {
    pub device_info: DeviceInfo,
    pub app_package_info: AppPackageInfo,
    pub reactivate: bool,
    pub account: String,
    pub pwd: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountBindListReq {
    pub device_info: DeviceInfo,
    pub app_package_info: AppPackageInfo,
    pub reactivate: bool,
    pub token: String,
    pub user_id: u64,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountAutoLoginReq {
    pub device_info: DeviceInfo,
    pub app_package_info: AppPackageInfo,
    pub reactivate: bool,
    pub token: String,
    pub user_id: u64,
    pub account_type: i32,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountLoginVerifyReq {
    pub device_info: DeviceInfo,
    pub app_package_info: AppPackageInfo,
    pub user_id: String,
    pub token: String,
    #[serde(default)]
    pub ext_args: serde_json::Value,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct JspLoginQuery {
    #[serde(rename = "slSessionId")]
    pub sl_session_id: String,
    #[serde(rename = "clientVersion")]
    pub client_version: String,
    #[serde(rename = "sysType")]
    pub sys_type: i32,
    #[serde(rename = "accountId")]
    pub account_id: String, // Format: "200_1337"
    #[serde(rename = "channelId")]
    pub channel_id: String,
    #[serde(rename = "subChannelId")]
    pub sub_channel_id: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct LoadZoneQuery {
    #[serde(rename = "sessionId")]
    pub session_id: String, // This is the token
    #[serde(rename = "zoneId")]
    pub zone_id: i32,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct SummonQueryReq {
    #[serde(rename = "userId")]
    pub user_id: i64,
    #[serde(rename = "token")]
    pub token: String,
}
