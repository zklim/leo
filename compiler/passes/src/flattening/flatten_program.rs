// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

use crate::Flattener;

use leo_ast::{Finalize, Function, ProgramReconstructor, ProgramScope, Statement, StatementReconstructor};

impl ProgramReconstructor for Flattener<'_> {
    /// Flattens a program scope.
    fn reconstruct_program_scope(&mut self, input: ProgramScope) -> ProgramScope {
        self.program = Some(input.program_id.name.name);
        ProgramScope {
            program_id: input.program_id,
            structs: input.structs.into_iter().map(|(i, c)| (i, self.reconstruct_struct(c))).collect(),
            mappings: input.mappings.into_iter().map(|(id, mapping)| (id, self.reconstruct_mapping(mapping))).collect(),
            functions: input.functions.into_iter().map(|(i, f)| (i, self.reconstruct_function(f))).collect(),
            consts: input
                .consts
                .into_iter()
                .map(|(i, c)| match self.reconstruct_const(c) {
                    (Statement::Const(declaration), _) => (i, declaration),
                    _ => unreachable!("`reconstruct_const` can only return `Statement::Const`"),
                })
                .collect(),
            span: input.span,
        }
    }

    /// Flattens a function's body and finalize block, if it exists.
    /// Note that the finalize block is not flattened since it uses `branch` instructions to produce correct code in for conditional execution.
    fn reconstruct_function(&mut self, function: Function) -> Function {
        // Flatten the function body.
        let mut block = self.reconstruct_block(function.block).0;

        // Get all of the guards and return expression.
        let returns = self.clear_early_returns();

        // Fold the return statements into the block.
        self.fold_returns(&mut block, returns);

        Function {
            annotations: function.annotations,
            variant: function.variant,
            identifier: function.identifier,
            input: function.input,
            output: function.output,
            output_type: function.output_type,
            block,
            finalize: function.finalize.map(|finalize| {
                // Set the `is_finalize` flag before reconstructing the finalize block.
                self.is_finalize = true;
                // Reconstruct the finalize block.
                let finalize = Finalize {
                    identifier: finalize.identifier,
                    input: finalize.input,
                    output: finalize.output,
                    output_type: finalize.output_type,
                    block: self.reconstruct_block(finalize.block).0,
                    span: finalize.span,
                    id: finalize.id,
                };
                // Reset the `is_finalize` flag.
                self.is_finalize = false;
                finalize
            }),
            span: function.span,
            id: function.id,
        }
    }
}
