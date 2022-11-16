use fs::File;
use std::fs;
use std::io;
use std::io::Error;
use std::os::unix::io::{AsRawFd, FromRawFd};
use libc::{ioctl, mmap, size_t};

// ioctls from kvm.h in the Linux kernel
const KVM_CREATE_VM: u64 = 0xae01;
const KVM_CREATE_VCPU: u64 = 0xae41;
const KVM_GET_VCPU_MMAP_SIZE: u64 = 0xae04;

/// Helper to turn libc return values into an [io::Result](std::io::Result). Returns
/// [`Error::last_os_error`](std::io::Error::last_os_error) if `ret < 0`.
fn convert_os_err(ret: libc::c_int) -> io::Result<libc::c_int> {
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret)
    }
}

fn main() -> Result<(), Error> {
    println!("Create KVM device handle");
    let kvm = convert_os_err(unsafe {
        libc::open("/dev/kvm\0".as_ptr().cast(), libc::O_RDWR | libc::O_CLOEXEC)
    }).map(|fd| unsafe { File::from_raw_fd(fd) })?;

    println!("Fetching KVM VCPU mmap size");
    let vcpu_mmap_size = convert_os_err(unsafe { ioctl(kvm.as_raw_fd(), KVM_GET_VCPU_MMAP_SIZE, 0) })
        .map(|size| size as usize)?;
    println!("KVM VCPU size: {}", vcpu_mmap_size);

    println!("Creating VM instance");
    let vm = convert_os_err(unsafe { ioctl(kvm.as_raw_fd(), KVM_CREATE_VM, 0 /* machine id */) })
        .map(|fd| unsafe { File::from_raw_fd(fd) })?;

    println!("Creating KVM VCPU");
    let vcpu = convert_os_err(unsafe { ioctl(vm.as_raw_fd(), KVM_CREATE_VCPU, 0) })
        .map(|fd| unsafe { File::from_raw_fd(fd) })?;

    println!("Sharing MMAP space of VM's VCPU resource");
    let ptr = unsafe {
        mmap(
            std::ptr::null_mut(),
            vcpu_mmap_size as size_t,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            vcpu.as_raw_fd(),
            0,
        )
    };

    drop(ptr);
    Ok(())
}
