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

mod input;
use input::*;

mod instruction;
pub(crate) use instruction::*;

mod output;
use output::*;

mod register;
pub(crate) use register::Register;

mod bytes;
mod parse;

use crate::{
    function::{Input, Output},
    program::Stack,
    Identifier,
    Sanitizer,
};
use snarkvm_console_network::prelude::*;

use indexmap::IndexSet;

#[derive(Clone, PartialEq, Eq)]
pub struct Function<N: Network> {
    /// The name of the function.
    name: Identifier<N>,
    /// The input statements, added in order of the input registers.
    /// Input assignments are ensured to match the ordering of the input statements.
    inputs: IndexSet<Input<N>>,
    /// The instructions, in order of execution.
    instructions: Vec<Instruction<N>>,
    /// The output statements, in order of the desired output.
    /// There is no expectation that the output registers are in any ordering.
    outputs: IndexSet<Output<N>>,
}

impl<N: Network> Function<N> {
    /// Initializes a new function with the given name.
    pub fn new(name: Identifier<N>) -> Self {
        Self { name, inputs: IndexSet::new(), instructions: Vec::new(), outputs: IndexSet::new() }
    }

    /// Returns the name of the function.
    pub const fn name(&self) -> &Identifier<N> {
        &self.name
    }

    /// Returns the function inputs.
    pub const fn inputs(&self) -> &IndexSet<Input<N>> {
        &self.inputs
    }

    /// Returns the function instructions.
    pub fn instructions(&self) -> &[Instruction<N>] {
        &self.instructions
    }

    /// Returns the function outputs.
    pub const fn outputs(&self) -> &IndexSet<Output<N>> {
        &self.outputs
    }
}

impl<N: Network> Function<N> {
    /// Adds the input statement to the function.
    ///
    /// # Errors
    /// This method will halt if there are instructions or output statements already.
    /// This method will halt if the maximum number of inputs has been reached.
    /// This method will halt if the input statement was previously added.
    #[inline]
    fn add_input(&mut self, input: Input<N>) -> Result<()> {
        // Ensure there are no instructions or output statements in memory.
        ensure!(self.instructions.is_empty(), "Cannot add inputs after instructions have been added");
        ensure!(self.outputs.is_empty(), "Cannot add inputs after outputs have been added");

        // Ensure the maximum number of inputs has not been exceeded.
        ensure!(self.inputs.len() <= N::MAX_FUNCTION_INPUTS, "Cannot add more than {} inputs", N::MAX_FUNCTION_INPUTS);
        // Ensure the input statement was not previously added.
        ensure!(!self.inputs.contains(&input), "Cannot add duplicate input statement");

        // Insert the input statement.
        self.inputs.insert(input);
        Ok(())
    }

    /// Adds the given instruction to the function.
    ///
    /// # Errors
    /// This method will halt if there are no input statements in memory.
    /// This method will halt if the maximum number of instructions has been reached.
    #[inline]
    pub fn add_instruction(&mut self, instruction: Instruction<N>) -> Result<()> {
        // Ensure there are input statements in memory.
        ensure!(!self.inputs.is_empty(), "Cannot add instructions before inputs have been added");

        // Ensure the maximum number of instructions has not been exceeded.
        ensure!(
            self.instructions.len() <= N::MAX_FUNCTION_INSTRUCTIONS,
            "Cannot add more than {} instructions",
            N::MAX_FUNCTION_INSTRUCTIONS
        );

        // Insert the instruction.
        self.instructions.push(instruction);
        Ok(())
    }

    /// Adds the output statement to the function.
    ///
    /// # Errors
    /// This method will halt if there are no input statements or instructions in memory.
    /// This method will halt if the maximum number of outputs has been reached.
    #[inline]
    fn add_output(&mut self, output: Output<N>) -> Result<()> {
        // Ensure there are input statements and instructions in memory.
        ensure!(!self.inputs.is_empty(), "Cannot add outputs before inputs have been added");
        ensure!(!self.instructions.is_empty(), "Cannot add outputs before instructions have been added");

        // Ensure the maximum number of outputs has not been exceeded.
        ensure!(
            self.outputs.len() <= N::MAX_FUNCTION_OUTPUTS,
            "Cannot add more than {} outputs",
            N::MAX_FUNCTION_OUTPUTS
        );

        // Insert the output statement.
        self.outputs.insert(output);
        Ok(())
    }

    /// Evaluates the function on the given inputs.
    ///
    /// # Errors
    /// This method will halt if there are no input statements or instructions in memory.
    #[inline]
    pub fn evaluate(&self, stack: &mut Stack<N>) -> Result<()> {
        // Ensure there are input statements and instructions in memory.
        ensure!(!self.inputs.is_empty(), "Cannot evaluate a function without input statements");
        ensure!(!self.instructions.is_empty(), "Cannot evaluate a function without instructions");

        // Evaluate the instructions.
        self.instructions.iter().try_for_each(|instruction| instruction.evaluate(stack))
    }
}

impl<N: Network> TypeName for Function<N> {
    /// Returns the type name as a string.
    #[inline]
    fn type_name() -> &'static str {
        "function"
    }
}
