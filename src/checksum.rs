// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(clippy::unreadable_literal)] // Crazy lookup table, don't care

use crate::decode::{Error as DecoderError, Result as DecoderResult};
use std::io::Read;

/* CRC polynomial: 0xedb88320 */
const CRC32_TABLE: [u32; 256] = [
    0, 1996959894, 3993919788, 2567524794, 124634137, 1886057615, 3915621685,
    2657392035, 249268274, 2044508324, 3772115230, 2547177864, 162941995,
    2125561021, 3887607047, 2428444049, 498536548, 1789927666, 4089016648,
    2227061214, 450548861, 1843258603, 4107580753, 2211677639, 325883990,
    1684777152, 4251122042, 2321926636, 335633487, 1661365465, 4195302755,
    2366115317, 997073096, 1281953886, 3579855332, 2724688242, 1006888145,
    1258607687, 3524101629, 2768942443, 901097722, 1119000684, 3686517206,
    2898065728, 853044451, 1172266101, 3705015759, 2882616665, 651767980,
    1373503546, 3369554304, 3218104598, 565507253, 1454621731, 3485111705,
    3099436303, 671266974, 1594198024, 3322730930, 2970347812, 795835527,
    1483230225, 3244367275, 3060149565, 1994146192, 31158534, 2563907772,
    4023717930, 1907459465, 112637215, 2680153253, 3904427059, 2013776290,
    251722036, 2517215374, 3775830040, 2137656763, 141376813, 2439277719,
    3865271297, 1802195444, 476864866, 2238001368, 4066508878, 1812370925,
    453092731, 2181625025, 4111451223, 1706088902, 314042704, 2344532202,
    4240017532, 1658658271, 366619977, 2362670323, 4224994405, 1303535960,
    984961486, 2747007092, 3569037538, 1256170817, 1037604311, 2765210733,
    3554079995, 1131014506, 879679996, 2909243462, 3663771856, 1141124467,
    855842277, 2852801631, 3708648649, 1342533948, 654459306, 3188396048,
    3373015174, 1466479909, 544179635, 3110523913, 3462522015, 1591671054,
    702138776, 2966460450, 3352799412, 1504918807, 783551873, 3082640443,
    3233442989, 3988292384, 2596254646, 62317068, 1957810842, 3939845945,
    2647816111, 81470997, 1943803523, 3814918930, 2489596804, 225274430,
    2053790376, 3826175755, 2466906013, 167816743, 2097651377, 4027552580,
    2265490386, 503444072, 1762050814, 4150417245, 2154129355, 426522225,
    1852507879, 4275313526, 2312317920, 282753626, 1742555852, 4189708143,
    2394877945, 397917763, 1622183637, 3604390888, 2714866558, 953729732,
    1340076626, 3518719985, 2797360999, 1068828381, 1219638859, 3624741850,
    2936675148, 906185462, 1090812512, 3747672003, 2825379669, 829329135,
    1181335161, 3412177804, 3160834842, 628085408, 1382605366, 3423369109,
    3138078467, 570562233, 1426400815, 3317316542, 2998733608, 733239954,
    1555261956, 3268935591, 3050360625, 752459403, 1541320221, 2607071920,
    3965973030, 1969922972, 40735498, 2617837225, 3943577151, 1913087877,
    83908371, 2512341634, 3803740692, 2075208622, 213261112, 2463272603,
    3855990285, 2094854071, 198958881, 2262029012, 4057260610, 1759359992,
    534414190, 2176718541, 4139329115, 1873836001, 414664567, 2282248934,
    4279200368, 1711684554, 285281116, 2405801727, 4167216745, 1634467795,
    376229701, 2685067896, 3608007406, 1308918612, 956543938, 2808555105,
    3495958263, 1231636301, 1047427035, 2932959818, 3654703836, 1088359270,
    936918000, 2847714899, 3736837829, 1202900863, 817233897, 3183342108,
    3401237130, 1404277552, 615818150, 3134207493, 3453421203, 1423857449,
    601450431, 3009837614, 3294710456, 1567103746, 711928724, 3020668471,
    3272380065, 1510334235, 755167117,
];

const CRC32_INIT: u32 = 4294967295;

/// CRC32 Checksum
pub(super) struct Crc32(u32);

impl Crc32 {
    /// Create a blank CRC32 Checksum
    pub(super) fn new() -> Self {
        Crc32(CRC32_INIT)
    }

