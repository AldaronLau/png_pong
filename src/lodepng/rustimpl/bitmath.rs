//! Reading and writing single bits and bytes from/to stream for LodePNG

#[inline(always)]
pub(super) fn read_bit_from_reversed_stream(
    bitpointer: &mut usize,
    bitstream: &[u8],
) -> u8 {
    let result = ((bitstream[(*bitpointer) >> 3] >> (7 - ((*bitpointer) & 7)))
        & 1) as u8;
    *bitpointer += 1;
    result
}

pub(super) fn set_bit_of_reversed_stream0(
    bitpointer: &mut usize,
    bitstream: &mut [u8],
    bit: u8,
) {
    /*the current bit in bitstream must be 0 for this to work*/
    if bit != 0 {
        /*earlier bit of huffman code is in a lesser significant bit of an earlier byte*/
        bitstream[(*bitpointer) >> 3] |= bit << (7 - ((*bitpointer) & 7));
    }
    *bitpointer += 1;
}

pub(super) fn set_bit_of_reversed_stream(
    bitpointer: &mut usize,
    bitstream: &mut [u8],
    bit: u8,
) {
    /*the current bit in bitstream may be 0 or 1 for this to work*/
    if bit == 0 {
        bitstream[(*bitpointer) >> 3] &=
            (!(1 << (7 - ((*bitpointer) & 7)))) as u8;
    } else {
        bitstream[(*bitpointer) >> 3] |= 1 << (7 - ((*bitpointer) & 7));
    }
    *bitpointer += 1;
}
