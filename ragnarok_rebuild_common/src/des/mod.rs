mod block;
mod tables;

use std::{io::Error, ops::Shr};

use crate::des::block::DesBlock;

pub fn decode(
    buffer: &[u8],
    aligned_length: usize,
    unaligned_length: usize,
    decode_header_only: bool,
) -> Result<Vec<u8>, Error> {
    let header_len = aligned_length.shr(3);

    let cycle = {
        let digits = (unaligned_length as f64).log10() as usize + 1;
        match digits {
            0..=2 => 3,
            3..=4 => digits + 1,
            5..=6 => digits + 9,
            _ => digits + 15,
        }
    };

    let mut j = 0;
    // Buffer should always be 8-bytes aligned, and therefore, chucks should always return slices that can be turned to [u8; 8]
    #[allow(clippy::expect_used)]
    Ok(buffer
        .chunks(8)
        .map(|block| block.try_into().expect("Chunk is 8 bytes long."))
        .enumerate()
        .flat_map(|(i, block): (usize, [u8; 8])| {
            let block_is_part_of_header = i < 20 && i < header_len;
            match (block_is_part_of_header, decode_header_only) {
                (true, _) => block.decode_block(),
                (false, false) => match i % cycle {
                    0 => block.decode_block(),
                    _ => {
                        let data = if j == 7 {
                            j = 0;
                            block.shuffle_decode()
                        } else {
                            block
                        };
                        j += 1;
                        data
                    }
                },
                (false, true) => block,
            }
        })
        .collect())
}
