// $ DEFMT_LOG=info cargo rb button-bool
#![no_main]
#![no_std]

use f411_rtic as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART1], compiler_passes = [standard])]
mod app {
    use dwt_systick_monotonic::{DwtSystick, ExtU32};
    use stm32f4xx_hal::{
        gpio::{gpioc::PC13, Edge, ExtiPin},
        prelude::*,
    };
    const FREQ: u32 = 48_000_000;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<FREQ>;

    #[shared]
    struct Shared {
        was_pressed: bool,
        btn: PC13<>,
    }

    #[local]
    struct Local {
        hold: Option<hold::SpawnHandle>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(FREQ.Hz()).freeze();

        let gpioc = ctx.device.GPIOC.split();
        let mut btn = gpioc.pc13.into_pull_up_input();
        let mut sys_cfg = ctx.device.SYSCFG.constrain();
        btn.make_interrupt_source(&mut sys_cfg);
        btn.enable_interrupt(&mut ctx.device.EXTI);
        btn.trigger_on_edge(&mut ctx.device.EXTI, Edge::RisingFalling);

        ctx.core.DCB.enable_trace();
        ctx.core.DWT.enable_cycle_counter();
        let mono = DwtSystick::new(
            &mut ctx.core.DCB,
            ctx.core.DWT,
            ctx.core.SYST,
            clocks.hclk().to_Hz(),
        );

        defmt::info!("Press the button!");
        (
            Shared {
                btn,
                was_pressed: false,
            },
            Local {
                hold: None
            },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    #[task(binds = EXTI15_10, shared = [btn])]
    fn on_exti(mut ctx: on_exti::Context) { 
        ctx.shared.btn.lock(ExtiPin::clear_interrupt_pending_bit);
        debounce::spawn_after(30.millis()).ok();
    }

    #[task(shared = [btn, was_pressed], local = [hold])]
    fn debounce(ctx: debounce::Context) {
        let hold = ctx.local.hold;
        if let Some(handle) = hold.take() {
            handle.cancel().ok();
        }
        (ctx.shared.btn, ctx.shared.was_pressed).lock(|btn, was_pressed| {
            if btn.is_low() {
                *was_pressed = true;
                *hold = hold::spawn_after(1.secs()).ok();
            } else if *was_pressed {
                *was_pressed = false;
                defmt::info!("Short press");
            }
        });
    }

    #[task(shared = [btn, was_pressed])]
    fn hold(ctx: hold::Context) {
        (ctx.shared.btn, ctx.shared.was_pressed).lock(|btn, was_pressed| {
            if btn.is_low() {
                *was_pressed = false;
                defmt::info!("Long press");
            }
        });
    }
}
