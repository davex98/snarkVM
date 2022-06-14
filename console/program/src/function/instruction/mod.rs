// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

mod operand;
pub(crate) use operand::*;

mod operation;
use operation::*;

use crate::{
    program::{RegisterType, Stack},
    Register,
    Sanitizer,
};
use snarkvm_console_network::{
    prelude::{
        alt,
        bail,
        ensure,
        error,
        fmt,
        map,
        tag,
        Debug,
        Display,
        Error,
        Formatter,
        FromBytes,
        FromStr,
        IoResult,
        Parser,
        ParserResult,
        Read,
        Result,
        ToBytes,
        Write,
    },
    Network,
};

/// Creates a match statement that applies the given operation for each instruction.
///
/// ## Example
/// ```ignore
/// instruction!(self, |instruction| write!(f, "{} {};", self.opcode(), instruction))
/// ```
/// The above example will print the opcode and the instruction to the given stream.
/// ```ignore
///     match self {
///         Self::Add(instruction) => write!(f, "{} {};", self.opcode(), instruction),
///         Self::Sub(instruction) => write!(f, "{} {};", self.opcode(), instruction),
///         Self::Mul(instruction) => write!(f, "{} {};", self.opcode(), instruction),
///         Self::Div(instruction) => write!(f, "{} {};", self.opcode(), instruction),
///     }
/// )
/// ```
#[macro_export]
macro_rules! instruction {
    // A variant **with** curly braces:
    // i.e. `instruction!(self, |instruction| { operation(instruction) })`.
    ($object:expr, |$input:ident| $operation:block) => {{
        instruction!(instruction, $object, |$input| $operation)
    }};
    // A variant **without** curly braces:
    // i.e. `instruction!(self, |instruction| operation(instruction))`.
    ($object:expr, |$input:ident| $operation:expr) => {{
        instruction!(instruction, $object, |$input| { $operation })
    }};
    // A variant **with** curly braces:
    // i.e. `instruction!(custom_macro, self, |instruction| { operation(instruction) })`.
    ($macro_:ident, $object:expr, |$input:ident| $operation:block) => {
        $macro_!{$object, |$input| $operation, {
            // Abs,
            // AbsWrapped,
            Add,
            AddWrapped,
            // And,
            // CommitBHP256,
            // CommitBHP512,
            // CommitBHP768,
            // CommitBHP1024,
            // CommitPed64,
            // CommitPed128,
            Div,
            DivWrapped,
            // Double,
            // Equal,
            // GreaterThan,
            // GreaterThanOrEqual,
            // HashBHP256,
            // HashBHP512,
            // HashBHP768,
            // HashBHP1024,
            // HashPed64,
            // HashPed128,
            // HashPsd2,
            // HashPsd4,
            // HashPsd8,
            // Inv,
            // LessThan,
            // LessThanOrEqual,
            Mul,
            MulWrapped,
            // Nand,
            // Neg,
            // Nor,
            // Not,
            // NotEqual,
            // Or,
            // Pow,
            // PowWrapped,
            // PRFPsd2,
            // PRFPsd4,
            // PRFPsd8,
            // Shl,
            // ShlWrapped,
            // Shr,
            // ShrWrapped,
            // Square,
            Sub,
            SubWrapped,
            // Ternary,
            // Xor,
        }}
    };
    // A variant **without** curly braces:
    // i.e. `instruction!(custom_macro, self, |instruction| operation(instruction))`.
    ($macro_:ident, $object:expr, |$input:ident| $operation:expr) => {{
        instruction!($macro_, $object, |$input| { $operation })
    }};
    // A variant invoking a macro internally:
    // i.e. `instruction!(instruction_to_bytes_le!(self, writer))`.
    ($macro_:ident!($object:expr, $input:ident)) => {{
        instruction!($macro_, $object, |$input| {})
    }};

    ////////////////////
    // Private Macros //
    ////////////////////

    // A static variant **with** curly braces:
    // i.e. `instruction!(self, |InstructionMember| { InstructionMember::opcode() })`.
    ($object:expr, |InstructionMember| $operation:block, { $( $variant:ident, )+ }) => {{
        // Build the match cases.
        match $object {
            $(
                Self::$variant(..) => {{
                    // Set the variant to be called `InstructionMember`.
                    type InstructionMember<N> = $variant<N>;
                    // Perform the operation.
                    $operation
                }}
            ),+
        }
    }};
    // A static variant **without** curly braces:
    // i.e. `instruction!(self, |InstructionMember| InstructionMember::opcode())`.
    ($object:expr, |InstructionMember| $operation:expr, { $( $variant:ident, )+ }) => {{
        instruction!($object, |InstructionMember| { $operation }, { $( $variant, )+ })
    }};
    // A non-static variant **with** curly braces:
    // i.e. `instruction!(self, |instruction| { operation(instruction) })`.
    ($object:expr, |$instruction:ident| $operation:block, { $( $variant:ident, )+ }) => {{
        // Build the match cases.
        match $object {
            $( Self::$variant($instruction) => { $operation } ),+
        }
    }};
    // A non-static variant **without** curly braces:
    // i.e. `instruction!(self, |instruction| operation(instruction))`.
    ($object:expr, |$instruction:ident| $operation:expr, { $( $variant:ident, )+ }) => {{
        instruction!($object, |$instruction| { $operation }, { $( $variant, )+ })
    }};
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Instruction<N: Network> {
    // /// Compute the absolute value of `first`, checking for overflow, and storing the outcome in `destination`.
    // Abs(Abs<N>),
    // /// Compute the absolute value of `first`, wrapping around at the boundary of the type, and storing the outcome in `destination`.
    // AbsWrapped(AbsWrapped<N>),
    /// Adds `first` with `second`, storing the outcome in `destination`.
    Add(Add<N>),
    /// Adds `first` with `second`, wrapping around at the boundary of the type, and storing the outcome in `destination`.
    AddWrapped(AddWrapped<N>),
    // /// Performs a bitwise AND operation on `first` and `second`, storing the outcome in `destination`.
    // And(And<N>),
    // /// Performs a BHP commitment taking a 256-bit value as input.
    // CommitBHP256(CommitBHP256<N>),
    // /// Performs a BHP commitment taking a 512-bit value as input.
    // CommitBHP512(CommitBHP512<N>),
    // /// Performs a BHP commitment taking a 768-bit value as input.
    // CommitBHP768(CommitBHP768<N>),
    // /// Performs a BHP commitment taking a 1024-bit value as input.
    // CommitBHP1024(CommitBHP1024<N>),
    // /// Performs a Pedersen commitment taking a 64-bit value as input.
    // CommitPed64(CommitPed64<N>),
    // /// Performs a Pedersen commitment taking a 128-bit value as input.
    // CommitPed128(CommitPed128<N>),
    /// Divides `first` by `second`, storing the outcome in `destination`.
    Div(Div<N>),
    /// Divides `first` by `second`, wrapping around at the boundary of the type, and storing the outcome in `destination`.
    DivWrapped(DivWrapped<N>),
    // /// Doubles `first`, storing the outcome in `destination`.
    // Double(Double<N>),
    // /// Checks if `first` is equal to `second`, storing the outcome in `destination`.
    // Equal(Equal<N>),
    // /// Checks if `first` is greater than `second`, storing the result in `destination`.
    // GreaterThan(GreaterThan<N>),
    // /// Checks if `first` is greater than or equal to `second`, storing the result in `destination`.
    // GreaterThanOrEqual(GreaterThanOrEqual<N>),
    // /// Performs a BHP hash taking a 256-bit value as input.
    // HashBHP256(HashBHP256<N>),
    // /// Performs a BHP hash taking a 512-bit value as input.
    // HashBHP512(HashBHP512<N>),
    // /// Performs a BHP hash taking a 768-bit value as input.
    // HashBHP768(HashBHP768<N>),
    // /// Performs a BHP hash taking a 1024-bit value as input.
    // HashBHP1024(HashBHP1024<N>),
    // /// Performs a Pedersen hash taking a 64-bit value as input.
    // HashPed64(HashPed64<N>),
    // /// Performs a Pedersen hash taking a 128-bit value as input.
    // HashPed128(HashPed128<N>),
    // /// Performs a Poseidon hash with an input rate of 2.
    // HashPsd2(HashPsd2<N>),
    // /// Performs a Poseidon hash with an input rate of 4.
    // HashPsd4(HashPsd4<N>),
    // /// Performs a Poseidon hash with an input rate of 8.
    // HashPsd8(HashPsd8<N>),
    // /// Computes the multiplicative inverse of `first`, storing the outcome in `destination`.
    // Inv(Inv<N>),
    // /// Checks if `first` is less than `second`, storing the outcome in `destination`.
    // LessThan(LessThan<N>),
    // /// Checks if `first` is less than or equal to `second`, storing the outcome in `destination`.
    // LessThanOrEqual(LessThanOrEqual<N>),
    /// Multiplies `first` with `second`, storing the outcome in `destination`.
    Mul(Mul<N>),
    /// Multiplies `first` with `second`, wrapping around at the boundary of the type, and storing the outcome in `destination`.
    MulWrapped(MulWrapped<N>),
    // /// Returns false only if `first` and `second` are true, storing the outcome in `destination`.
    // Nand(Nand<N>),
    // /// Negates `first`, storing the outcome in `destination`.
    // Neg(Neg<N>),
    // /// Returns true when neither `first` nor `second` is true, storing the outcome in `destination`.
    // Nor(Nor<N>),
    // /// Flips each bit in the representation of `first`, storing the outcome in `destination`.
    // Not(Not<N>),
    // /// Returns true if `first` is not equal to `second`, storing the result in `destination`.
    // NotEqual(NotEqual<N>),
    // /// Performs a bitwise Or on `first` and `second`, storing the outcome in `destination`.
    // Or(Or<N>),
    // /// Raises `first` to the power of `second`, storing the outcome in `destination`.
    // Pow(Pow<N>),
    // /// Raises `first` to the power of `second`, wrapping around at the boundary of the type, storing the outcome in `destination`.
    // PowWrapped(PowWrapped<N>),
    // /// Performs a Poseidon PRF with an input rate of 2.
    // PRFPsd2(PRFPsd2<N>),
    // /// Performs a Poseidon PRF with an input rate of 4.
    // PRFPsd4(PRFPsd4<N>),
    // /// Performs a Poseidon PRF with an input rate of 8.
    // PRFPsd8(PRFPsd8<N>),
    // /// Shifts `first` left by `second` bits, storing the outcome in `destination`.
    // Shl(Shl<N>),
    // /// Shifts `first` left by `second` bits, wrapping around at the boundary of the type, storing the outcome in `destination`.
    // ShlWrapped(ShlWrapped<N>),
    // /// Shifts `first` right by `second` bits, storing the outcome in `destination`.
    // Shr(Shr<N>),
    // /// Shifts `first` right by `second` bits, wrapping around at the boundary of the type, storing the outcome in `destination`.
    // ShrWrapped(ShrWrapped<N>),
    // /// Squares 'first', storing the outcome in `destination`.
    // Square(Square<N>),
    /// Computes `first - second`, storing the outcome in `destination`.
    Sub(Sub<N>),
    /// Computes `first - second`, wrapping around at the boundary of the type, and storing the outcome in `destination`.
    SubWrapped(SubWrapped<N>),
    // /// Selects `first`, if `condition` is true, otherwise selects `second`, storing the result in `destination`.
    // Ternary(Ternary<N>),
    // /// Performs a bitwise Xor on `first` and `second`, storing the outcome in `destination`.
    // Xor(Xor<N>),
}

/// Derives `From<Operation>` for the instruction.
///
/// ## Example
/// ```ignore
/// derive_from_operation!(Instruction, |None| {}, { Add, Sub, Mul, Div })
/// ```
macro_rules! derive_from_operation {
    ($_object:expr, |$_reader:ident| $_operation:block, { $( $variant:ident, )+ }) => {
        $(impl<N: Network> From<$variant<N>> for Instruction<N> {
            #[inline]
            fn from(operation: $variant<N>) -> Self {
                Self::$variant(operation)
            }
        })+
    }
}
instruction!(derive_from_operation, Instruction, |None| {});

impl<N: Network> Instruction<N> {
    // /// Returns the opcode of the instruction.
    // #[inline]
    // pub(crate) fn opcode(&self) -> &'static str {
    //     instruction!(self, |InstructionMember| InstructionMember::<N>::opcode())
    // }

    /// Returns the operands of the instruction.
    #[inline]
    pub(crate) fn operands(&self) -> &[Operand<N>] {
        instruction!(self, |instruction| instruction.operands())
    }

    /// Returns the destination register of the instruction.
    #[inline]
    pub(crate) fn destination(&self) -> &Register<N> {
        instruction!(self, |instruction| instruction.destination())
    }

    /// Evaluates the instruction.
    #[inline]
    pub(in crate::function) fn evaluate(&self, stack: &mut Stack<N>) -> Result<()> {
        instruction!(self, |instruction| instruction.evaluate(stack))
    }

    /// Returns the output type from the given input types.
    #[inline]
    pub(crate) fn output_type(&self, inputs: &[RegisterType<N>]) -> Result<RegisterType<N>> {
        instruction!(self, |InstructionMember| InstructionMember::<N>::output_type(inputs))
    }
}

impl<N: Network> Parser for Instruction<N> {
    /// Parses a string into an instruction.
    #[inline]
    fn parse(string: &str) -> ParserResult<Self> {
        /// Create an alt parser that matches the instruction.
        ///
        /// `nom` documentation notes that alt supports a maximum of 21 parsers.
        /// The documentation suggests to nest alt to support more parsers, as we do here.
        /// Note that order of the individual parsers matters.
        macro_rules! alt_parser {
            ($v0:expr) => {{ alt(($v0,)) }};
            ($v0:expr, $v1:expr) => {{ alt(($v0, $v1,)) }};
            ($v0:expr, $v1:expr, $v2:expr) => {{ alt(($v0, $v1, $v2,)) }};
            ($v0:expr, $v1:expr, $v2:expr, $v3:expr) => {{ alt(($v0, $v1, $v2, $v3,)) }};
            ($v0:expr, $v1:expr, $v2:expr, $v3:expr, $v4:expr) => {{ alt(($v0, $v1, $v2, $v3, $v4,)) }};
            ($v0:expr, $v1:expr, $v2:expr, $v3:expr, $v4:expr, $v5:expr) => {{ alt(($v0, $v1, $v2, $v3, $v4, $v5,)) }};
            ($v0:expr, $v1:expr, $v2:expr, $v3:expr, $v4:expr, $v5:expr, $v6:expr) => {{ alt(($v0, $v1, $v2, $v3, $v4, $v5, $v6,)) }};
            ($v0:expr, $v1:expr, $v2:expr, $v3:expr, $v4:expr, $v5:expr, $v6:expr, $v7:expr) => {{ alt(($v0, $v1, $v2, $v3, $v4, $v5, $v6, $v7,)) }};
            ($v0:expr, $v1:expr, $v2:expr, $v3:expr, $v4:expr, $v5:expr, $v6:expr, $v7:expr, $v8:expr) => {{ alt(($v0, $v1, $v2, $v3, $v4, $v5, $v6, $v7, $v8,)) }};
            ($v0:expr, $v1:expr, $v2:expr, $v3:expr, $v4:expr, $v5:expr, $v6:expr, $v7:expr, $v8:expr, $v9:expr) => {{ alt(($v0, $v1, $v2, $v3, $v4, $v5, $v6, $v7, $v8, $v9,)) }};
            ($v0:expr, $v1:expr, $v2:expr, $v3:expr, $v4:expr, $v5:expr, $v6:expr, $v7:expr, $v8:expr, $v9:expr, $( $variants:expr ),*) => {{ alt((
                alt_parser!($( $variants ),*), $v0, $v1, $v2, $v3, $v4, $v5, $v6, $v7, $v8, $v9,
            )) }};
        }

        /// Creates a parser for the given instructions.
        ///
        /// ## Example
        /// ```ignore
        /// instruction_parsers!(self, |_instruction| {}, { Add, Sub, Mul, Div })
        /// ```
        macro_rules! instruction_parsers {
            ($object:expr, |_instruction| $_operation:block, { $( $variant:ident, )+ }) => {{
                alt_parser!( $( map($variant::parse, Into::into) ),+ )
            }};
        }

        // Parse the whitespace and comments from the string.
        let (string, _) = Sanitizer::parse(string)?;
        // Parse the instruction from the string.
        let (string, instruction) = instruction!(instruction_parsers!(self, _instruction))(string)?;
        // Parse the semicolon from the string.
        let (string, _) = tag(";")(string)?;

        Ok((string, instruction))
    }
}

impl<N: Network> FromStr for Instruction<N> {
    type Err = Error;

    /// Parses a string into an instruction.
    #[inline]
    fn from_str(string: &str) -> Result<Self> {
        match Self::parse(string) {
            Ok((remainder, object)) => {
                // Ensure the remainder is empty.
                ensure!(remainder.is_empty(), "Failed to parse string. Found invalid character in: \"{remainder}\"");
                // Return the object.
                Ok(object)
            }
            Err(error) => bail!("Failed to parse string. {error}"),
        }
    }
}

impl<N: Network> Debug for Instruction<N> {
    /// Prints the instruction as a string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<N: Network> Display for Instruction<N> {
    /// Prints the instruction as a string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        instruction!(self, |instruction| write!(f, "{};", instruction))
    }
}

