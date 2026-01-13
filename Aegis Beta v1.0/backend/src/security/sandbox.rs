use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use nix::sys::stat::Mode;
use nix::unistd::{Gid, Uid};

use super::isolation::{VMSandbox, IsolationError};

#[derive(Debug)]
pub struct ResourceLimits {
    pub memory_limit_mb: u64,
    pub cpu_limit_percent: u32,
    pub disk_limit_mb: u64,
    pub network_limit_mbps: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory_limit_mb: 4096,
            cpu_limit_percent: 100,
            disk_limit_mb: 20480,
            network_limit_mbps: 100,
        }
    }
}

pub struct VMSandboxBuilder {
    sandbox: VMSandbox,
    limits: ResourceLimits,
    allowed_devices: Vec<String>,
    allowed_syscalls: Vec<String>,
    read_only_paths: Vec<PathBuf>,
    writable_paths: Vec<PathBuf>,
}

impl VMSandboxBuilder {
    pub fn new() -> Self {
        Self {
            sandbox: VMSandbox::new(),
            limits: ResourceLimits::default(),
            allowed_devices: vec![
                "/dev/null".to_string(),
                "/dev/zero".to_string(),
                "/dev/random".to_string(),
                "/dev/urandom".to_string(),
            ],
            allowed_syscalls: vec![
                "read".to_string(),
                "write".to_string(),
                "open".to_string(),
                "close".to_string(),
                "stat".to_string(),
                "fstat".to_string(),
                "lseek".to_string(),
                "mmap".to_string(),
                "mprotect".to_string(),
                "munmap".to_string(),
                "brk".to_string(),
                "rt_sigaction".to_string(),
                "rt_sigprocmask".to_string(),
                "rt_sigreturn".to_string(),
                "ioctl".to_string(),
                "pread64".to_string(),
                "pwrite64".to_string(),
                "readv".to_string(),
                "writev".to_string(),
                "access".to_string(),
                "pipe".to_string(),
                "select".to_string(),
                "dup".to_string(),
                "dup2".to_string(),
                "pause".to_string(),
                "nanosleep".to_string(),
                "getitimer".to_string(),
                "alarm".to_string(),
                "setitimer".to_string(),
                "getpid".to_string(),
                "sendfile".to_string(),
                "socket".to_string(),
                "connect".to_string(),
                "accept".to_string(),
                "sendto".to_string(),
                "recvfrom".to_string(),
                "sendmsg".to_string(),
                "recvmsg".to_string(),
                "shutdown".to_string(),
                "bind".to_string(),
                "listen".to_string(),
                "getsockname".to_string(),
                "getpeername".to_string(),
                "socketpair".to_string(),
                "setsockopt".to_string(),
                "getsockopt".to_string(),
                "clone".to_string(),
                "fork".to_string(),
                "vfork".to_string(),
                "execve".to_string(),
                "exit".to_string(),
                "wait4".to_string(),
                "kill".to_string(),
                "uname".to_string(),
                "semget".to_string(),
                "semop".to_string(),
                "semctl".to_string(),
                "shmdt".to_string(),
                "msgrcv".to_string(),
                "msgsnd".to_string(),
                "msgctl".to_string(),
                "fcntl".to_string(),
                "flock".to_string(),
                "fsync".to_string(),
                "fdatasync".to_string(),
                "truncate".to_string(),
                "ftruncate".to_string(),
                "getdents".to_string(),
                "getcwd".to_string(),
                "chdir".to_string(),
                "fchdir".to_string(),
                "rename".to_string(),
                "mkdir".to_string(),
                "rmdir".to_string(),
                "creat".to_string(),
                "link".to_string(),
                "unlink".to_string(),
                "symlink".to_string(),
                "readlink".to_string(),
                "chmod".to_string(),
                "fchmod".to_string(),
                "chown".to_string(),
                "fchown".to_string(),
                "lchown".to_string(),
                "umask".to_string(),
                "gettimeofday".to_string(),
                "getrlimit".to_string(),
                "getrusage".to_string(),
                "sysinfo".to_string(),
                "times".to_string(),
                "ptrace".to_string(),
                "getuid".to_string(),
                "syslog".to_string(),
                "getgid".to_string(),
                "setuid".to_string(),
                "setgid".to_string(),
                "geteuid".to_string(),
                "getegid".to_string(),
                "setpgid".to_string(),
                "getppid".to_string(),
                "getpgrp".to_string(),
                "setsid".to_string(),
                "setreuid".to_string(),
                "setregid".to_string(),
                "getgroups".to_string(),
                "setgroups".to_string(),
                "setresuid".to_string(),
                "getresuid".to_string(),
                "setresgid".to_string(),
                "getresgid".to_string(),
                "getpgid".to_string(),
                "setfsuid".to_string(),
                "setfsgid".to_string(),
                "getsid".to_string(),
                "capget".to_string(),
                "capset".to_string(),
                "rt_sigpending".to_string(),
                "rt_sigtimedwait".to_string(),
                "rt_sigqueueinfo".to_string(),
                "rt_sigsuspend".to_string(),
                "sigaltstack".to_string(),
                "utime".to_string(),
                "mknod".to_string(),
                "uselib".to_string(),
                "personality".to_string(),
                "ustat".to_string(),
                "statfs".to_string(),
                "fstatfs".to_string(),
                "sysfs".to_string(),
                "getpriority".to_string(),
                "setpriority".to_string(),
                "sched_setparam".to_string(),
                "sched_getparam".to_string(),
                "sched_setscheduler".to_string(),
                "sched_getscheduler".to_string(),
                "sched_get_priority_max".to_string(),
                "sched_get_priority_min".to_string(),
                "sched_rr_get_interval".to_string(),
                "mlock".to_string(),
                "munlock".to_string(),
                "mlockall".to_string(),
                "munlockall".to_string(),
                "vhangup".to_string(),
                "modify_ldt".to_string(),
                "pivot_root".to_string(),
                "_sysctl".to_string(),
                "prctl".to_string(),
                "arch_prctl".to_string(),
                "adjtimex".to_string(),
                "setrlimit".to_string(),
                "chroot".to_string(),
                "sync".to_string(),
                "acct".to_string(),
                "settimeofday".to_string(),
                "mount".to_string(),
                "umount2".to_string(),
                "swapon".to_string(),
                "swapoff".to_string(),
                "reboot".to_string(),
                "sethostname".to_string(),
                "setdomainname".to_string(),
                "iopl".to_string(),
                "ioperm".to_string(),
                "create_module".to_string(),
                "init_module".to_string(),
                "delete_module".to_string(),
                "get_kernel_syms".to_string(),
                "query_module".to_string(),
                "quotactl".to_string(),
                "nfsservctl".to_string(),
                "getpmsg".to_string(),
                "putpmsg".to_string(),
                "afs_syscall".to_string(),
                "tuxcall".to_string(),
                "security".to_string(),
                "gettid".to_string(),
                "readahead".to_string(),
                "setxattr".to_string(),
                "lsetxattr".to_string(),
                "fsetxattr".to_string(),
                "getxattr".to_string(),
                "lgetxattr".to_string(),
                "fgetxattr".to_string(),
                "listxattr".to_string(),
                "llistxattr".to_string(),
                "flistxattr".to_string(),
                "removexattr".to_string(),
                "lremovexattr".to_string(),
                "fremovexattr".to_string(),
                "tkill".to_string(),
                "time".to_string(),
                "futex".to_string(),
                "sched_setaffinity".to_string(),
                "sched_getaffinity".to_string(),
                "set_thread_area".to_string(),
                "io_setup".to_string(),
                "io_destroy".to_string(),
                "io_getevents".to_string(),
                "io_submit".to_string(),
                "io_cancel".to_string(),
                "get_thread_area".to_string(),
                "lookup_dcookie".to_string(),
                "epoll_create".to_string(),
                "epoll_ctl_old".to_string(),
                "epoll_wait_old".to_string(),
                "remap_file_pages".to_string(),
                "getdents64".to_string(),
                "set_tid_address".to_string(),
                "restart_syscall".to_string(),
                "semtimedop".to_string(),
                "fadvise64".to_string(),
                "timer_create".to_string(),
                "timer_settime".to_string(),
                "timer_gettime".to_string(),
                "timer_getoverrun".to_string(),
                "timer_delete".to_string(),
                "clock_settime".to_string(),
                "clock_gettime".to_string(),
                "clock_getres".to_string(),
                "clock_nanosleep".to_string(),
                "exit_group".to_string(),
                "epoll_wait".to_string(),
                "epoll_ctl".to_string(),
                "tgkill".to_string(),
                "utimes".to_string(),
                "vserver".to_string(),
                "mbind".to_string(),
                "set_mempolicy".to_string(),
                "get_mempolicy".to_string(),
                "mq_open".to_string(),
                "mq_unlink".to_string(),
                "mq_timedsend".to_string(),
                "mq_timedreceive".to_string(),
                "mq_notify".to_string(),
                "mq_getsetattr".to_string(),
                "kexec_load".to_string(),
                "waitid".to_string(),
                "add_key".to_string(),
                "request_key".to_string(),
                "keyctl".to_string(),
                "ioprio_set".to_string(),
                "ioprio_get".to_string(),
                "inotify_init".to_string(),
                "inotify_add_watch".to_string(),
                "inotify_rm_watch".to_string(),
                "migrate_pages".to_string(),
                "openat".to_string(),
                "mkdirat".to_string(),
                "mknodat".to_string(),
                "fchownat".to_string(),
                "futimesat".to_string(),
                "newfstatat".to_string(),
                "unlinkat".to_string(),
                "renameat".to_string(),
                "linkat".to_string(),
                "symlinkat".to_string(),
                "readlinkat".to_string(),
                "fchmodat".to_string(),
                "faccessat".to_string(),
                "pselect6".to_string(),
                "ppoll".to_string(),
                "unshare".to_string(),
                "set_robust_list".to_string(),
                "get_robust_list".to_string(),
                "splice".to_string(),
                "tee".to_string(),
                "sync_file_range".to_string(),
                "vmsplice".to_string(),
                "move_pages".to_string(),
                "utimensat".to_string(),
                "epoll_pwait".to_string(),
                "signalfd".to_string(),
                "timerfd_create".to_string(),
                "eventfd".to_string(),
                "fallocate".to_string(),
                "timerfd_settime".to_string(),
                "timerfd_gettime".to_string(),
                "accept4".to_string(),
                "signalfd4".to_string(),
                "eventfd2".to_string(),
                "epoll_create1".to_string(),
                "dup3".to_string(),
                "pipe2".to_string(),
                "inotify_init1".to_string(),
                "preadv".to_string(),
                "pwritev".to_string(),
                "rt_tgsigqueueinfo".to_string(),
                "perf_event_open".to_string(),
                "recvmmsg".to_string(),
                "fanotify_init".to_string(),
                "fanotify_mark".to_string(),
                "prlimit64".to_string(),
                "name_to_handle_at".to_string(),
                "open_by_handle_at".to_string(),
                "clock_adjtime".to_string(),
                "syncfs".to_string(),
                "sendmmsg".to_string(),
                "setns".to_string(),
                "getcpu".to_string(),
                "process_vm_readv".to_string(),
                "process_vm_writev".to_string(),
                "kcmp".to_string(),
                "finit_module".to_string(),
                "sched_setattr".to_string(),
                "sched_getattr".to_string(),
                "renameat2".to_string(),
                "seccomp".to_string(),
                "getrandom".to_string(),
                "memfd_create".to_string(),
                "kexec_file_load".to_string(),
                "bpf".to_string(),
                "execveat".to_string(),
                "userfaultfd".to_string(),
                "membarrier".to_string(),
                "mlock2".to_string(),
                "copy_file_range".to_string(),
                "preadv2".to_string(),
                "pwritev2".to_string(),
                "pkey_mprotect".to_string(),
                "pkey_alloc".to_string(),
                "pkey_free".to_string(),
                "statx".to_string(),
                "io_pgetevents".to_string(),
                "rseq".to_string(),
                "pidfd_send_signal".to_string(),
                "io_uring_setup".to_string(),
                "io_uring_enter".to_string(),
                "io_uring_register".to_string(),
                "open_tree".to_string(),
                "move_mount".to_string(),
                "fsopen".to_string(),
                "fsconfig".to_string(),
                "fsmount".to_string(),
                "fspick".to_string(),
                "pidfd_open".to_string(),
                "clone3".to_string(),
                "close_range".to_string(),
                "openat2".to_string(),
                "pidfd_getfd".to_string(),
                "faccessat2".to_string(),
                "process_madvise".to_string(),
                "epoll_pwait2".to_string(),
                "mount_setattr".to_string(),
                "quotactl_fd".to_string(),
                "landlock_create_ruleset".to_string(),
                "landlock_add_rule".to_string(),
                "landlock_restrict_self".to_string(),
                "memfd_secret".to_string(),
                "process_mrelease".to_string(),
                "futex_waitv".to_string(),
                "set_mempolicy_home_node".to_string(),
            ],
            read_only_paths: Vec::new(),
            writable_paths: Vec::new(),
        }
    }

