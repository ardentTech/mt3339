#![no_std]

use core::str::{from_utf8, Utf8Error};
use defmt::{debug, error, info, Format};
use embedded_io_async::{ErrorType, Read, Write};
use heapless::{format, String};

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

    pub async fn send_cmd(&mut self, cmd: &[u8]) -> Result<(), MT3339Error<UART::Error>> {
        debug!("MT3339.send_cmd(): {:?}", cmd);
        self.write(&[PREAMBLE]).await?;
        self.write(cmd).await?;
        self.write(&[DATA_FIELD_TERMINATOR]).await?;
        self.write(&checksum(cmd)).await?;
        self.write(&[CARRIAGE_RETURN, LINE_FEED]).await?;
        Ok(())
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
                            match from_utf8(&self.buf[..self.buf_idx]) {
                                Ok(msg) => {
                                    let csum = &msg[self.buf_idx - 4..self.buf_idx - 2];
                                    let data_fields = &msg[1..self.buf_idx - 5];
                                    if checksum(data_fields.as_bytes()) != csum.as_bytes() {
                                        error!("checksum error");
                                    }
                                    info!("msg: {:?}", msg);
                                    self.buf = [0; 255];
                                    self.buf_idx = 0;
                                }
                                Err(_) => {
                                    error!("failed to convert buffer to utf8 :(")
                                }
                            }
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


pub(crate) fn checksum(data: &[u8]) -> [u8; 2] {
    let mut checksum = 0;
    for c in data {
        checksum ^= *c;
    }
    // TODO find way to avoid String alloc?
    let msg: String<2> = format!(2; "{:X?}", checksum).unwrap();
    msg.into_bytes().into_array().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_nmea_out() {
        assert_eq!(checksum("PMTK314,1,1,1,1,1,5,0,0,0,0,0,0,0,0,0,0,0,0,0".as_bytes()), [0x31, 0x39]);
    }

    #[test]
    fn checksum_pmtk_ack() {
        assert_eq!(checksum("PMTK001,604,3".as_bytes()), [0x33, 0x32]);
    }
}