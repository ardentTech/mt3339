#![no_std]
#![no_main]

use embassy_time::Timer;
use defmt::{debug, error, info};
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, Config, Error};
use embedded_io_async::{Read, Write};
use heapless::{String, Vec};
use static_cell::StaticCell;
use mt3339::{MT3339Error, MT3339};
use mt3339::pmtk::{FrequencySetting, SetNmeaOutput, SetNmeaUpdateRate};

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let (tx_pin, rx_pin, uart) = (p.PIN_16, p.PIN_17, p.UART0);

    static TX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; 16])[..];
    static RX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; 16])[..];
    let mut config = Config::default();
    config.baudrate = 9600;
    let uart = BufferedUart::new(uart, tx_pin, rx_pin, Irqs, tx_buf, rx_buf, config);

    let mut gps = MT3339::new(uart);

    let mut cmd = SetNmeaOutput::default();
    cmd.gll = FrequencySetting::OnePositionFix;
    cmd.rmc = FrequencySetting::OnePositionFix;
    cmd.vtg = FrequencySetting::OnePositionFix;
    cmd.gga = FrequencySetting::OnePositionFix;
    cmd.gsa = FrequencySetting::OnePositionFix;
    cmd.gsv = FrequencySetting::FivePositionFixes;
    gps.send_cmd(cmd).await.ok();
    // Turn on the basic GGA and RMC info (what you typically want)
    //gps.send_cmd("PMTK314,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0".as_bytes()).await.unwrap();
    // Set update rate to once a second (1hz) which is what you typically want.
    let cmd = SetNmeaUpdateRate::new(1000).unwrap();
    gps.send_cmd(cmd).await.ok();
    //gps.send_cmd("PMTK220,1000".as_bytes()).await.unwrap();

    loop {
        gps.read_sentence().await.unwrap();
    }
}