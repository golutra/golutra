//! 调试快照导出：按固定频率把虚拟屏幕写入日志，便于追踪状态变化。

use std::{
    fs::{create_dir_all, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Arc,
    thread,
    time::Duration,
};

use crate::now_millis;

use super::{lock_sessions, TerminalManager};

const SNAPSHOT_DUMP_IDLE_SLEEP_MS: u64 = 200;

pub(crate) fn spawn_snapshot_dumper(manager: &TerminalManager, log_dir: PathBuf) {
    if read_snapshot_dump_interval().is_none() {
        return;
    }
    let sessions = Arc::clone(&manager.sessions);
    thread::spawn(move || {
        if let Err(err) = create_dir_all(&log_dir) {
            log::warn!("terminal snapshot dump skipped: log dir error {err}");
            return;
        }
        loop {
            let interval = match read_snapshot_dump_interval() {
                Some(value) => value,
                None => {
                    thread::sleep(Duration::from_millis(SNAPSHOT_DUMP_IDLE_SLEEP_MS));
                    continue;
                }
            };
            let now = match now_millis() {
                Ok(value) => value,
                Err(_) => 0,
            };
            let targets = {
                let guard = lock_sessions(&sessions);
                guard
                    .sessions
                    .values()
                    .filter(|session| session.active && session.handle.is_some())
                    .map(|session| {
                        (
                            session.id.clone(),
                            session.member_name.clone(),
                            session.output_seq,
                            session.snapshot.snapshot_lines(),
                        )
                    })
                    .collect::<Vec<_>>()
            };
            for (terminal_id, member_name, seq, lines) in targets {
                let prefix = member_name
                    .map(|name| format!("{}-", name.replace(|c: char| !c.is_alphanumeric(), "_")))
                    .unwrap_or_default();
                let log_path = log_dir.join(format!("{prefix}{terminal_id}-snapshot-stream.log"));
                let mut file = match OpenOptions::new().create(true).append(true).open(&log_path) {
                    Ok(file) => file,
                    Err(err) => {
                        log::warn!(
                            "terminal snapshot dump open failed terminal_id={} err={}",
                            terminal_id,
                            err
                        );
                        continue;
                    }
                };
                if writeln!(
                    file,
                    "# ts={} terminal_id={} seq={} line_count={}",
                    now,
                    terminal_id,
                    seq,
                    lines.len()
                )
                .is_err()
                {
                    continue;
                }
                for (index, line) in lines.iter().enumerate() {
                    let sanitized = line.replace('\t', "\\t").replace('\r', "\\r");
                    let _ = writeln!(file, "{:04} | {}", index.saturating_add(1), sanitized);
                }
            }
            thread::sleep(interval);
        }
    });
}

fn read_snapshot_dump_interval() -> Option<Duration> {
    let raw = std::env::var("GOLUTRA_Backend_DEBUG").ok()?;
    let value: f64 = raw.trim().parse().ok()?;
    if value <= 0.0 {
        return None;
    }
    let ms = (value * 1000.0).round() as u64;
    Some(Duration::from_millis(ms.max(1)))
}