impl<N: Network> FromBytes for Instruction<N> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        /// Creates a match statement that produces the `FromBytes` implementation for the given instruction.
        ///
        /// ## Example
        /// ```ignore
        /// instruction_from_bytes_le!(self, |reader| {}, { Add, Sub, Mul, Div })
        /// ```
        macro_rules! instruction_from_bytes_le {
            ($object:expr, |$reader:ident| $_operation:block, { $( $variant:ident, )+ }) => {{
                // A list of instruction enum variants.
                const INSTRUCTION_VARIANTS: &[&'static str] = &[ $( stringify!($variant), )+];
                // Ensure the size is sufficiently large.
                assert!(INSTRUCTION_VARIANTS.len() <= u16::MAX as usize);

                // Read the enum variant index.
                let variant = u16::read_le(&mut $reader)?;

                // Build the cases for all instructions.
                $(if INSTRUCTION_VARIANTS[variant as usize] == stringify!($variant) {
                    // Read the instruction.
                    let instruction = $variant::read_le(&mut $reader)?;
                    // Return the instruction.
                    return Ok(Self::$variant(instruction));
                })+
                // If the index is out of bounds, return an error.
                Err(error(format!("Failed to deserialize an instruction of variant {variant}")))
            }};
        }
        instruction!(instruction_from_bytes_le!(self, reader))
    }
}

