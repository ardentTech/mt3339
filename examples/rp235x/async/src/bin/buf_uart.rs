#![no_std]
#![no_main]

use defmt::{debug, info, unwrap};
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, BufferedUartRx, Config};
use embassy_time::Timer;
use embedded_io_async::{Read, Write};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let (tx_pin, rx_pin, uart) = (p.PIN_16, p.PIN_17, p.UART0);

    static TX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; 16])[..];
    static RX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; 16])[..];
    let uart = BufferedUart::new(uart, tx_pin, rx_pin, Irqs, tx_buf, rx_buf, Config::default());
    let (mut tx, rx) = uart.split();

    spawner.spawn(unwrap!(reader(rx)));

    let data = [
        1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
        29, 30, 31,
    ];

    loop {
        tx.write_all(&data).await.unwrap();
        Timer::after_secs(3).await;
    }
}

#[embassy_executor::task]
async fn reader(mut rx: BufferedUartRx) {
    info!("Reading...");
    loop {
        let mut buf = [0; 31];
        rx.read_exact(&mut buf).await.unwrap();
        info!("RX {:?}", buf);
    }
}