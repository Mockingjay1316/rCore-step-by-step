pub mod structs;

use structs::Thread;

#[no_mangle]
pub extern "C" fn temp_thread(from_thread: &mut Thread, current_thread: &mut Thread) {
    println!("I'm leaving soon, but I still want to say: Hello world!");
    current_thread.switch_to(from_thread);
}
pub fn init() {

    let mut boot_thread = Thread::get_boot_thread();
    let mut temp_thread = Thread::new_kernel(temp_thread as usize);

    unsafe {
        // 对于放在堆上的数据，我只想到这种比较蹩脚的办法拿到它所在的地址...
        temp_thread.append_initial_arguments([&*boot_thread as *const Thread as usize, &*temp_thread as *const Thread as usize, 0]);
    }
    boot_thread.switch_to(&mut temp_thread);

    println!("switched back from temp_thread!");
    loop {}
}