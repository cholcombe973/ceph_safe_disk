extern crate users;

use error::CSDError;
use users::*;

use std::process::Command;

pub fn call_ceph(cmd: &str) -> Result<String, CSDError> {
    debug!("calling ceph {} -f json", cmd);
    let ceph = Command::new("/usr/bin/env")
        .args(&["sh", "-c", &format!("ceph {} -f json", cmd)])
        .output()?;
    if ceph.status.success() {
        let stdout = String::from_utf8(ceph.stdout)?;
        trace!("ceph_cmd stdout: {}", stdout.trim_start());
        Ok(stdout.trim_start().to_string())
    } else {
        let stderr = String::from_utf8(ceph.stderr)?;
        Err(CSDError::CephExecError(stderr))
    }
}

// Check which user this is being run as
pub fn check_user() -> Result<(), CSDError> {
    match get_current_username() {
        Some(user) => {
            let user = user.to_string_lossy();
            match user.as_ref() {
                "ceph" => Ok(()),
                "root" => Ok(()),
                _ => Err(CSDError::ExecError),
            }
        }
        None => Err(CSDError::ExecError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn check_user_panic() {
        assert!(check_user().is_ok());
    }
}