    pub fn with_limits(mut self, limits: ResourceLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn with_user(mut self, uid: u32, gid: u32) -> Self {
        self.sandbox = self.sandbox.with_user(Uid::from_raw(uid), Gid::from_raw(gid));
        self
    }

    pub fn with_chroot(mut self, path: &str) -> Self {
        self.sandbox = self.sandbox.with_chroot(path);
        self
    }

    pub fn add_allowed_device(mut self, device: &str) -> Self {
        self.allowed_devices.push(device.to_string());
        self
    }

    pub fn add_read_only_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.read_only_paths.push(path.as_ref().to_path_buf());
        self
    }

    pub fn add_writable_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.writable_paths.push(path.as_ref().to_path_buf());
        self
    }

    pub fn build(self) -> VMSandbox {
        self.sandbox
    }

    pub fn setup_vm_environment(&self, vm_id: &str, base_path: &Path) -> Result<(), IsolationError> {
        // Create VM directory structure
        VMSandbox::create_vm_directory(vm_id, base_path)?;

        let vm_path = base_path.join(vm_id);
        
        // Setup device nodes
        self.setup_devices(&vm_path)?;
        
        // Setup filesystem
        self.setup_filesystem(&vm_path)?;
        
        // Apply resource limits
        self.apply_resource_limits(vm_id)?;
        
        // Setup seccomp filter
        self.setup_seccomp()?;

        Ok(())
    }

    fn setup_devices(&self, vm_path: &Path) -> Result<(), IsolationError> {
        let dev_path = vm_path.join("dev");
        
        for device in &self.allowed_devices {
            if Path::new(device).exists() {
                let dest = dev_path.join(
                    Path::new(device)
                        .file_name()
                        .ok_or_else(|| IsolationError::IoError(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Invalid device path",
                        )))?,
                );
                
                // Create device node (simplified - in reality would use mknod)
                fs::copy(device, &dest)?;
            }
        }

        Ok(())
    }

    fn setup_filesystem(&self, vm_path: &Path) -> Result<(), IsolationError> {
        // Create necessary directories
        let root = vm_path.join("root");
        
        // Setup read-only bind mounts
        for path in &self.read_only_paths {
            if path.exists() {
                let dest = root.join(path.strip_prefix("/").unwrap_or(path));
                fs::create_dir_all(dest.parent().unwrap())?;
                // In reality, would use mount --bind here
            }
        }
        
        // Setup writable directories
        for path in &self.writable_paths {
            let dest = root.join(path.strip_prefix("/").unwrap_or(path));
            fs::create_dir_all(&dest)?;
            
            // Set permissions
            let mut perms = fs::metadata(&dest)?.permissions();
            perms.set_mode(0o755); // rwxr-xr-x
            fs::set_permissions(&dest, perms)?;
        }

        Ok(())
    }

    fn apply_resource_limits(&self, vm_id: &str) -> Result<(), IsolationError> {
        // Apply cgroup limits
        let cgroup_path = format!("/sys/fs/cgroup/vm-manager/{}", vm_id);
        
        // Create cgroup directory
        fs::create_dir_all(&cgroup_path)?;
        
        // Set memory limit
        if self.limits.memory_limit_mb > 0 {
            let memory_limit = self.limits.memory_limit_mb * 1024 * 1024;
            fs::write(
                format!("{}/memory.limit_in_bytes", cgroup_path),
                memory_limit.to_string(),
            )?;
        }
        
        // Set CPU limit
        if self.limits.cpu_limit_percent < 100 {
            let cpu_quota = (self.limits.cpu_limit_percent as u64) * 1000; // Convert to microseconds
            fs::write(
                format!("{}/cpu.cfs_quota_us", cgroup_path),
                cpu_quota.to_string(),
            )?;
            fs::write(
                format!("{}/cpu.cfs_period_us", cgroup_path),
                "100000".to_string(), // 100ms period
            )?;
        }

        Ok(())
    }

    fn setup_seccomp(&self) -> Result<(), IsolationError> {
        // This is a simplified version
        // In production, you would use libseccomp to create and load a filter
        log::info!("Seccomp filter would be applied here ({} syscalls allowed)", self.allowed_syscalls.len());
        Ok(())
    }
}