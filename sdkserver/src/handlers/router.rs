use crate::AppState;
use crate::handlers::{account, game, index, jsp};
use axum::Router;
use axum::routing::{get, post};
use paste::paste;

// // Example usage:
// router! {
//     name-of-function-and-module;
//     "/route" get route_handler;
//     "/more-route" post more_route_handler;
// }
//
// // it will then become:
// pub fn name-of-function-and-module -> Router {
//     Router::new()
//         .route("/route" get(name-of-function-and-module::route_handler::get))
//         .route("/more-route" post(name-of-function-and-module::more_route_handler::post))
// }
//

macro_rules! router {
    ($module:ident; $($route:literal $method:ident $handler:ident);* $(;)?) => {
        paste! {
            pub fn [<$module _router>]() -> Router<AppState> {
                Router::new()
                    $(.route($route, $method($module::$handler::$method)))*
            }
        }
    };
}

// these use crypto
router! {
    account;
    "/login/autologin" post auto_login;
    "/uidAccount/bindList" post bind_list;
    "/login/config" post login_config;
    "/login/mail" post login_mail;
    "/login/verify" post login_verify;
    "/sdk/init" post sdk_init;
}

router! {
    jsp;
    "/loadzone.jsp" get load_zone;
    "/login.jsp" get login;
    "/startgame.jsp" get start_game;
}

router! {
    game;
    "/v1.0/c2s/session/app/nativepc/60001" post c2s_session;
    "/config" get config;
    "/noticecp/config" get noticecp_config;
    "/noticecp/client/query" get noticecp_query;
    "/patch/60001/version" get patch_version;
    "/receiver/app" post receiver_app;
    "/resource/60001/check" get resource_check;
    "/query/summon" get summon_query;
}

router! {
    index;
    "/" get home;
    "/favicon.ico" get favicon;
}
