// $ DEFMT_LOG=info cargo rb rtic-tick
#![no_main]
#![no_std]

use f411_rtic as _; // global logger + panicking-behavior + memory layout
mod mono; // monotonic timer impl for TIM2

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART1])]
mod app {
    use super::mono::MonoTimer;
    use fugit::ExtU32;
    use stm32f4xx_hal::{pac, prelude::*};
    const FREQ: u32 = 48_000_000;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[monotonic(binds = TIM5, default = true)]
    type MyMono = MonoTimer<pac::TIM5, 1_000_000>;

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(FREQ.hz()).freeze();
        let mono = MyMono::new(ctx.device.TIM5, &clocks);
        tick::spawn().ok();
        (Shared {}, Local {}, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }

    #[task]
    fn tick(_: tick::Context) {
        defmt::info!("Tick!");
        tick::spawn_after(1.secs()).ok();
    }
}
