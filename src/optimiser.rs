//! Module that implements `optimise`, which reduces redundant brainfuck operators
//! between brackets. These regions between brackets are called sections.
//! 
//! Ex: `>>>-<<+>` would become `>+>>-<`.
//! We do this by analysing the cells which are changed relative to the starting position.
//! (In this case `1: +1` and `3: -1`)
//! Then we make a "path" from the start position (0) to the end (in this case 2) which modifies
//! all thoses cells with the least distance. In this situation distance is the number of '>' and '<'.

use std::{collections::HashSet, mem, ops::Range};

/// Takes in a brainfuck program and removes redundant brainfuck operators by bulking them in `Operation`s.
/// May break some behviour, like moving the pointer at the end of the program.
pub fn optimise(bf: &str) -> String {
    // Parses the operations
    let mut operations = parse_operations(&bf).0;

    // -- Reordering and merging the operations --
    // Note how we didn't collect the operations into a hashmap as that would
    // lose the orignial operation ordering. This is important, because while we can reorder
    // '+' and '-' all we want, we can't do the same with ',' and '.'. These form a dependency
    // relation. '+' and '-' cannot pass over ',' and '.' operating on the same cell.
    // Basically ',' and '.' serve as absolute barriers for operators on the same cell.
    // (Because of the lack of conditionals, we don't have to worry about one cell having causality on another)
    // NOTE: we will want to sort cell operations in increasing order if `start < end`
    // and in decreasing order if `start > end`
    merge_offsets(&mut operations);
    operations_to_brainfuck(&operations)
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
    for (idx, op) in src.char_indices() {
        // block completion mode
        if sub_section_bracket_depth != 0 {
            match (sub_section_bracket_depth, op) {
                (ref mut depth, '[') => *depth += 1,
                (1, ']') => {
                    let sub_string = &src[sub_section_start.unwrap()..=idx];
                    let sub_section = Block::new(sub_string);
                    operations.push(Operation::Block { cell: relative_cell_position, block: sub_section });

                    sub_section_bracket_depth = 0;
                    sub_section_start = None;
                }
                (ref mut depth, ']') => *depth -= 1,
                _ => (),
            }
            continue;
        }

        match op {
            '>' => relative_cell_position += 1,
            '<' => relative_cell_position -= 1,
            '+' => if let Some(Operation::Offset { cell, ref mut recurence }) = operations.last_mut() {
                if relative_cell_position != *cell {
                    operations.push(Operation::Offset { cell: relative_cell_position, recurence: 1 });
                } else {
                    *recurence += 1;
                }
            } else {
                operations.push(Operation::Offset { cell: relative_cell_position, recurence: 1 })
            },
            '-' => if let Some(Operation::Offset { cell, ref mut recurence }) = operations.last_mut() {
                if relative_cell_position != *cell {
                    operations.push(Operation::Offset { cell: relative_cell_position, recurence: -1 });
                } else {
                    *recurence -= 1;
                } 
            } else {
                operations.push(Operation::Offset { cell: relative_cell_position, recurence: -1 })
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
            text => if let Some(Operation::Text { src: ref mut other_src }) = operations.last_mut() {
                *other_src = &src[idx-other_src.len()..idx+text.len_utf8()];
            } else {
                operations.push(Operation::Text { src: &src[idx..idx+text.len_utf8()] });
            },
        }
    }

    // we should be done with that
    debug_assert_eq!(sub_section_bracket_depth, 0);

    // removes empty operations
    let operations = operations.into_iter().filter(|op| {
        match op {
            Operation::Offset { recurence: 0, .. } => false,
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
            Operation::Offset { recurence, .. } => {
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

/// Merges offset operations togheter as much as fencing allows.
fn merge_offsets<'a, 'b>(ops: &'a mut Vec<Operation<'b>>) {
    // gets the indexes of all offsets,
    // these won't change as we merge since we'll simply zero the recurence of merged offsets AND THEN delete them.
    let offsets_indexes = ops.iter()
        .enumerate()
        .filter_map(|(i, op)| if let Operation::Offset { .. } = op {
            Some(i)
        } else {
            None
        }).collect::<Vec<_>>();

    for offset_index in offsets_indexes {
        let (self_cell, self_recurence) = {
            let Operation::Offset { cell, recurence } = ops[offset_index] else { panic!("we know that it is offset") };
            (cell, recurence)
        };
        // Getting where we can look for merge companions
        let range = operation_validity_range(ops, offset_index);
        
        // search in front (excluding self ofc)
        let other_offset_opt = ops[range.clone()].iter_mut()
            .zip(range)
            .find(|(op, i)| if let Operation::Offset { cell, recurence } = op {
                if *i == offset_index {
                    return false
                }

                if *cell != self_cell {
                    return false
                }

                if *recurence == 0 {
                    return false;
                }

                true
            } else { false });

        let other_offset = if let Some((other_offset, 0)) = other_offset_opt {
            other_offset
        } else {
            continue;
        };

        // If we got here this means that we found a merge companion
        let Operation::Offset { recurence, .. } = other_offset else { unreachable!() };
        *recurence += self_recurence;

        // we'll want to remove the recurence we just added to the original
        let Operation::Offset { recurence, .. } = &mut ops[offset_index] else { unreachable!() };
        *recurence = 0;
    }

    // Cleanup, we remove all the offsets we set to 0
    let tmp_ops = mem::take(ops);
    *ops = tmp_ops.into_iter().filter(|op| match op {
        Operation::Offset { recurence: 0, ..} => false,
        _ => true
    }).collect();
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
        recurence: i32,
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
            Self::Block { cell, block: section } => {
                section.fences_cell(idx+cell) || idx == *cell
            },
            Self::InOut { cell, .. } => idx == *cell,
            Self::Offset { .. } => false,
            // NOTE: THIS MAY CAUSE LOOSE BRACKETS TO GET AFFECTED TO PAIRS WHICH THEY WEREN'T A PART OF PRIOR (MAYBE IDK)
            Self::LooseBracket { cell, .. } => idx == *cell,
            // to not confuse the user by seperating text from code
            // TODO: it is also intuitive, but wasteful to fence all cells because a single loop is written `[ like - this]`
            Self::Text { .. } => true,
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
            // we don't reorganise text
            (Operation::Text { .. }, _) => return false,
            (_, Operation::Text { .. }) => return false,
            // we don't reorganise io
            (Operation::InOut { .. }, Operation::InOut { .. }) => return false,
            _ => (),
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

/// Represents a matched bracket block.
#[derive(Debug, Clone, PartialEq, Default)]
struct Block<'a> {
    operations: Vec<Operation<'a>>,
    dynamic_endpoint: Option<isize>,
}

impl<'a> Block<'a> {
    /// Takes in the source of a matched bracket block.
    /// The slice should start and end with '[' and ']' respectively.
    fn new(src: &str) -> Block {
        debug_assert!(src.starts_with('['));
        debug_assert!(src.ends_with(']'));
        
        // remove the brackets
        let content = &src[1..src.len()-1];

        let (operations, end_point) = parse_operations(content);

        Block {
            operations,
            dynamic_endpoint: end_point,
        }
    }

    /// Returns true if the section is dynamic, aka does it offset the tape pointer.
    fn is_dynamic(&self) -> bool {
        self.dynamic_endpoint.is_some()
    }

    fn fences_cell(&self, idx: isize) -> bool {
        // we can't trust this section to not use this cell if it is dynamic
        if self.is_dynamic() {
            return true;
        }

        // we read from 0 to branch or not, so this needs to fence
        if idx == 0 {
            return true;
        }

        self.operations.iter().find(|op| op.fences_cell(idx)).is_some()
    }

    fn modified_cells(&self) -> HashSet<isize> {
        self.operations.iter().fold(HashSet::new(), |mut buf, op| {
            for cell in op.modified_cells() {
                buf.insert(cell);
            };

            buf
        })
    }

    fn to_brainfuck(&self) -> String {
        let mut buf = operations_to_brainfuck(&self.operations);
        
        // we want to end on the dynamic endpoint, or at the start if we are not dynamic
        // find the last position we got put on
        let last_position = self.operations.iter()
            .rfind(|op| op.cell_position().is_some())
            .map(|op| op.cell_position().unwrap())
            .unwrap_or(0);
        
        let offset = self.dynamic_endpoint.unwrap_or(0);
        let difference = offset - last_position;
            let movement_ch = if difference.is_positive() { '>' } else { '<' };
            
            for _ in 0..(difference.abs()) {
                buf.push(movement_ch);
        }

        // we are a bracket block after all
        buf.insert(0, '[');
        buf.push(']');

        buf
    }
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
            Operation::Offset { cell: 2, recurence: 1 },
            Operation::Offset { cell: 1, recurence: -1 },
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
        assert_matches!(ops[1], Operation::Offset { cell: 0, recurence: 1 });
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
        let program = include_str!("../test-resources/fib.bf");
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
    fn merging_offset_optimisation() {
        // does nothing when there is nothing to do
        let (mut ops, _) = parse_operations("[>-  the harsh advice of those who love you rots within this hell<<]");
        merge_offsets(&mut ops);
        let string = operations_to_brainfuck(&ops);
        assert_eq!(string, "[>-  the harsh advice of those who love you rots within this hell<<]");

        // does nothing when there is nothing to do
        let (mut ops, _) = parse_operations("+++[-]--");
        assert_eq!(operation_validity_range(&ops, 0), 0..1);
        merge_offsets(&mut ops);
        let string = operations_to_brainfuck(&ops);
        assert_eq!(string, "+++[-]--");

        // does something when it needs to
        let (mut ops, _) = parse_operations("+++>[-]<--");
        assert_eq!(operation_validity_range(&ops, 0), 0..3);
        merge_offsets(&mut ops);
        let string = operations_to_brainfuck(&ops);
        assert_eq!(string, "+>[-]");

        let (mut ops, _) = parse_operations("++>>[-][-<<+>>]<<++");
        merge_offsets(&mut ops);
        let string = operations_to_brainfuck(&ops);
        assert_eq!(string, "++++>>[-][-<<+>>]");
    }
}

