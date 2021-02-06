use actix::{Actor, AsyncContext, StreamHandler};
use actix_web::{error::ErrorBadRequest, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde_json::Value;
use std::{
    cell::Cell,
    process::Stdio,
    sync::{atomic::Ordering, Arc},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    stream::StreamExt,
    sync::Mutex,
};

use crate::{config::Lang, lsp_intercept::intercept_notification, AppState};

const TEST_JAVA_SERVER_PATH: &str = "/home/ayomide/Development/LanguageServers/Java/eclipse.jdt.ls/org.eclipse.jdt.ls.product/target/repository";

pub struct LangServer {
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Cell<ChildStdout>,
}
#[derive(Debug)]
struct Line(String);

impl LangServer {
    pub fn new(child: Arc<std::sync::Mutex<Child>>) -> Self {
        let mut child = child.lock().unwrap();
        LangServer {
            stdin: Arc::new(Mutex::new(child.stdin.take().unwrap())),
            stdout: Cell::new(child.stdout.take().unwrap()),
        }
    }
}

impl StreamHandler<Result<Line, ws::ProtocolError>> for LangServer {
    fn handle(&mut self, msg: Result<Line, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(line) = msg {
            ctx.text(line.0)
        }
    }
}

impl Actor for LangServer {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        /* Send the bytes received from stdout to ctx */
        unsafe {
            let stdout = &mut *self.stdout.as_ptr();
            let reader = BufReader::new(stdout).lines();
            ctx.add_stream(reader.map(|l| {
                println!("{:?}", &l);
                Ok(Line(l.expect("Not a line")))
            }));
        }
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for LangServer {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let stdin = self.stdin.clone();

                let msg = serde_json::from_str::<Value>(&text);
                println!("StartClient\n{}\nEndClient", &text);

                let intercept_fut = async move {
                    if let Ok(msg) = msg {
                        if let Err(err) = intercept_notification(msg).await {
                            println!("err: {}", err);
                        };
                    }
                };
                let lang_server_fut = async move {
                    let mut stdin = stdin.lock().await;
                    let text = format!("Content-Length: {}\r\n\r\n{}", text.len(), text);
                    if let Err(er) = stdin.write_all(&text.as_bytes()).await {
                        eprintln!("Error writing to language server! {:?}", er);
                    }
                    stdin.flush();
                };

                let lang_server_fut = actix::fut::wrap_future(lang_server_fut);
                let intercept_fut = actix::fut::wrap_future(intercept_fut);
                ctx.spawn(intercept_fut);
                ctx.spawn(lang_server_fut);
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Err(_) => {}
            _ => {}
        }
    }
}

pub fn start_lang_server(lang: Lang, file_path: String) -> Option<Child> {
    match lang {
        Lang::Java => Some(
            Command::new("java")
                .current_dir(TEST_JAVA_SERVER_PATH)
                .arg("-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=1044")
                .arg("-Declipse.application=org.eclipse.jdt.ls.core.id1")
                .arg("-Dosgi.bundles.defaultStartLevel=4")
                .arg("-Declipse.product=org.eclipse.jdt.ls.core.product")
                .arg("-Dlog.level=ALL")
                .arg("-noverify")
                .arg("-Xmx1G")
                .arg("-jar")
                .arg("./plugins/org.eclipse.equinox.launcher_1.6.0.v20200915-1508.jar")
                .arg("-configuration")
                .arg("./config_linux")
                .arg("-data")
                .arg(file_path)
                .arg("--add-modules=ALL-SYSTEM")
                .arg("--add-opens")
                .arg("java.base/java.util=ALL-UNNAMED")
                .arg("--add-opens")
                .arg("java.base/java.lang=ALL-UNNAMED")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("failed to execute"),
        ),
        Lang::C => None,
    }
}

pub async fn to_lsp(
    req: HttpRequest,
    stream: web::Payload,
    process: web::Data<Arc<std::sync::Mutex<Child>>>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session_started = state.ws_session_started.load(Ordering::Relaxed);
    if !session_started {
        let lang_server = LangServer::new(process.as_ref().to_owned());
        let session = ws::start(lang_server, &req, stream);
        println!("Language Server started\n{:?}", session);
        //*session_started.get_mut() = true;
        state.ws_session_started.store(true, Ordering::Relaxed);
        session
    } else {
        Err(ErrorBadRequest(
            "Language server web socket session already started.",
        ))
    }
}
