mod test_helper;

use lsp_proxy::lang_server::start_lang_server;

use test_helper::{COMMON_TEST_LANG, _COMMON_TEST_DIRECTORY};

fn _server_starts() {
    let server = start_lang_server(COMMON_TEST_LANG, _COMMON_TEST_DIRECTORY.parse().unwrap());

    match server {
        Some(mut a) => {
            a.kill().expect("server could not be killed!");
        }

        None => assert!(false, "server could not be started!"),
    }
}
