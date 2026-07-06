#![no_std]
pub mod pmtk;

use core::fmt::Debug;
use core::str::from_utf8;
use defmt::{debug, error, info, Format};
use embedded_io_async::{ErrorType, Read, Write};
use heapless::format;
use nmea::Nmea;
use crate::pmtk::{generate_checksum, NmeaPacket, PmtkCommand};

const PREAMBLE: u8 = 0x24; // &, 36
const DATA_FIELD_TERMINATOR: u8 = 0x2a; // *, 42
const CARRIAGE_RETURN: u8 = 0x0d; // \r, 13
const LINE_FEED: u8 = 0x0a; // \n, 10
const MAX_PACKET_LEN: usize = 255;

#[derive(Debug, Format)]
pub enum MT3339Error<UART> {
    Overflow,
    Uart(UART),
}

#[derive(Debug)]
pub struct MT3339<UART> {
    buf: [u8; MAX_PACKET_LEN],
    buf_idx: usize,
    uart: UART,
}
//impl<UART: Read + ReadReady + Write + ErrorType> MT3339<UART> {
impl<UART: Read + Write + ErrorType> MT3339<UART> {
    pub fn new(uart: UART) -> Self {
        Self { buf: [0u8; MAX_PACKET_LEN], buf_idx: 0, uart }
    }

    pub async fn send_cmd<const N: usize, C: Into<PmtkCommand<N>> + Format>(&mut self, cmd: C) -> Result<(), MT3339Error<UART::Error>> {
        debug!("MT3339.send_cmd(): {:?}", cmd);
        let pmtk_command: PmtkCommand<N> = cmd.into();
        let nmea_packet = NmeaPacket { pmtk_command };
        // TODO should probably use 255 and then trim before as bytes (or remove null bytes?)
        let s = format!(51; "{}", nmea_packet).unwrap();
        self.write(s.as_bytes()).await
    }

    pub async fn read_sentence(&mut self) -> Result<(), MT3339Error<UART::Error>> {
        let mut buf: [u8; MAX_PACKET_LEN] = [0; MAX_PACKET_LEN]; // TODO should this be smaller than 255?

        match self.uart.read(&mut buf).await.map_err(MT3339Error::Uart) {
            Ok(len) => {
                if len > 0 {
                    buf[..len].iter().for_each(|b| {
                        self.buf[self.buf_idx] = *b;
                        if self.buf_idx + 1 == MAX_PACKET_LEN {
                            error!("overflow error");
                            //return Err(MT3339Error::Overflow) // TODO
                        }
                        self.buf_idx += 1;

                        if *b == LINE_FEED {
                            // TOOD i don't think this handles PMTKCHN sentences
                            let mut nmea = Nmea::default();
                            nmea.parse(from_utf8(&self.buf[..self.buf_idx]).unwrap()).unwrap();
                            info!("{}", nmea);
                            self.buf = [0; 255];
                            self.buf_idx = 0;
                        }
                    });
                }
            }
            Err(_) => {}
        }
        Ok(())
    }

    async fn write(&mut self, data: &[u8]) -> Result<(), MT3339Error<UART::Error>> {
        self.uart.write(data).await.map_err(MT3339Error::Uart)?;
        Ok(())
    }
}