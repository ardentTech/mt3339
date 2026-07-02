//     const CR_LF: [u8; 2] = [0x0d, 0x0a]; // \r\n
//     const PREAMBLE: u8 = 0x24; // $
//     const TALKER_ID: [u8; 4] = [0x50, 0x4d, 0x54, 0x4b]; // PMTK
//     const TERMINATOR: u8 = 0x2a; // *
//
//     fn checksum(&self, data: &[u8]) -> u16 {
//         let mut checksum = 016;
//         for c in data {
//             checksum ^= *c as u16;
//         }
//         checksum
//     }
//
//     fn encode(&self) -> [u8; LEN] {
//         // $PMTK314,1,1,1,1,1,5,0,0,0,0,0,0,0,0,0,0,0,0,0*2C<CR><LF>
//         let mut res = [0u8; LEN];
//         res[0] = Self::PREAMBLE;
//         for (i, v) in Self::TALKER_ID.iter().enumerate() {
//             res[i + 1usize] = *v;
//         }
//         for (i, v) in self.pkt_type().iter().enumerate() {
//             res[i + 5usize] = *v;
//         }
//         for (i, v) in self.data_fields().iter().enumerate() {
//             res[i + 8usize] = 0x2c; // comma
//             res[i + 9usize] = *v;
//         }
//         res[LEN - 5] = Self::TERMINATOR;
//         let checksum = self.checksum(&self.data_fields());
//         res[LEN - 4] = (checksum >> 8) as u8;
//         res[LEN - 3] = (checksum & 0xff) as u8;
//         res[LEN - 2] = 0x0d;
//         res[LEN - 1] = 0x0a;
//         res
//     }