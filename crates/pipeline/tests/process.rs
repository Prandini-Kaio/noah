#![cfg(unix)]
use noah_pipeline::{Job, Status, inspect};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    time::{Duration, Instant},
};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let root = std::env::temp_dir()
            .join(format!("noah-pipeline-{}", std::process::id()))
            .join(format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }
    fn job(&self, script: &str, timeout: Duration) -> Job {
        let images = self.0.join("photos");
        fs::create_dir(&images).unwrap();
        for n in 0..3 {
            fs::write(images.join(format!("{n}.jpg")), b"\xff\xd8\xfftest").unwrap();
        }
        let engine = self.0.join("engine.py");
        fs::write(&engine, format!("#!/usr/bin/env python3\n{script}")).unwrap();
        fs::set_permissions(&engine, fs::Permissions::from_mode(0o755)).unwrap();
        Job::create(
            &self.0.join("run"),
            &images,
            &engine,
            BTreeMap::new(),
            timeout,
        )
        .unwrap()
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn args_without_shell_and_artifact_integrity() {
    let t = Temp::new();
    let mut job = t.job("import sys\nprint(sys.argv[1])\n", Duration::from_secs(3));
    job.step("safe", vec!["$(touch unsafe); & space".into()])
        .unwrap();
    assert!(!job.root().join("unsafe").exists());
    let log = fs::read_to_string(job.root().join("logs/00-safe.stdout.log")).unwrap();
    assert_eq!(log.trim(), "$(touch unsafe); & space");
    fs::write(job.root().join("output.txt"), b"result").unwrap();
    job.artifact("output.txt", "test").unwrap();
    job.succeed().unwrap();
    assert_eq!(inspect(job.root()).unwrap().status, Status::Succeeded);
    fs::write(job.root().join("output.txt"), b"changed").unwrap();
    assert!(inspect(job.root()).is_err());
}
#[test]
fn failure_timeout_and_cancel_preserve_diagnostics() {
    for mode in ["failure", "timeout", "cancel", "log"] {
        let t = Temp::new();
        let script = match mode {
            "failure" => "import sys\nprint('reason',file=sys.stderr)\nsys.exit(7)\n",
            "log" => "import sys\nsys.stdout.write('x'*9000000)\n",
            _ => "import time\ntime.sleep(5)\n",
        };
        let timeout = if mode == "timeout" {
            Duration::from_millis(100)
        } else {
            Duration::from_secs(3)
        };
        let mut job = t.job(script, timeout);
        let cancel = if mode == "cancel" {
            let root = job.root().to_path_buf();
            Some(std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(75));
                noah_pipeline::cancel(&root).unwrap();
            }))
        } else {
            None
        };
        let start = Instant::now();
        let error = job.step("test", vec![]).unwrap_err();
        job.fail(&error.to_string()).unwrap();
        if let Some(thread) = cancel {
            thread.join().unwrap();
        }
        assert!(start.elapsed() < Duration::from_secs(4));
        let m = inspect(job.root()).unwrap();
        assert_eq!(
            m.status,
            if mode == "cancel" {
                Status::Cancelled
            } else {
                Status::Failed
            }
        );
        assert!(m.finished_ms.is_some());
        assert!(m.steps[0].diagnostic.is_some());
        if mode == "failure" {
            assert_eq!(m.steps[0].exit_code, Some(7));
            assert!(
                fs::read_to_string(job.root().join(&m.steps[0].stderr))
                    .unwrap()
                    .contains("reason")
            );
        }
    }
}
#[test]
fn images_are_snapshots_and_existing_run_is_not_reused() {
    let t = Temp::new();
    let job = t.job("print('ok')\n", Duration::from_secs(3));
    let input = job.manifest().inputs[0].clone();
    fs::write(t.0.join("photos").join(&input.original_name), b"different").unwrap();
    assert!(inspect(job.root()).is_ok());
    fs::write(job.root().join(input.snapshot), b"different").unwrap();
    assert!(inspect(job.root()).is_err());
    assert!(
        Job::create(
            job.root(),
            &t.0.join("photos"),
            &t.0.join("engine.py"),
            BTreeMap::new(),
            Duration::from_secs(1)
        )
        .is_err()
    );
}
