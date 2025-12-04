use core::convert::TryFrom;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dex {
    UniswapV3 = 0,
    PancakeSwapV3 = 1,
    SlipStream = 2,
    UniswapV4 = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DexFromU8Error;

impl From<Dex> for u8 {
    #[inline]
    fn from(dex: Dex) -> Self {
        dex as u8
    }
}

impl TryFrom<u8> for Dex {
    type Error = DexFromU8Error;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Dex::UniswapV3),
            1 => Ok(Dex::PancakeSwapV3),
            2 => Ok(Dex::SlipStream),
            3 => Ok(Dex::UniswapV4),
            _ => Err(DexFromU8Error),
        }
    }
}
