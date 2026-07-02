#![no_std]
#![no_main]

use defmt::debug;
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_rp::uart;
use embassy_time::Timer;
use embassy_rp::uart::Config;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut uart = uart::Uart::new_blocking(p.UART0, p.PIN_16, p.PIN_17, Config::default());

    loop {
        uart.blocking_write("howdy\r\n".as_bytes()).unwrap();
        Timer::after_secs(3).await;
    }
}