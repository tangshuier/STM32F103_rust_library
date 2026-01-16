// Let me create a test file to understand the issue better
use library::*;
use core::ops::DerefMut;

// Test the timer methods
fn test_timer_methods() {
    // Check if the timer register blocks have the required methods
    unsafe {
        let tim1 = &mut *(0x40012C00 as *mut tim1::RegisterBlock);
        let tim2 = &mut *(0x40000000 as *mut tim2::RegisterBlock);
        
        // These should work
        tim1.ccmr1_output().write(|w| w.oc1m().bits(0b110).oc1pe().set_bit());
        tim2.ccmr1_output().write(|w| w.oc1m().bits(0b110).oc1pe().set_bit());
        
        tim1.ccer().modify(|_, w| w.cc1p().clear_bit().cc1e().set_bit());
        tim2.ccer().modify(|_, w| w.cc1p().clear_bit().cc1e().set_bit());
        
        tim1.ccr1().write(|w| w.ccr1().bits(1000));
        tim2.ccr1().write(|w| w.ccr1().bits(1000));
    }
}