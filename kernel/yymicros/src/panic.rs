use crate::sbi::shutdown;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Print the panic message to the console
    println!("Panicked: {}", info);
    // Shutdown the system
    shutdown(true)
}