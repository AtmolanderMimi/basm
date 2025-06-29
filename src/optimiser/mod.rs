//! Module that implements `optimise`, which reduces redundant brainfuck operators
//! between brackets. These regions between brackets are called sections.
//! 
//! Ex: `>>>-<<+>` would become `>+>>-<`.
//! We do this by analysing the cells which are changed relative to the starting position.
//! (In this case `1: +1` and `3: -1`)
//! Then we make a "path" from the start position (0) to the end (in this case 2) which modifies
//! all thoses cells with the least distance. In this situation distance is the number of '>' and '<'.

use std::{collections::HashSet, ops::Range};
mod block;
use block::Block;
mod optimisations;


/// Takes in a brainfuck program and removes redundant brainfuck operators by bulking them in `Operation`s.
/// May break some behviour, like moving the pointer at the end of the program.
pub fn optimise(bf: &str) -> String {
    // Parses the operations
    let mut operations = parse_operations(&bf).0;

    optimisations::reorder_operations(&mut operations);

    optimisations::merge_offsets(&mut operations);

    // running it twice shaves off a few characters
    optimisations::reorder_operations(&mut operations);

    optimisations::remove_offsets_before_zeroing(&mut operations);

    operations_to_brainfuck(&operations)
}

// TODO: Zero operation would enable more optimisations.
/// Brainfuck operations on cells. Operations are collections of operators that achieve one thing.
#[derive(Debug, Clone, PartialEq)]
enum Operation<'a> {
    Block {
        cell: isize,
        block: Block<'a>,
    },
    Offset {
        cell: isize,
        recurrence: i32,
    },
    InOut {
        cell: isize,
        operator: char,
    },
    LooseBracket {
        cell: isize,
        operator: char,
    },
    Text {
        src: &'a str,
    },

}

impl<'a> Operation<'a> {
    // NOTE: the return of this function may be the cause of weird behaviour, investigate first with `modified_cells`
    /// Returns true if the value of the cell at `idx` needs to be known by the operation else
    /// returns false if the operation is "blind" to the value of the cell.
    /// An operation is blind if it does not care about the value of the cell.
    /// For example `+` and `-` are because their behviour is not affected by the value of the cell they offset.
    /// (Look for modified cells if you want the cells afected by the operations rather than the cells the operation require)
    /// ',' '.' on the other hand do care about the value. (',' is special)
    fn fences_cell(&self, idx: isize) -> bool {
        match self {
            Self::Block { cell, block } => block.fences_cell(idx-cell),
            Self::InOut { cell, .. } => idx == *cell,
            Self::Offset { .. } => false,
            // NOTE: THIS MAY CAUSE LOOSE BRACKETS TO GET AFFECTED TO PAIRS WHICH THEY WEREN'T A PART OF PRIOR (MAYBE IDK)
            Self::LooseBracket { cell, .. } => idx == *cell,
            Self::Text { .. } => false,
        }
    }

    /// Returns a `HashSet` of the cells which are accessed and modified by the operation.
    fn modified_cells(&self) -> HashSet<isize> {
        let vec = match self {
            Self::Block { cell, block: section } => {
                section.modified_cells().into_iter()
                    .map(|c| c+cell)
                    .collect()
            },
            Self::InOut { cell, operator } => if *operator == '.' {
                vec![] // out does not modify the cell (it still needs the value, so still fence though)
            } else {
                vec![*cell]
            },
            Self::Offset { cell, .. } => vec![*cell],
            Self::LooseBracket { .. } => vec![], //vec![*cell],
            Self::Text { .. } => vec![],
        };

        // TODO: because i am lazy, but in the best of worlds we don't pass by vec
        HashSet::from_iter(vec)
    }

    fn cell_position(&self) -> Option<isize> {
        match self {
            Self::Block { cell, .. } => Some(*cell),
            Self::InOut { cell, .. } => Some(*cell),
            Self::Offset { cell, .. } => Some(*cell),
            Self::LooseBracket { cell, .. } => Some(*cell),
            Self::Text { .. } => None,
        }
    }

