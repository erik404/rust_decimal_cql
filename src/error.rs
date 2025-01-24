use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DecimalCqlError {
    MismatchedType(String),
    FrameHasNoSlice(),
    ByteArrayTooShort(usize),
    InvalidMantissaConversion(),
}

impl fmt::Display for DecimalCqlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecimalCqlError::MismatchedType(msg) => write!(f, "Type Mismatch: {}", msg),
            DecimalCqlError::FrameHasNoSlice() => write!(f, "Frame has no slice"),
            DecimalCqlError::ByteArrayTooShort(len) => {
                write!(
                    f,
                    "Byte array too short. Expected at least 4 bytes, got: {}",
                    len
                )
            }
            DecimalCqlError::InvalidMantissaConversion() => {
                write!(f, "Could not convert array to i128")
            }
        }
    }
}

impl Error for DecimalCqlError {}