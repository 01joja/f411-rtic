// $ cargo rb exti
#![no_main]
#![no_std]

use f411_rtic as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f4xx_hal::stm32, peripherals = true)]
mod app {
    use stm32f4xx_hal::{
        gpio::{gpioa::PA5, gpioc::PC13, Edge, ExtiPin, Input, Output, PullUp, PushPull},
        prelude::*,
    };

    #[resources]
    struct Resources {
        led: PA5<Output<PushPull>>,
        btn: PC13<Input<PullUp>>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (init::LateResources, init::Monotonics) {
        // Set up the system clock.
        let rcc = ctx.device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        // Set up the LED. On the Nucleo-F411RE it's connected to pin PA5.
        let gpioa = ctx.device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        // Set up the button. On the Nucleo-F411RE it's connected to pin PC13.
        let gpioc = ctx.device.GPIOC.split();
        let mut btn = gpioc.pc13.into_pull_up_input();
        let mut sys_cfg = ctx.device.SYSCFG.constrain();
        btn.make_interrupt_source(&mut sys_cfg);
        btn.enable_interrupt(&mut ctx.device.EXTI);
        btn.trigger_on_edge(&mut ctx.device.EXTI, Edge::FALLING);

        defmt::info!("Press button!");
        (init::LateResources { btn, led }, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = EXTI15_10, resources = [btn, led])]
    fn on_exti(mut ctx: on_exti::Context) {
        ctx.resources.btn.lock(|b| b.clear_interrupt_pending_bit());
        ctx.resources.led.lock(|l| l.toggle().ok());
        defmt::warn!("Button was pressed!");
    }
}
