//! [`critical_section`] implementation for **single core cpu boards**
//! using RIOTs interrupts interface. (Just disables interrupts).
//!
//! This is needed by [`portable_atomic`] to provide atomics to boards without hardware
//! support or rather where rust does not support atomics.

use critical_section::RawRestoreState;

struct RIOTCriticalSection;

critical_section::set_impl!(RIOTCriticalSection);

unsafe impl critical_section::Impl for RIOTCriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        if crate::irq_is_in() {
            return false;
        }

        let disabled = crate::irq_is_enabled();
        // It is possible that between here an interrupt
        // interferes and causes another thread/process to continue.
        // This should not be a problem because:
        // Szenario 1: The other process does not enter a critical section
        //   or disables interrupts and nothing happens.
        // Szenario 2: The other process does disable interrupts, but
        //   then this thread here can only continue once the interrupts are back on
        //   and one causes this thread to continue.
        // In both cases we should not face a toctu problem.
        crate::irq_disable();
        disabled
    }

    unsafe fn release(token: RawRestoreState) {
        if token {
            crate::irq_enable();
        }
    }
}
