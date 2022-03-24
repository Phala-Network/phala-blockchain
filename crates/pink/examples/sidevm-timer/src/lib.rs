extern "C" {
    fn sidevm_ocall(func_id: i32, p0: i32, p1: i32, p2: i32, p3: i32) -> i32;
}

const OCALL_CREATE_TIMER: i32 = 1;
const OCALL_POLL_TIMER: i32 = 2;

// entry point
#[no_mangle]
extern "C" fn sidevm_poll() -> i32 {
    static mut TIMER_ID: i32 = -1;
    unsafe {
        // create a timer if it doesn't exist
        if TIMER_ID == -1 {
            let timeout = 1000 * 10; // 10 seconds
            TIMER_ID = sidevm_ocall(OCALL_CREATE_TIMER, timeout, 0, 0, 0);
            if TIMER_ID == -1 {
                return 2;
            }
        }

        // poll the timer
        sidevm_ocall(OCALL_POLL_TIMER, TIMER_ID, 0, 0, 0)
    }
}