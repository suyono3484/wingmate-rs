use nix::sys::wait;
use nix::unistd::Pid;

#[allow(dead_code)]
fn wait_all() {
    match wait::waitpid(Pid::from_raw(-1), Some(wait::WaitPidFlag::WNOHANG)) {
        Ok(_x) => {},
        Err(_err) => {},
    }
}