    /// Add a byte to the checksum
    pub(super) fn add(&mut self, byte: u8) {
        self.0 = CRC32_TABLE[((self.0 ^ byte as u32) & 255) as usize]
            ^ (self.0 >> 8);
    }

    /// Calculate the CRC based on the current state
    pub(super) fn into_u32(self) -> u32 {
        self.0 ^ 4294967295
    }
}

/// CRC32 Checksum
pub(super) struct CrcDecoder<'a, R: Read> {
    // Intermediate CRC
    icrc: u32,
    // Reader
    reader: &'a mut R,
}

impl<'a, R: Read> CrcDecoder<'a, R> {
    /// Create a new CRC decoder.
    #[inline(always)]
    pub(super) fn new(reader: &'a mut R, name: [u8; 4]) -> Self {
        let mut chunk = CrcDecoder {
            icrc: CRC32_INIT,
            reader,
        };
        chunk.add_bytes(&name);
        chunk
    }

    /// Read big-endian u32 from reader.
    #[inline(always)]
    pub(super) fn u32(&mut self) -> DecoderResult<u32> {
        let mut buffer = [0; 4];
        self.reader
            .read_exact(&mut buffer)
            .map_err(DecoderError::from)?;
        self.add_bytes(&buffer);
        Ok(u32::from_be_bytes(buffer))
    }

    /// Read big-endian u16 from reader.
    #[inline(always)]
    pub(super) fn u16(&mut self) -> DecoderResult<u16> {
        let mut buffer = [0; 2];
        self.reader
            .read_exact(&mut buffer)
            .map_err(DecoderError::from)?;
        self.add_bytes(&buffer);
        Ok(u16::from_be_bytes(buffer))
    }

    /// Read u8 from reader.
    #[inline(always)]
    pub(super) fn u8(&mut self) -> DecoderResult<u8> {
        let mut buffer = [0; 1];
        self.reader
            .read_exact(&mut buffer)
            .map_err(DecoderError::from)?;
        self.add_bytes(&buffer);
        Ok(buffer[0])
    }

    /// Read Null-Terminated String from reader.
    #[inline(always)]
    pub(super) fn utf8z(&mut self) -> DecoderResult<String> {
        let mut bytes = [0u8; 4];
        let mut index = 0;
        let mut out = String::new();
        for byte in self.reader.bytes() {
            let byte = byte.map_err(DecoderError::from)?;
            self.icrc = CRC32_TABLE[((self.icrc ^ byte as u32) & 255) as usize]
                ^ (self.icrc >> 8);
            match byte {
                0 => break,
                c => {
                    bytes[index] = c;
                    index += 1;
                    match std::str::from_utf8(&bytes[0..index]) {
                        Ok(c) => {
                            out.push_str(c);
                            index = 0;
                        }
                        Err(e) => {
                            if e.error_len().is_some() {
                                out.push(std::char::REPLACEMENT_CHARACTER);
                                index = 0;
                            }
                        }
                    }
                }
            }
        }
        Ok(out)
    }

    /// Read u8 from reader if it exists.
    #[inline(always)]
    pub(super) fn maybe_u8(&mut self) -> DecoderResult<Option<u8>> {
        let mut buffer = [0; 1];
        let len = self.reader.read(&mut buffer).map_err(DecoderError::from)?;
        if len == 0 {
            Ok(None)
        } else {
            self.add_bytes(&buffer);
            Ok(Some(buffer[0]))
        }
    }

    /// Read bytes until EOF.
    #[inline(always)]
    pub(super) fn vec_eof(&mut self) -> DecoderResult<Vec<u8>> {
        let mut bytes = Vec::new();
        self.reader
            .read_to_end(&mut bytes)
            .map_err(DecoderError::from)?;
        self.add_bytes(&bytes);
        Ok(bytes)
    }

    /// Check to see if the reader has ended.
    #[inline(always)]
    pub(super) fn end(self) -> DecoderResult<u32> {
        let mut buffer = [0; 1];
        self.reader
            .read_exact(&mut buffer)
            .err()
            .ok_or(DecoderError::NoEnd)?;
        Ok(self.icrc ^ CRC32_INIT)
    }

    #[inline(always)]
    fn add_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes.iter().cloned() {
            self.icrc = CRC32_TABLE[((self.icrc ^ byte as u32) & 255) as usize]
                ^ (self.icrc >> 8);
        }
    }
}