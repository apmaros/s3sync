use std::io::{Stdout, Write};
use crate::GenError;
use crossterm::{QueueableCommand, cursor};

pub fn rewrite_message(mut stdout: Stdout, msg: String) -> Result<(), GenError> {
    stdout.queue(cursor::SavePosition)?;
    stdout.write(msg.as_bytes())?;
    stdout.queue(cursor::RestorePosition)?;
    stdout.flush()?;
    Ok(())
}