    fn can_swap(&self, other: &Self) -> bool {
        // we don't want to 
        match (self, other) {
            // we don't reorganise text because that would be unintuitive
            (Operation::Text { .. }, _) => return false,
            (_, Operation::Text { .. }) => return false,
            // we don't reorganise io
            (Operation::InOut { .. }, Operation::InOut { .. }) => return false,
            _ => (),
        }

        // checks if there is a dynmanic block just in case
        // (fencing and use may not work if the dynamic does not modify anything)
        if let Operation::Block { block, .. } = self {
            if block.is_dynamic() {
                return false;
            }
        }
        
        if let Operation::Block { block, .. } = other {
            if block.is_dynamic() {
                return false;
            }
        }

        // checks whether their fences allow swapping
        for cell in self.modified_cells() {
            if other.fences_cell(cell) {
                return false;
            }
        }
        // checks whether our fences allow swapping
        for cell in other.modified_cells() {
            if self.fences_cell(cell) {
                return false;
            }
        }

        true
    }
}

/// Encodes a string into a series of operations.
/// Returns the vector of operations and a `Option<usize>` representing if the code is dynamic.
/// The option is `None` if the code is not dynamic and `Some(offset)` where offset is the amount off from the starting point.
/// It is possible that a code segment is dynamic while having an offset of 0, this means that a subsection (block) is dynamic.
/// Dynamic means that it does not end where it started, so if it were to be executed it would
/// offset all other operations.
/// This function does not provide any optimisations in itself.
fn parse_operations(src: &str) -> (Vec<Operation>, Option<isize>) {
    // -- Encoding the operations on the cells --
    let mut operations = Vec::new();
    let mut relative_cell_position = 0; // NOTE: this may be invalid when dynamic is involved
    let mut sub_section_bracket_depth = 0;
    let mut sub_section_start = None;
    let mut last_op_is_text = false;
    for (idx, op) in src.char_indices() {
        // block completion mode
        if sub_section_bracket_depth != 0 {
            match (sub_section_bracket_depth, op) {
                // ahem, ahem, rust go to hell (matching and integer with the ref mut pattern will explicitly copy the integer)
                (_, '[') => sub_section_bracket_depth += 1,
                (1, ']') => {
                    let sub_string = &src[sub_section_start.unwrap()..=idx];
                    let sub_section = Block::new(sub_string);
                    operations.push(Operation::Block { cell: relative_cell_position, block: sub_section });

                    sub_section_bracket_depth = 0;
                    sub_section_start = None;
                }
                (_, ']') => sub_section_bracket_depth -= 1,
                _ => (),
            }
            continue;
        }

        match op {
            '>' => { relative_cell_position += 1; last_op_is_text = false; },
            '<' => { relative_cell_position -= 1; last_op_is_text = false; },
            '+' => if let Some(Operation::Offset { cell, recurrence: ref mut recurence }) = operations.last_mut() {
                if relative_cell_position != *cell {
                    operations.push(Operation::Offset { cell: relative_cell_position, recurrence: 1 });
                } else {
                    *recurence += 1;
                }
            } else {
                operations.push(Operation::Offset { cell: relative_cell_position, recurrence: 1 })
            },
            '-' => if let Some(Operation::Offset { cell, recurrence: ref mut recurence }) = operations.last_mut() {
                if relative_cell_position != *cell {
                    operations.push(Operation::Offset { cell: relative_cell_position, recurrence: -1 });
                } else {
                    *recurence -= 1;
                } 
            } else {
                operations.push(Operation::Offset { cell: relative_cell_position, recurrence: -1 })
            },
            ',' => operations.push(Operation::InOut { operator: ',', cell: relative_cell_position }),
            '.' => operations.push(Operation::InOut { operator: '.', cell: relative_cell_position }),
            '[' => {
                let number_left_brackets_left = &src[idx+1..].chars().filter(|c| *c == '[').count();
                let number_right_brackets_left = &src[idx+1..].chars().filter(|c| *c == ']').count();
                let is_loose = number_left_brackets_left >= number_right_brackets_left;
                if is_loose {
                    operations.push(Operation::LooseBracket { cell: relative_cell_position, operator: '[' });
                } else {
                    sub_section_bracket_depth = 1;
                    sub_section_start = Some(idx);
                }
            },
            ']' => operations.push(Operation::LooseBracket { cell: relative_cell_position, operator: ']' }),
            ch => {
                if last_op_is_text {
                    if let Some(Operation::Text { src: other_src }) = operations.last_mut() {
                        let new_src = &src[idx-other_src.len()..idx+ch.len_utf8()];
                        *other_src = new_src;
                    }
                } else {
                    operations.push(Operation::Text { src: &src[idx..idx+ch.len_utf8()] });
                }

                last_op_is_text = true;
            },
        }
    }

    // we should be done with that
    debug_assert_eq!(sub_section_bracket_depth, 0);

    // removes empty operations
    let operations = operations.into_iter().filter(|op| {
        match op {
            Operation::Offset { recurrence: 0, .. } => false,
            _ => true,
        }
    }).collect::<Vec<_>>();

    // checks recursively if one of the blocks is dynamic
    let dynamic_sub_section = operations.iter()
        .filter_map(|op| if let Operation::Block { block: section, .. } = op { Some(section) } else { None })
        .find(|sec| sec.is_dynamic())
        .is_some();
    // if we don't end at the same place we started (0), then WE are dynamic
    let is_dynamic = relative_cell_position != 0 || dynamic_sub_section;

    let dynamic_endpoint = if is_dynamic {
        Some(relative_cell_position)
    } else {
        None
    };

    (operations, dynamic_endpoint)
}

