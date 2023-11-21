use nix::errno::Errno;
use nix::sys::wait::{self, WaitStatus};
use nix::unistd::Pid;

pub fn wait_all() {
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
                        break 'wait;
                    },
                    _ => {}
                }
            },
        }
        // dbg!("sanity");
    }

}