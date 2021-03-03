use actix::{Actor, AsyncContext, StreamHandler};
use actix_web::{dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse};
use actix_web_actors::ws;
use derive_more::{Display, Error};
use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout},
    stream::StreamExt,
    sync::Mutex,
};

use crate::Line;

#[derive(Debug)]
pub struct UserProgram {
    child: Option<Child>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    stdout: Option<ChildStdout>,
}

impl StreamHandler<Result<Line, ws::ProtocolError>> for UserProgram {
    fn handle(&mut self, msg: Result<Line, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(line) = msg {
            ctx.text(line.0)
        }
    }
}

impl Actor for UserProgram {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        /* Send the bytes received from stdout to ctx */
        let stdout = self.stdout.take().unwrap();
        let reader = BufReader::new(stdout).lines();
        ctx.add_stream(reader.map(|l| {
            if let Ok(l) = l {
                Ok(Line(l))
            } else {
                Ok(Line("Failed to read from user program".to_string()))
            }
        }));
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for UserProgram {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Text(text)) = msg {
            let stdin = self.stdin.clone();
            let user_program_fut = async move {
                let mut stdin = stdin.lock().await;
                if stdin.is_some() {
                    let stdin = stdin.as_mut().unwrap();
                    let text = format!("{}\n", text);
                    if let Err(er) = stdin.write_all(&text.as_bytes()).await {
                        eprintln!("Error writing to program! {:?}", er);
                    }
                    stdin.flush();
                }
            };
            let user_program_fut = actix::fut::wrap_future(user_program_fut);
            ctx.spawn(user_program_fut);
        }
    }
}

impl UserProgram {
    pub fn start(child: Child) -> Self {
        let mut child = child;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        UserProgram {
            child: Some(child),
            stdin: Arc::new(Mutex::new(stdin)),
            stdout,
        }
    }
}

#[derive(Debug, Display, Error, Clone, Copy)]
pub enum UserProgramError {
    #[display(fmt = "Program failed to start")]
    FailedRun,
    #[display(fmt = "Program failed to compile")]
    FailedCompilation,
    #[display(fmt = "No program to run")]
    NoProgram,
    #[display(fmt = "Program failed to exit")]
    FailedKill,
    #[display(fmt = "Running this programming language is not currently supported")]
    UnsupportedLanguage,
    #[display(fmt = "Failed to acuqire lock on running program")]
    FailedProgramLock,
    #[display(fmt = "Failed to get output from the program")]
    NoOutput,
}

impl error::ResponseError for UserProgramError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
