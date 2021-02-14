use actix_web::{dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};
use futures_util::{
    future::{AbortHandle, Abortable},
    TryFutureExt,
};
use std::process::Output;
use tokio::{io::AsyncWriteExt, process::Child, sync::Mutex};

#[derive(Debug)]
pub struct UserProgram(pub Option<Child>);

pub type UserProgramHandle = AbortHandle;

impl UserProgram {
    pub async fn wait_with_output(
        &mut self,
        handle_state: &Mutex<Option<UserProgramHandle>>,
    ) -> Result<Vec<u8>, UserProgramError> {
        if let Some(child) = self.0.take() {
            let (abort_handle, abort_registration) = AbortHandle::new_pair();
            let run_output = Abortable::new(
                child
                    .wait_with_output()
                    .map_err(|_| UserProgramError::NoOutput),
                abort_registration,
            );

            if let Ok(handle) = &mut handle_state.try_lock() {
                handle.replace(abort_handle.clone());
            }
            let run_output: Output = run_output
                .await
                .map_err(|_| UserProgramError::FailedRun)??;

            let output: Vec<u8> = run_output
                .stdout
                .into_iter()
                .chain(run_output.stderr.into_iter())
                .collect();
            return Ok(output);
        }
        Err(UserProgramError::NoProgram)
    }

    pub async fn read_user_program_input(
        &mut self,
        inputs: &[String],
    ) -> Result<(), UserProgramError> {
        if let Some(child) = &mut self.0 {
            if !inputs.is_empty() {
                if let Some(stdin) = &mut child.stdin {
                    for input in inputs.iter() {
                        if let Err(er) = stdin.write_all(&input.as_bytes()).await {
                            eprintln!("Error writing to child process {:?}", er);
                        }
                        stdin.flush();
                    }
                }
            }
            Ok(())
        } else {
            Err(UserProgramError::NoProgram)
        }
    }

    pub async fn stop(&mut self) -> Result<(), UserProgramError> {
        if let Some(child) = &mut self.0.take() {
            child.kill().map_err(|_| UserProgramError::FailedKill)?;
            self.0.take();
        }
        Err(UserProgramError::NoProgram)
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
