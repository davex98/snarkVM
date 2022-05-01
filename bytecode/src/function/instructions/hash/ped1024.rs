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

/// Performs a Pedersen hash taking a 1024-bit value as input.
pub struct Ped1024<P: Program> {
    operation: UnaryOperation<P>,
}

impl_instruction_boilerplate!(Ped1024, UnaryOperation, "hash.ped1024");

impl_hash_instruction!(Ped1024);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_instruction_halts, test_modes, Identifier, Process};

    type P = Process;

    #[test]
    fn test_parse() {
        let (_, instruction) = Instruction::<P>::parse("hash.ped1024 r0 into r1;").unwrap();
        assert!(matches!(instruction, Instruction::Ped1024(_)));
    }

    test_modes!(
        address,
        Ped1024,
        "aleo1d5hg2z3ma00382pngntdp68e74zv54jdxy249qhaujhks9c72yrs33ddah",
        "7481826612753713410217729925917351756348444555753582640939800395433217644869field"
    );
    test_modes!(
        bool,
        Ped1024,
        "true",
        "6122249396247477588925765696834100286827340493907798245233656838221917119242field"
    );
    test_modes!(
        field,
        Ped1024,
        "1field",
        "6122249396247477588925765696834100286827340493907798245233656838221917119242field"
    );
    test_modes!(
        group,
        Ped1024,
        "2group",
        "564546604862166251269187547407669874296117017437472033102698766525356841251field"
    );
    test_modes!(
        i8,
        Ped1024,
        "1i8",
        "6122249396247477588925765696834100286827340493907798245233656838221917119242field"
    );
    test_modes!(
        i16,
        Ped1024,
        "1i16",
        "6122249396247477588925765696834100286827340493907798245233656838221917119242field"
    );
    test_modes!(
        i32,
        Ped1024,
        "1i32",
        "6122249396247477588925765696834100286827340493907798245233656838221917119242field"
    );
    test_modes!(
        i64,
        Ped1024,
        "1i64",
        "6122249396247477588925765696834100286827340493907798245233656838221917119242field"
    );
    test_modes!(
        i128,
        Ped1024,
        "1i128",
        "6122249396247477588925765696834100286827340493907798245233656838221917119242field"
    );
    test_modes!(
        u8,
        Ped1024,
        "1u8",
        "6122249396247477588925765696834100286827340493907798245233656838221917119242field"
    );
    test_modes!(
        u16,
        Ped1024,
        "1u16",
        "6122249396247477588925765696834100286827340493907798245233656838221917119242field"
    );
    test_modes!(
        u32,
        Ped1024,
        "1u32",
        "6122249396247477588925765696834100286827340493907798245233656838221917119242field"
    );
    test_modes!(
        u64,
        Ped1024,
        "1u64",
        "6122249396247477588925765696834100286827340493907798245233656838221917119242field"
    );
    test_modes!(
        u128,
        Ped1024,
        "1u128",
        "6122249396247477588925765696834100286827340493907798245233656838221917119242field"
    );
    test_modes!(
        scalar,
        Ped1024,
        "1scalar",
        "6122249396247477588925765696834100286827340493907798245233656838221917119242field"
    );
    test_modes!(
        string,
        Ped1024,
        "\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"",
        "6527352455101447058676936043301538275522692367742743342479030477370441210344field"
    );

    test_instruction_halts!(
        string_halts,
        Ped1024,
        "The Pedersen hash input cannot exceed 1024 bits.",
        "\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\""
    );

    #[test]
    fn test_composite() {
        let first = Value::<P>::Composite(Identifier::from_str("message"), vec![
            Literal::from_str("true.public"),
            Literal::from_str("false.private"),
        ]);

        let registers = Registers::<P>::default();
        registers.define(&Register::from_str("r0"));
        registers.define(&Register::from_str("r1"));
        registers.assign(&Register::from_str("r0"), first);

        Ped1024::from_str("r0 into r1").evaluate(&registers);

        let value = registers.load(&Register::from_str("r1"));
        let expected = Value::<P>::from_str(
            "6122249396247477588925765696834100286827340493907798245233656838221917119242field.private",
        );
        assert_eq!(expected, value);
    }

    #[test]
    #[should_panic(expected = "The Pedersen hash input cannot exceed 1024 bits.")]
    fn test_composite_halts() {
        let first = Value::<P>::Composite(Identifier::from_str("message"), vec![
            Literal::from_str("1field.public"),
            Literal::from_str("2field.private"),
            Literal::from_str("3field.private"),
            Literal::from_str("4field.private"),
            Literal::from_str("5field.private"),
        ]);

        let registers = Registers::<P>::default();
        registers.define(&Register::from_str("r0"));
        registers.define(&Register::from_str("r1"));
        registers.assign(&Register::from_str("r0"), first);

        Ped1024::from_str("r0 into r1").evaluate(&registers);
    }
}