impl<N: Network> ToBytes for Instruction<N> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        /// Creates a match statement that produces the `ToBytes` implementation for the given instruction.
        ///
        /// ## Example
        /// ```ignore
        /// instruction_to_bytes_le!(self, |writer| {}, { Add, Sub, Mul, Div })
        /// ```
        macro_rules! instruction_to_bytes_le {
            ($object:expr, |$writer:ident| $_operation:block, { $( $variant:ident, )+ }) => {{
                // A list of instruction enum variants.
                const INSTRUCTION_VARIANTS: &[&'static str] = &[ $( stringify!($variant), )+];
                // Ensure the size is sufficiently large.
                assert!(INSTRUCTION_VARIANTS.len() <= u16::MAX as usize);

                // Build the match cases.
                match $object {
                    $(Self::$variant(instruction) => {
                        // Retrieve the enum variant index.
                        // Note: This unwrap is guaranteed to succeed because the enum variant is known to exist.
                        let variant = INSTRUCTION_VARIANTS.iter().position(|&name| stringify!($variant) == name).unwrap();

                        // Serialize the instruction.
                        u16::write_le(&(variant as u16),&mut $writer)?;
                        instruction.write_le(&mut $writer)?;
                    }),+
                }
                Ok(())
            }};
        }
        instruction!(instruction_to_bytes_le!(self, writer))
    }
}
