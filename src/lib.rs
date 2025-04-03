mod error;

use crate::error::DecimalCqlError;
use rust_decimal::Decimal;
use scylla::_macro_internal::{
    CellWriter, ColumnType, DeserializeValue, SerializeValue, WrittenCellProof,
};
use scylla::cluster::metadata::NativeType;
use scylla::deserialize::{DeserializationError, FrameSlice, TypeCheckError};
use scylla::serialize::SerializationError;
use scylla::value::CqlDecimal;
use std::ops::Deref;

const SCALE_BYTES: usize = 4;
const PADDING_BYTES: usize = 16;

#[derive(Debug, Clone, PartialEq)]
pub struct DecimalCql(Decimal);

impl From<Decimal> for DecimalCql {
    fn from(decimal: Decimal) -> Self {
        Self(decimal)
    }
}

/// Transparent access to the inner `Decimal` value within `DecimalCql` by Dereferencing.
///
/// # Examples
///
/// ```rust
/// use rust_decimal::Decimal;
/// use rust_decimal_cql::DecimalCql;
///
/// let decimal: Decimal = Decimal::new(12345, 2); // Represents 123.45
/// let wrapper: DecimalCql = decimal.into();
///
/// // Using Decimal operations directly on the wrapper
/// let result = *wrapper + Decimal::new(100, 0); // Adds 100.00
/// assert_eq!(result, Decimal::new(22345, 2)); // Result is 223.45
/// ```

impl Deref for DecimalCql {
    type Target = Decimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Implements `SerializeValue` for `DecimalCql` to serialize `Decimal`
/// into ScyllaDB's `CqlDecimal` format.
///
/// # Parameters
/// - `_typ`: expected database column type (not used).
/// - `writer`: `CellWriter` for serializing the value.
///
/// # Returns
/// - `Ok(WrittenCellProof)` on successful serialization.
/// - `Err(SerializationError)` if serialization fails.

impl SerializeValue for DecimalCql {
    fn serialize<'b>(
        &self,
        _typ: &ColumnType,
        writer: CellWriter<'b>,
    ) -> Result<WrittenCellProof<'b>, SerializationError> {
        let mantissa_bytes = self.0.mantissa().to_be_bytes();
        let cql_decimal: CqlDecimal = CqlDecimal::from_signed_be_bytes_and_exponent(
            mantissa_bytes.to_vec(),
            self.0.scale() as i32,
        );
        cql_decimal.serialize(_typ, writer)
    }
}

/// Implements deserialization for `DecimalCql` to deserialize a `CqlDecimal` to a
/// `DecimalCql` with an inner `Decimal`
///
/// - `type_check`: Verifies that the column type matches `Decimal`, returning a `TypeCheckError` if not.
/// - `deserialize`: Converts the serialized `CqlDecimal` data into a `DecimalCql` with an inner `Decimal`
///
/// # Errors
/// - Returns `TypeCheckError` if the column type is not ` ColumnType::Decimal`.
/// - Returns `DeserializationError` if the frame is empty or the data cannot be parsed.

impl<'frame, 'metadata> DeserializeValue<'frame, 'metadata> for DecimalCql {
    fn type_check(typ: &ColumnType) -> Result<(), TypeCheckError> {
        if !matches!(typ, ColumnType::Native(NativeType::Decimal)) {
            let typ_info: String = format!("Expected {:?}, got {:?}", NativeType::Decimal, typ);
            return Err(TypeCheckError::new(DecimalCqlError::MismatchedType(
                typ_info,
            )));
        }
        Ok(())
    }

    fn deserialize(
        _typ: &'metadata ColumnType<'metadata>,
        frame: Option<FrameSlice<'frame>>,
    ) -> Result<DecimalCql, DeserializationError> {
        let fs: FrameSlice =
            frame.ok_or_else(|| DeserializationError::new(DecimalCqlError::FrameHasNoSlice()))?;
        let (scale, mantissa): (u32, i128) = extract_scale_and_mantissa_from_slice(fs.as_slice())
            .map_err(|e| DeserializationError::new(e))?;
        let decimal: Decimal = Decimal::from_i128_with_scale(mantissa, scale);
        Ok(DecimalCql(decimal))
    }
}

/// The first 4 bytes are the scale (`u32`), and the remaining bytes as
/// the mantissa (`i128`). Pads the mantissa to 16 bytes if needed.
///
/// # Arguments
/// - `bytes`: A byte slice derived from a `FrameSlice`.
///
/// # Returns
/// - `Ok((u32, i128))`: The scale and mantissa.
/// - `Err(DecimalCqlError)`

