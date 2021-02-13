use actix_web::{dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse};
use tokio::{io::AsyncWriteExt, process::Child};

use derive_more::{Display, Error};

#[derive(Debug)]
pub struct UserProgram(pub Option<Child>);

impl UserProgram {
    pub async fn wait_with_output(&mut self) -> Result<Vec<u8>, UserProgramError> {
        if let Some(child) = self.0.take() {
            let run_output = child
                .wait_with_output()
                .await
                .map_err(|_| UserProgramError::FailedRun)?;

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
        } else  {
            Err(UserProgramError::NoProgram)
        }
    }

    pub async fn stop(&mut self) -> Result<(), UserProgramError> {
        if let Some(child) = &mut self.0.take() {
            child.kill().map_err(|_| UserProgramError::FailedKill)?;
        }
        Err(UserProgramError::NoProgram)
    }
}

#[derive(Debug, Display, Error)]
pub enum UserProgramError {
    #[display(fmt = "Program failed to start")]
    FailedRun,
    #[display(fmt = "Program failed to compile")]
    FailedCompilation,
    #[display(fmt = "No program to run")]
    NoProgram,
    #[display(fmt = "Program failed to exit")]
    FailedKill,
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
