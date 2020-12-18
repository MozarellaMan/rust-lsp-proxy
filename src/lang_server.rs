use std::{cell::Cell, process::Stdio, sync::Arc};

use actix::{Actor, AsyncContext, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    stream::StreamExt,
    sync::Mutex,
};

use crate::config::Lang;

const TEST_JAVA_SERVER_PATH: &str = "/home/ayomide/Development/LanguageServers/Java/eclipse.jdt.ls/org.eclipse.jdt.ls.product/target/repository";

struct LangServer {
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
        match msg {
            Ok(line) => ctx.text(line.0),
            _ => (), //Handle errors
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
            ctx.add_stream(reader.map(|l| Ok(Line(l.expect("Not a line")))));
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

                let text = format!("Content-Length: {}\r\n\r\n{}", text.len(), text);

                let fut = async move {
                    let mut stdin = stdin.lock().await;

                    if let Err(er) = stdin.write_all(&text.as_bytes()).await {
                        eprintln!("Error writing to language server! {:?}", er);
                    }
                    stdin.flush();
                };

                let fut = actix::fut::wrap_future(fut);
                ctx.spawn(fut);
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Err(_) => {}
            _ => {}
        }
    }
}

pub fn start_lang_server(lang: Lang, file_path: String) -> Option<Child> {
    match lang {
        Lang::Java => {
               Some(Command::new("java")
                .current_dir(TEST_JAVA_SERVER_PATH)
                .arg("-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=1044")
                .arg("-Declipse.application=org.eclipse.jdt.ls.core.id1")
                .arg("-Dosgi.bundles.defaultStartLevel=4")
                .arg("-Declipse.product=org.eclipse.jdt.ls.core.product")
                .arg("-Dlog.level=ALL")
                .arg("-noverify")
                .arg("-Xmx1G")
                .arg("-jar")
                .arg("./plugins/org.eclipse.equinox.launcher_1.5.700.v20200207-2156.jar")
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
                .expect("failed to execute"))
        },
        Lang::C => None
    }
}

pub async fn to_lsp(
    req: HttpRequest,
    stream: web::Payload,
    process: web::Data<Arc<std::sync::Mutex<Child>>>,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(LangServer::new(process.as_ref().to_owned()), &req, stream);
    println!("{:?}", resp);
    resp
}
