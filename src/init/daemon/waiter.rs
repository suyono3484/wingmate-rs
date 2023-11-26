use nix::errno::Errno;
use nix::sys::wait::{self, WaitStatus};
use nix::unistd::Pid;
use std::sync::{Mutex, Arc};
use std::{thread, time};
use tokio_util::sync::CancellationToken;

pub fn wait_all(flag: Arc<Mutex<bool>>, stop_sighandler: CancellationToken) {
    'wait: loop {
        match wait::waitpid(Pid::from_raw(-1), None) {
            Ok(x) => {
                // dbg!(x);
                match x {
                    WaitStatus::Exited(pid, v) => {
                        println!("wait_all: pid {}: exited with status {}", pid, v);
                    },
                    WaitStatus::Signaled(pid, sig, _dumped) => {
                        println!("wait_all: pid {} killed with signal {}", pid, sig);
                    },
                    _ => {}
                }
            },
            Err(err) => {
                dbg!(err);
                match err {
                    Errno::ECHILD => {
                        let fl = flag.lock().unwrap();
                        if *fl {
                            stop_sighandler.cancel();
                            break 'wait;
                        } else {
                            drop(fl);
                            thread::sleep(time::Duration::from_millis(100));
                        }
                    },
                    _ => {}
                }
            },
        }
    }

}