/// Turns back the operations into text format.
fn operations_to_brainfuck(ops: &[Operation]) -> String {
    let mut tape_pointer = 0;
    let mut buf = String::new();

    for op in ops {
        // move to position (if required)
        if let Some(ntape) = op.cell_position() {
            let difference = ntape - tape_pointer;
            let movement_ch = if difference.is_positive() { '>' } else { '<' };
            
            for _ in 0..(difference.abs()) {
                buf.push(movement_ch);
            }

            tape_pointer = ntape;
        }

        // do the actual operation
        let op_str = match op {
            Operation::Block { block, .. } => block.to_brainfuck(),
            Operation::InOut { operator, .. } => operator.to_string(),
            Operation::Offset { recurrence: recurence, .. } => {
                let mut offset_buf = String::new();
                let offset_ch = if recurence.is_positive() { '+' } else { '-' };
            
                for _ in 0..(recurence.abs()) {
                    offset_buf.push(offset_ch);
                }

                offset_buf
            },
            Operation::LooseBracket { operator, .. } => operator.to_string(),
            Operation::Text { src } => { buf.push_str(&src); continue; },
        };

        buf.push_str(&op_str);
    }

    buf
}

/// Returns the range of valid positions the operator at `idx` can be inserted at.
/// If the index is invalid returns an empty `0..0` range.
fn operation_validity_range<'a, 'b>(ops: &'a [Operation<'b>], idx: usize) -> Range<usize> {
    let Some(operator) = ops.get(idx) else { return 0..0 };
    let mut range = idx..idx+1;

    // searching forward
    loop {
        if range.end >= ops.len() {
            break
        }

        let other = ops.get(range.end)
            .expect("Since we know that range.start != ops.len(), we know we can be guarentied there is more");
        if !operator.can_swap(other) {
            break
        }

        range.end += 1;
    }

    // searching backward
    loop {
        if range.start == 0 {
            break
        }

        let other = ops.get(range.start-1)
            .expect("Since we know that range.start != 0, we know we can substract and not go oob");
        if !operator.can_swap(other) {
            break
        }

        range.start -= 1;
    }

    range
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::interpreter::InterpreterBuilder;

    use super::*;

    #[test]
    fn optimiser_basic_parsing_operations() {
        let (ops, end_point) = parse_operations(">>+<->.");
        assert!(end_point.is_some());
        assert_eq!(ops, vec![
            Operation::Offset { cell: 2, recurrence: 1 },
            Operation::Offset { cell: 1, recurrence: -1 },
            Operation::InOut { cell: 2, operator: '.' }
        ]);

        let (ops, end_point) = parse_operations(">>,[---++]++--<<");
        assert!(end_point.is_none());
        assert_matches!(ops[0], Operation::InOut { cell: 2, operator: ',' });
        assert_matches!(ops[1], Operation::Block { cell: 2, .. });

        let (ops, end_point) = parse_operations(">>,[--+]++--<<");
        assert!(end_point.is_none());
        assert_matches!(ops[0], Operation::InOut { cell: 2, operator: ',' });
        assert_matches!(ops[1], Operation::Block { cell: 2, .. });

        let (ops, end_point) = parse_operations("did \n you know +>.[<atmic bomb[++-]");
        assert!(end_point.is_none());
        assert_matches!(ops[0], Operation::Text { src: "did \n you know " });
        assert_matches!(ops[1], Operation::Offset { cell: 0, recurrence: 1 });
        assert_matches!(ops[2], Operation::InOut { cell: 1, operator: '.' });
        assert_matches!(ops[3], Operation::LooseBracket { cell: 1, operator: '[' });
        assert_matches!(ops[4], Operation::Text { src: "atmic bomb" });
        assert_matches!(ops[5], Operation::Block { cell: 0, .. });
    }

    #[test]
    fn optimiser_modified_cells_block() {
        let (ops, _) = parse_operations(">>,[--+]++--<<");
        assert_eq!(ops[1].modified_cells(), HashSet::from_iter([2]));

        let (ops, _) = parse_operations(">>>[-<<+>>]");
        assert_eq!(ops[0].modified_cells(), HashSet::from_iter([1, 3]));
        let (ops, _) = parse_operations(">>>>>[<<++<<[->>>+<<<]>>>>]");
        assert_eq!(ops[0].modified_cells(), HashSet::from_iter([1, 3, 4]));
    }

    #[test]
    fn optimiser_swap_operations() {
        let (ops, _) = parse_operations(">>,[--+]++.--<<");
        assert!(!ops[0].can_swap(&ops[1]));
        assert!(ops[2].can_swap(&ops[4]));
        assert!(ops[4].can_swap(&ops[2]));

        let (ops, _) = parse_operations("+>>[-<+>]<->++");
        assert!(ops[0].can_swap(&ops[1]));
        assert!(ops[1].can_swap(&ops[2]));
        assert!(!ops[1].can_swap(&ops[3]));

        let (ops, _) = parse_operations("+>.<,");
        assert!(ops[0].can_swap(&ops[1]));
        assert!(!ops[0].can_swap(&ops[2]));
        assert!(!ops[1].can_swap(&ops[2]));
        assert!(!ops[2].can_swap(&ops[1]));
    }

    #[test]
    fn optimiser_operations_to_string() {
        let (ops, _) = parse_operations(">>,[>--+,]++.--<<");
        let string = operations_to_brainfuck(&ops);
        assert_eq!(string, ">>,[>-,]++.--");

        let (ops, _) = parse_operations("[>-  the harsh advice of those who love you rots within this hell<<]");
        let string = operations_to_brainfuck(&ops);
        assert_eq!(string, "[>-  the harsh advice of those who love you rots within this hell<<]");

        // behaviour is the same
        let program = include_str!("../../test-resources/fib.bf");
        let mut inter =  InterpreterBuilder::new(&program).finish();
        inter.complete().unwrap();
        let original_output = inter.captured_output();

        let parsed_and_rebuilt_program = operations_to_brainfuck(&parse_operations(&program).0);
        let mut inter =  InterpreterBuilder::new(&parsed_and_rebuilt_program).finish();
        inter.complete().unwrap();
        let new_output = inter.captured_output();

        assert_eq!(original_output, new_output);
    }

    #[test]
    fn validity_range_works() {
        // does nothing when there is nothing to do
        let (ops, _) = parse_operations(">>,[>--+,]++>.<--<<");
        assert_eq!(operation_validity_range(&ops, 0), 0..1);
        assert_eq!(operation_validity_range(&ops, 2), 2..5);
        assert_eq!(operation_validity_range(&ops, 1), 1..2);

        let (ops, _) = parse_operations("+++[-]--");
        assert!(!ops[0].can_swap(&ops[1]));
        assert_eq!(operation_validity_range(&ops, 0), 0..1);
    }

    #[test]
    fn validity_range_works_dynamic() {
        // does nothing when there is nothing to do
        let (ops, _) = parse_operations("[>][<]");
        assert_eq!(operation_validity_range(&ops, 0), 0..1);
        assert_eq!(operation_validity_range(&ops, 1), 1..2);

        let (ops, _) = parse_operations("+++>[>]<--");
        assert!(!ops[0].can_swap(&ops[1]));
        assert_eq!(operation_validity_range(&ops, 0), 0..1);
    }
}