fn extract_scale_and_mantissa_from_slice(bytes: &[u8]) -> Result<(u32, i128), DecimalCqlError> {
    if bytes.len() < SCALE_BYTES {
        return Err(DecimalCqlError::ByteArrayTooShort(bytes.len()));
    }
    let scale: u32 = u32::from_be_bytes(
        bytes[0..SCALE_BYTES]
            .try_into()
            .expect("Is safe because bytes length have been checked"),
    );
    let mantissa_bytes: &[u8] = &bytes[SCALE_BYTES..];
    let mantissa: i128 = if mantissa_bytes.len() >= PADDING_BYTES {
        // If mantissa_bytes has 16 or more bytes, truncate to the first 16 bytes
        i128::from_be_bytes(
            mantissa_bytes[0..PADDING_BYTES]
                .try_into()
                .map_err(|_| DecimalCqlError::InvalidMantissaConversion())?,
        )
    } else {
        // Otherwise, pad the mantissa_bytes to 16 bytes
        let padding_length: usize = PADDING_BYTES - mantissa_bytes.len();
        let mut padded_bytes: Vec<u8> = vec![0; padding_length];
        padded_bytes.extend_from_slice(mantissa_bytes);
        i128::from_be_bytes(
            padded_bytes
                .try_into()
                .map_err(|_| DecimalCqlError::InvalidMantissaConversion())?,
        )
    };

    Ok((scale, mantissa))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_cql_deref() {
        let decimal = Decimal::new(12345, 2);
        let wrapper: DecimalCql = decimal.into();
        assert_eq!(*wrapper, Decimal::new(12345, 2));
        assert_eq!(*wrapper + Decimal::new(100, 0), Decimal::new(22345, 2));
    }

    #[test]
    fn test_decimal_cql_serialize() {
        let decimal = Decimal::new(12345, 2);
        let expected_bytes = [
            0, 0, 0, 20, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 48, 57,
        ];
        let wrapper: DecimalCql = decimal.into();
        let mut buffer = Vec::new();
        let writer = CellWriter::new(&mut buffer);
        wrapper
            .serialize(&ColumnType::Native(NativeType::Decimal), writer)
            .unwrap();
        assert_eq!(
            buffer, expected_bytes,
            "Buffer should match expected_bytes exactly"
        );
    }

    #[test]
    fn test_decimal_cql_no_frame() {
        let result = DecimalCql::deserialize(&ColumnType::Native(NativeType::Decimal), None);
        assert!(
            result.is_err(),
            "Deserialization should fail if frame slice is None"
        );
    }

    #[test]
    fn test_extract_scale_and_mantissa_from_slice_success() {
        let bytes = &[0, 0, 0, 2, 130];
        let result = extract_scale_and_mantissa_from_slice(bytes).unwrap();
        assert_eq!(result, (2, 130));
    }

    #[test]
    fn test_extract_scale_and_mantissa_from_slice_short_bytes() {
        let bytes = &[0, 0, 0];
        let result = extract_scale_and_mantissa_from_slice(bytes);
        assert!(result.is_err());
        if let Err(DecimalCqlError::ByteArrayTooShort(len)) = result {
            assert_eq!(len, 3);
        } else {
            panic!("Expected ByteArrayTooShort error");
        }
    }

    #[test]
    fn test_extract_scale_and_mantissa_from_slice_only_scale() {
        let bytes = &[0, 0, 0, 2];
        let result = extract_scale_and_mantissa_from_slice(bytes).unwrap();
        assert_eq!(
            result,
            (2, 0),
            "Mantissa should default to 0 if no bytes remain"
        );
    }

    #[test]
    fn test_extract_scale_and_mantissa_from_slice_zero_scale() {
        let bytes = &[0, 0, 0, 0, 0x01];
        let result = extract_scale_and_mantissa_from_slice(bytes).unwrap();
        assert_eq!(
            result,
            (0, 1),
            "Failed test: expected scale 0 and mantissa 1, got {:?}",
            result
        );
    }

    #[test]
    fn test_extract_scale_and_mantissa_from_slice_exact_padding() {
        let bytes = &[
            0, 0, 0, 2, // Scale (4 bytes)
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
            0x16, 0x17,
        ]; // Exactly 16 bytes
        let result = extract_scale_and_mantissa_from_slice(bytes).unwrap();
        assert_eq!(result.0, 2);
        assert_eq!(
            result.1,
            i128::from_be_bytes([
                0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
                0x16, 0x17
            ])
        );
    }

    #[test]
    fn test_extract_scale_and_mantissa_from_slice_large_mantissa() {
        let bytes = &[
            0, 0, 0, 2, // Scale (4 bytes)
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
            0x16, 0x17, 0x18, 0x19, 0x1A,
        ]; // More than 16 bytes
        let result = extract_scale_and_mantissa_from_slice(bytes).unwrap();
        assert_eq!(result.0, 2);
        assert_eq!(
            result.1,
            i128::from_be_bytes([
                0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
                0x16, 0x17
            ])
        );
    }

    #[test]
    fn test_extract_scale_and_mantissa_from_slice_partial_padding() {
        let bytes = &[
            0, 0, 0, 2, // Scale (4 bytes)
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF,
        ]; // Only 8 bytes of mantissa
        let result = extract_scale_and_mantissa_from_slice(bytes).unwrap();
        assert_eq!(result.0, 2);
        assert_eq!(
            result.1,
            i128::from_be_bytes([
                0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF
            ])
        );
    }

    #[test]
    fn test_extract_scale_and_mantissa_from_slice_empty_mantissa() {
        let bytes = &[0, 0, 0, 2]; // Only scale, no mantissa
        let result = extract_scale_and_mantissa_from_slice(bytes).unwrap();
        assert_eq!(result.0, 2);
        assert_eq!(result.1, 0);
    }
}
