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

mod helpers;
use helpers::PublicOrPrivate;

mod bytes;
mod parse;

use crate::{Identifier, ValueType};
use snarkvm_console_network::prelude::*;

use indexmap::IndexMap;

/// The declared layout for program data.
#[derive(Clone, PartialEq, Eq)]
pub struct RecordType<N: Network> {
    /// The name of the record type.
    name: Identifier<N>,
    /// The visibility for the owner of the program record.
    owner: PublicOrPrivate,
    /// The visibility for the balance of the program record.
    balance: PublicOrPrivate,
    /// The name and value type for the entries in data.
    entries: IndexMap<Identifier<N>, ValueType<N>>,
}

impl<N: Network> RecordType<N> {
    /// Returns the name of the record type.
    pub const fn name(&self) -> &Identifier<N> {
        &self.name
    }

    /// Returns the visibility of the record owner.
    pub const fn owner(&self) -> PublicOrPrivate {
        self.owner
    }

    /// Returns the visibility of the record balance.
    pub const fn balance(&self) -> PublicOrPrivate {
        self.balance
    }

    /// Returns the entries of the record type.
    pub const fn entries(&self) -> &IndexMap<Identifier<N>, ValueType<N>> {
        &self.entries
    }
}

impl<N: Network> TypeName for RecordType<N> {
    /// Returns the type name.
    fn type_name() -> &'static str {
        "record"
    }
}
