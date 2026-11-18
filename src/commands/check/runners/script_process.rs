use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const OUTPUT_LIMIT: u64 = 4 * 1024 * 1024;

pub(super) fn output_with_timeout(command: &mut Command, timeout: Duration) -> io::Result<Output> {
    let mut stdout = tempfile::tempfile()?;
    let mut stderr = tempfile::tempfile()?;
    command
        .stdout(Stdio::from(stdout.try_clone()?))
        .stderr(Stdio::from(stderr.try_clone()?));
    set_process_group(command);

    let mut child = command.spawn()?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Ok(Output {
                    status,
                    stdout: read_output(&mut stdout)?,
                    stderr: read_output(&mut stderr)?,
                });
            }
            Err(error) => {
                stop_process(&mut child);
                return Err(error);
            }
            Ok(None) => {}
        }
        let remaining = timeout.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            stop_process(&mut child);
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("timed out after {timeout:?}"),
            ));
        }
        thread::sleep(remaining.min(Duration::from_millis(20)));
    }
}

fn read_output(file: &mut File) -> io::Result<Vec<u8>> {
    if file.metadata()?.len() > OUTPUT_LIMIT {
        return Err(io::Error::other("script output exceeds 4 MiB per stream"));
    }
    file.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::new();
    file.take(OUTPUT_LIMIT).read_to_end(&mut bytes)?;
    Ok(bytes)
}

#[cfg(unix)]
fn set_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn set_process_group(_command: &mut Command) {}

fn stop_process(child: &mut Child) {
    #[cfg(unix)]
    if let Ok(id) = i32::try_from(child.id())
        && let Some(pid) = rustix::process::Pid::from_raw(id)
    {
        let _ = rustix::process::kill_process_group(pid, rustix::process::Signal::KILL);
    }
    let _ = child.kill();
    let _ = child.wait();
}
