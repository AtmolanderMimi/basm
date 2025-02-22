//! Module that implements `optimise`, which reduces redundant brainfuck operators
//! between brackets. These regions between brackets are called sections.
//! 
//! Ex: `>>>-<<+>` would become `>+>>-<`.
//! We do this by analysing the cells which are changed relative to the starting position.
//! (In this case `1: +1` and `3: -1`)
//! Then we make a "path" from the start position (0) to the end (in this case 2) which modifies
//! all thoses cells with the least distance. In this situation distance is the number of '>' and '<'.

use regex::Regex;

/// Takes in a brainfuck program and removes redundant brainfuck operators in sections.
/// "Sections" are clusters of '+', '-', '>' and '<'. They are delimited by brackets and text.
/// May break some behviour, like intentionally making the tape pointer negative.
pub fn optimise(bf: &str) -> String {
    let operations = parse_operations(&bf).0;

    //operations

    // -- Reordering and merging the operations --
    // Note how we didn't collect the operations into a hashmap as that would
    // lose the orignial operation ordering. This is important, because while we can reorder
    // '+' and '-' all we want, we can't do the same with ',' and '.'. These form a dependency
    // relation. '+' and '-' cannot pass over ',' and '.' operating on the same cell.
    // Basically ',' and '.' serve as absolute barriers for operators on the same cell.
    // (Because of the lack of conditionals, we don't have to worry about one cell having causality on another)
    // NOTE: we will want to sort cell operations in increasing order if `start < end`
    // and in decreasing order if `start > end`

    todo!()
}

/// Encodes a string into a series of operations.
/// Returns the vector of operations and a bool representing if the code is dynamic.
/// Dynamic means that it does not end where it started, so if it were to be executed it would
/// offset all other operations.
/// This function does not provide any optimisations in itself.
fn parse_operations(src: &str) -> (Vec<Operation>, bool) {
    // -- Encoding the operations on the cells --
    let mut operations = Vec::new();
    let mut relative_cell_position = 0; // NOTE: this may be invalid when dynamic is involved
    let mut sub_section_bracket_depth = 0;
    let mut sub_section_start = None;
    for (idx, op) in src.char_indices() {
        // section completion mode
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
    (operations, is_dynamic)
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
    fn fences_cell(&self, idx: isize) -> bool {
        match self {
            Self::Block { cell, block: section } => section.fences_cell(idx+cell),
            Self::InOut { cell, .. } => idx == *cell,
            Self::Offset { .. } => false,
            // NOTE: THIS MAY CAUSE LOOSE BRACKETS TO GET AFFECTED TO PAIRS WHICH THEY WEREN'T A PART OF PRIOR (MAYBE IDK)
            Self::LooseBracket { cell, .. } => idx == *cell,
            // to not confuse the user by seperating text from code
            // TODO: it is also intuitive, but wasteful to fence all cells because a single loop is written `[ like - this]`
            Self::Text { .. } => true,
        }
    }
}

/// Represents a matched bracket block.
#[derive(Debug, Clone, PartialEq, Default)]
struct Block<'a> {
    operations: Vec<Operation<'a>>,
    is_dynamic: bool,
}

impl<'a> Block<'a> {
    /// Takes in the source of a matched bracket block.
    /// The slice should start and end with '[' and ']' respectively.
    fn new(src: &str) -> Block {
        debug_assert!(src.starts_with('['));
        debug_assert!(src.ends_with(']'));
        
        // remove the brackets
        let content = &src[1..src.len()-1];

        let (operations, is_dynamic) = parse_operations(content);

        Block {
            operations,
            is_dynamic,
        }
    }

    /// Returns true if the section is dynamic, aka does it offset the tape pointer.
    fn is_dynamic(&self) -> bool {
        self.is_dynamic
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
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn optimiser_basic_parsing_operations() {
        let (ops, is_dyn) = parse_operations(">>+<->.");
        assert!(is_dyn);
        assert_eq!(ops, vec![
            Operation::Offset { cell: 2, recurence: 1 },
            Operation::Offset { cell: 1, recurence: -1 },
            Operation::InOut { cell: 2, operator: '.' }
        ]);

        let (ops, is_dyn) = parse_operations(">>,[---++]++--<<");
        assert!(!is_dyn);
        assert_matches!(ops[0], Operation::InOut { cell: 2, operator: ',' });
        assert_matches!(ops[1], Operation::Block { cell: 2, .. });

        let (ops, is_dyn) = parse_operations(">>,[--+]++--<<");
        assert!(!is_dyn);
        assert_matches!(ops[0], Operation::InOut { cell: 2, operator: ',' });
        assert_matches!(ops[1], Operation::Block { cell: 2, .. });

        let (ops, is_dyn) = parse_operations("did \n you know +>.[<atmic bomb[++-]");
        assert!(!is_dyn);
        assert_matches!(ops[0], Operation::Text { src: "did \n you know " });
        assert_matches!(ops[1], Operation::Offset { cell: 0, recurence: 1 });
        assert_matches!(ops[2], Operation::InOut { cell: 1, operator: '.' });
        assert_matches!(ops[3], Operation::LooseBracket { cell: 1, operator: '[' });
        assert_matches!(ops[4], Operation::Text { src: "atmic bomb" });
        assert_matches!(ops[5], Operation::Block { cell: 0, .. });
    }
}

