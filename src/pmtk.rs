use core::fmt::Write;
use defmt::Format;
use heapless::{format, String};
use crate::pmtk::PmtkError::InvalidInput;

const PACKET_LEN: usize = 255;
const SET_NMEA_OUTPUT: u16 = 314;
const SET_NMEA_UPDATE_RATE: u16 = 220;
const SET_NMEA_UPDATE_RATE_MIN: u16 = 100;
const SET_NMEA_UPDATE_RATE_MAX: u16 = 10000;

#[derive(Debug)]
pub enum PmtkError {
    InvalidInput
}

pub struct PmtkCommand<const N: usize> {
    pkt_type: u16, // impl takes care of str conversion
    data_field: [u16; N] // (255 - 13) / 2
}
impl<const N: usize> PmtkCommand<N> {
    pub(crate) fn serialize(&self) -> String<PACKET_LEN> {
        let payload = self.serialize_payload();
        let checksum = generate_checksum(payload.as_bytes());
        format!(PACKET_LEN; "${}*{:X?}\r\n", payload, checksum).unwrap()
    }

    fn serialize_payload(&self) -> String<247> {
        let mut s = String::<247>::new();
        write!(s, "PMTK{}", self.pkt_type).unwrap();
        for c in self.data_field {
            write!(s, ",{}", c).unwrap();
        }
        s
    }
}

#[derive(Debug, Default, Format)]
pub enum FrequencySetting {
    #[default]
    Disabled = 0x0,
    OnePositionFix = 0x1,
    TwoPositionFixes = 0x2,
    ThreePositionFixes = 0x3,
    FourPositionFixes = 0x4,
    FivePositionFixes = 0x5,
}

// TODO builder pattern?
#[derive(Debug, Default, Format)]
pub struct SetNmeaOutput {
    pub gll: FrequencySetting,
    pub rmc: FrequencySetting,
    pub vtg: FrequencySetting,
    pub gga: FrequencySetting,
    pub gsa: FrequencySetting,
    pub gsv: FrequencySetting,
    pub zda: FrequencySetting,
    pub mchn: FrequencySetting,
}
impl Into<PmtkCommand<19>> for SetNmeaOutput {
    fn into(self) -> PmtkCommand<19> {
        PmtkCommand {
            data_field: [
                self.gll as u16,
                self.rmc as u16,
                self.vtg as u16,
                self.gga as u16,
                self.gsa as u16,
                self.gsv as u16,
                0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                self.zda as u16,
                self.mchn as u16,
            ],
            pkt_type: SET_NMEA_OUTPUT
        }
    }
}
// TODO fn new

#[derive(Debug, Default, Format)]
pub struct SetNmeaUpdateRate(pub(crate) u16);
impl SetNmeaUpdateRate {
    pub fn new(ms: u16) -> Result<Self, PmtkError> {
        if !Self::validate(ms) {
            return Err(InvalidInput)
        }
        Ok(Self(ms))
    }

    fn validate(ms: u16) -> bool {
        SET_NMEA_UPDATE_RATE_MIN < ms && ms < SET_NMEA_UPDATE_RATE_MAX
    }
}
impl Into<PmtkCommand<1>> for SetNmeaUpdateRate {
    fn into(self) -> PmtkCommand<1> {
        PmtkCommand {
            data_field: [self.0],
            pkt_type: SET_NMEA_UPDATE_RATE
        }
    }
}

pub(crate) fn generate_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0, |acc, &x| acc ^ x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_checksum_ok() {
        assert_eq!(generate_checksum("PMTK314,1,1,1,1,1,5,0,0,0,0,0,0,0,0,0,0,0,0,0".as_bytes()), 0x2c);
    }

    #[test]
    fn pmtk_command_serialize_ok() {
        let cmd = PmtkCommand { pkt_type: 220, data_field: [1000] };
        assert_eq!(cmd.serialize(), "$PMTK220,1000*1F\r\n");
    }

    #[test]
    fn command_serialize_ok() {
        let cmd: PmtkCommand<1> = SetNmeaUpdateRate(1000).into();
        assert_eq!(cmd.serialize(), "$PMTK220,1000*1F\r\n");
    }
}