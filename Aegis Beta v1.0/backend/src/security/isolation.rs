use nix::sched::{unshare, CloneFlags};
use nix::unistd::{setgid, setuid, Gid, Uid};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum IsolationError {
    #[error("Failed to unshare namespaces: {0}")]
    UnshareFailed(#[from] nix::Error),
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Permission denied")]
    PermissionDenied,
}

pub struct VMSandbox {
    pub uid: Option<Uid>,
    pub gid: Option<Gid>,
    pub isolate_network: bool,
    pub isolate_pid: bool,
    pub isolate_mount: bool,
    pub chroot_path: Option<String>,
}

impl VMSandbox {
    pub fn new() -> Self {
        Self {
            uid: None,
            gid: None,
            isolate_network: true,
            isolate_pid: true,
            isolate_mount: true,
            chroot_path: None,
        }
    }

    pub fn with_user(mut self, uid: Uid, gid: Gid) -> Self {
        self.uid = Some(uid);
        self.gid = Some(gid);
        self
    }

    pub fn with_chroot(mut self, path: &str) -> Self {
        self.chroot_path = Some(path.to_string());
        self
    }

    pub fn apply(&self) -> Result<(), IsolationError> {
        // Drop privileges if specified
        if let Some(gid) = self.gid {
            setgid(gid)?;
        }
        if let Some(uid) = self.uid {
            setuid(uid)?;
        }

        // Unshare namespaces
        let mut flags = CloneFlags::empty();
        
        if self.isolate_pid {
            flags.insert(CloneFlags::CLONE_NEWPID);
        }
        
        if self.isolate_mount {
            flags.insert(CloneFlags::CLONE_NEWNS);
        }
        
        if self.isolate_network {
            flags.insert(CloneFlags::CLONE_NEWNET);
        }
        
        if !flags.is_empty() {
            unshare(flags)?;
        }

        // Apply chroot if specified
        if let Some(chroot_path) = &self.chroot_path {
            self.apply_chroot(chroot_path)?;
        }

        Ok(())
    }

    fn apply_chroot(&self, path: &str) -> Result<(), IsolationError> {
        let path = Path::new(path);
        
        // Verify path exists and is a directory
        if !path.exists() || !path.is_dir() {
            return Err(IsolationError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "Chroot path not found",
            )));
        }

        // Change root directory
        nix::unistd::chroot(path)?;
        
        // Change to root directory inside chroot
        nix::unistd::chdir("/")?;

        Ok(())
    }

    pub fn create_vm_directory(vm_id: &str, base_path: &Path) -> Result<(), IsolationError> {
        let vm_path = base_path.join(vm_id);
        
        // Create VM directory
        fs::create_dir_all(&vm_path)?;
        
        // Create necessary subdirectories
        fs::create_dir_all(vm_path.join("root"))?;
        fs::create_dir_all(vm_path.join("tmp"))?;
        fs::create_dir_all(vm_path.join("dev"))?;
        fs::create_dir_all(vm_path.join("proc"))?;
        
        // Set permissions (read-only for others)
        let mut perms = fs::metadata(&vm_path)?.permissions();
        perms.set_readonly(false);
        fs::set_permissions(&vm_path, perms)?;

        Ok(())
    }

    pub fn setup_network_isolation(vm_id: &str) -> Result<(), IsolationError> {
        // Create network namespace for VM
        let output = std::process::Command::new("ip")
            .args(&["netns", "add", vm_id])
            .output()?;

        if !output.status.success() {
            return Err(IsolationError::IoError(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr),
            )));
        }

        Ok(())
    }
}