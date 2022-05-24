use anchor_lang::prelude::*;

pub mod curve;
pub mod errors;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod token_swap {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[cfg(test)]
mod tests {
    use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

    fn write_u16(bytes: &mut [u8; 2], num: u16) {
        bytes[0] = num as u8;
        bytes[1] = (num >> 8) as u8;
    }

    #[test]
    fn test_arrayref() {
        let mut data = [0, 1, 2, 3, 4, 0, 6, 7, 8, 9];
        write_u16(array_mut_ref![data, 0, 2], 1);
        println!("1: {:?}", data); // [1, 0, 2, 3, 4, 0, 6, 7, 8, 9]

        write_u16(array_mut_ref![data,2,2], 5); // [1, 0, 5, 0, 4, 0, 6, 7, 8, 9]
        println!("2: {:?}", data);

        assert_eq!(*array_ref![data,0,4], [1,0,5,0]);
    }
}
