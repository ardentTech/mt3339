use core::fmt::{Display, Formatter};
use heapless::{format, String};

const SET_NMEA_UPDATE_RATE: u16 = 220;
const SET_NMEA_OUTPUT: u16 = 314;

struct NmeaPacket<const N: usize> {
    pmtk_command: PmtkCommand<N>
}
impl<const N: usize> Display for NmeaPacket<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // TODO can capacity be calculated dynamically? just set to 255?
        let cmd: String<51> = format!("{}", self.pmtk_command)?;
        let checksum = generate_checksum(cmd.as_bytes());
        write!(f, "${}*{}{}\r\n", cmd, checksum[0] as char, checksum[1] as char)
    }
}

struct PmtkCommand<const N: usize> {
    data_fields: [u16; N],
    pkt_type: u16
}
impl<const N: usize> Display for PmtkCommand<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "PMTK{}", self.pkt_type)?;
        for c in self.data_fields {
            write!(f, ",{}", c)?;
        }
        Ok(())
    }
}

// "PMTK220,1000"
#[derive(Debug, Default)]
pub struct SetNmeaUpdateRate {
    ms: u16 // TODO between 100 and 10000
}
impl Into<PmtkCommand<1>> for SetNmeaUpdateRate {
    fn into(self) -> PmtkCommand<1> {
        PmtkCommand {
            data_fields: [self.ms],
            pkt_type: SET_NMEA_UPDATE_RATE
        }
    }
}

// TODO builder pattern?
#[derive(Debug, Default)]
pub struct SetNmeaOutput {
    gll: FrequencySetting,
    rmc: FrequencySetting,
    vtg: FrequencySetting,
    gga: FrequencySetting,
    gsa: FrequencySetting,
    gsv: FrequencySetting,
    zda: FrequencySetting,
    mchn: FrequencySetting,
}
impl Into<PmtkCommand<19>> for SetNmeaOutput {
    fn into(self) -> PmtkCommand<19> {
        PmtkCommand {
            data_fields: [
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

fn generate_checksum(data: &[u8]) -> [u8; 2] {
    let mut checksum = 0;
    for c in data {
        checksum ^= *c;
    }
    let msg = format!(2; "{:X?}", checksum).unwrap();
    msg.into_bytes().into_array().unwrap()
}
//
#[derive(Debug, Default)]
enum FrequencySetting {
    #[default]
    Disabled = 0x0,
    OnePositionFix = 0x1,
    TwoPositionFixes = 0x2,
    ThreePositionFixes = 0x3,
    FourPositionFixes = 0x4,
    FivePositionFixes = 0x5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_nmea_output_display_ok() {
        let mut data = SetNmeaOutput::default();
        data.gll = FrequencySetting::OnePositionFix;
        data.rmc = FrequencySetting::OnePositionFix;
        data.vtg = FrequencySetting::OnePositionFix;
        data.gga = FrequencySetting::OnePositionFix;
        data.gsa = FrequencySetting::OnePositionFix;
        data.gsv = FrequencySetting::FivePositionFixes;
        let pmtk_command: PmtkCommand<19> = data.into();
        let res = format!(45; "{}", pmtk_command).unwrap();
        assert_eq!(res, "PMTK314,1,1,1,1,1,5,0,0,0,0,0,0,0,0,0,0,0,0,0");
    }

    #[test]
    fn set_nmea_update_rate_display_ok() {
        let data = SetNmeaUpdateRate { ms: 1000 };
        let pmtk_command: PmtkCommand<1> = data.into();
        let res = format!(15; "{}", pmtk_command).unwrap();
        assert_eq!(res, "PMTK220,1000");
    }

    #[test]
    fn nmea_packet_display_ok() {
        let mut data = SetNmeaOutput::default();
        data.gll = FrequencySetting::OnePositionFix;
        data.rmc = FrequencySetting::OnePositionFix;
        data.vtg = FrequencySetting::OnePositionFix;
        data.gga = FrequencySetting::OnePositionFix;
        data.gsa = FrequencySetting::OnePositionFix;
        data.gsv = FrequencySetting::FivePositionFixes;
        let pmtk_command: PmtkCommand<19> = data.into();
        let nmea_packet = NmeaPacket { pmtk_command };
        let res = format!(51; "{}", nmea_packet).unwrap();
        assert_eq!(res, "$PMTK314,1,1,1,1,1,5,0,0,0,0,0,0,0,0,0,0,0,0,0*2C\r\n");
    }
}