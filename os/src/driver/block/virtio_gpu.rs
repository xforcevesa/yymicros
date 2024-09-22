use virtio_drivers::{VirtIOGpu, VirtIOHeader};

use super::{virtio_blk::VirtioHal, VIRTIO1};

#[allow(unused)]
/// GPU test function
pub fn gpu_test() {
    let mut gpu = unsafe {
        VirtIOGpu::<VirtioHal>::new(&mut *(VIRTIO1 as *mut VirtIOHeader)).unwrap()
    };
    let (width, height) = gpu.resolution();
    let width = width as usize;
    let height = height as usize;
    println!("GPU resolution is {}x{}", width, height);
    let fb = gpu.setup_framebuffer().expect("failed to get fb");
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            fb[idx] = x as u8;
            fb[idx + 1] = y as u8;
            fb[idx + 2] = (x + y) as u8;
        }
    }
    gpu.flush().expect("failed to flush");
    //delay some time
    info!("virtio-gpu show graphics....");
    for _ in 0..10000 {
        for _ in 0..100000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }

    println!("virtio-gpu test finished");
}
