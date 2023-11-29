use crate::assets::des::tables::*;

pub trait DesBlock {
    fn decode_block(self) -> Self;

    fn permutation(self, permutation_table: &[u8]) -> Self;
    fn initial_permutation(self) -> Self;
    fn final_permutation(self) -> Self;

    fn round_function(self) -> Self;
    fn expansion(self) -> Self;
    fn substitution_box(self) -> Self;
    fn transposition(self) -> Self;

    fn shuffle_decode(self) -> Self;
}

impl DesBlock for [u8; 8] {
    fn decode_block(self) -> Self {
        self.initial_permutation()
            .round_function()
            .final_permutation()
    }

    fn permutation(self, permutation_table: &[u8]) -> Self {
        (0..64).fold([0u8; 8], |mut out_block, lop| {
            let prm = permutation_table[lop] - 1;
            if self[((prm >> 3) & 7) as usize] & BIT_MASK_TABLE[(prm & 7) as usize] != 0 {
                out_block[(lop >> 3) & 7] |= BIT_MASK_TABLE[lop & 7];
            }
            out_block
        })
    }

    fn initial_permutation(self) -> Self {
        self.permutation(&INITIAL_PERMUTATION_TABLE)
    }

    fn final_permutation(self) -> Self {
        self.permutation(&FINAL_PERMUTATION_TABLE)
    }

    fn round_function(self) -> Self {
        let tmp = self;
        let first_round = tmp.expansion().substitution_box().transposition();
        [
            self[0] ^ first_round[4],
            self[1] ^ first_round[5],
            self[2] ^ first_round[6],
            self[3] ^ first_round[7],
            self[4],
            self[5],
            self[6],
            self[7],
        ]
    }

    fn expansion(self) -> Self {
        [
            ((self[7] << 5) | (self[4] >> 3)) & 0x3f,
            ((self[4] << 1) | (self[5] >> 7)) & 0x3f,
            ((self[4] << 5) | (self[5] >> 3)) & 0x3f,
            ((self[5] << 1) | (self[6] >> 7)) & 0x3f,
            ((self[5] << 5) | (self[6] >> 3)) & 0x3f,
            ((self[6] << 1) | (self[7] >> 7)) & 0x3f,
            ((self[6] << 5) | (self[7] >> 3)) & 0x3f,
            ((self[7] << 1) | (self[4] >> 7)) & 0x3f,
        ]
    }

    fn substitution_box(self) -> Self {
        (0..4).fold([0u8; 8], |mut out_block, lop| {
            out_block[lop] = (SUBSTITUTION_BOX_TABLE[lop][self[lop * 2] as usize] & 0xf0)
                | (SUBSTITUTION_BOX_TABLE[lop][self[lop * 2 + 1] as usize] & 0x0f);
            out_block
        })
    }

    fn transposition(self) -> Self {
        (0..32).fold([0u8; 8], |mut out_block, lop| {
            let prm = TRANSPOSITION_TABLE[lop] - 1;
            if self[(prm >> 3) as usize] & BIT_MASK_TABLE[(prm & 7) as usize] != 0 {
                out_block[(lop >> 3) + 4] |= BIT_MASK_TABLE[lop & 7];
            }
            out_block
        })
    }

    fn shuffle_decode(self) -> Self {
        [
            self[3],
            self[4],
            self[6],
            self[0],
            self[1],
            self[2],
            self[5],
            match self[7] {
                0x00 => 0x2b,
                0x2b => 0x00,
                0x01 => 0x68,
                0x68 => 0x01,
                0x48 => 0x77,
                0x77 => 0x48,
                0x60 => 0xff,
                0xff => 0x60,
                0x6c => 0x80,
                0x80 => 0x6c,
                0xb9 => 0xc0,
                0xc0 => 0xb9,
                0xeb => 0xfe,
                0xfe => 0xeb,
                a => a,
            },
        ]
    }
}
