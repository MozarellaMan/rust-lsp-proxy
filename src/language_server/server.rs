use super::intercept::intercept_notification;
use actix::{Actor, AsyncContext, StreamHandler};
use actix_web_actors::ws;
use serde_json::Value;
use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout},
    stream::StreamExt,
    sync::Mutex,
};

pub struct LangServer {
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Option<ChildStdout>,
}
#[derive(Debug)]
struct Line(String);

impl LangServer {
    pub fn new(child: Arc<std::sync::Mutex<Child>>) -> Self {
        let mut child = child.lock().unwrap();
        LangServer {
            stdin: Arc::new(Mutex::new(child.stdin.take().unwrap())),
            stdout: Some(child.stdout.take().unwrap()),
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
        let stdout = self.stdout.take().unwrap();
        let reader = BufReader::new(stdout).lines();
        ctx.add_stream(reader.map(|l| {
            println!("{:?}", &l);
            Ok(Line(l.expect("Not a line")))
        }));
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for LangServer {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Text(text)) = msg {
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
    }
}
