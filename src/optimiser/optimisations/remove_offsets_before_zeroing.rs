use std::mem;

use crate::optimiser::{block::Block, Operation};

/// Removes offsets which will be negated by a block zeroing the cell before it's read.
pub fn remove_offsets_before_zeroing(ops: &mut Vec<Operation>) {
    let zeroing_block_indexes = ops.iter()
        .enumerate()
        .filter_map(|(i, op)| if let Operation::Block { block, ..} = op {
            if block.is_zeroing() {
                Some(i)
            } else {
                None
            }
        } else {
            None
        }).collect::<Vec<_>>();

    for idx in zeroing_block_indexes {
        // we get the cell being zeroed
        let Operation::Block { cell: zeroed_cell, .. } = ops[idx] else { panic!() };

        // we search right to left to find the range where useless offsets may be hiding
        // we just need to find the start of that range (we already know the end since it's the block)
        // to find the start we simply find the first operation that fences the cell we zero
        // TODO: It would be best to use operation_validity_range, but that is simply for operations and not for cells
        let range_start = ops[..idx].iter()
            .enumerate()
            .rfind(|(_, op)| op.fences_cell(zeroed_cell))
            .map(|(j, _)| j+1)
            .unwrap_or(0);

        let cell_validity_range = range_start..idx;

        // we remove all recurence from offsets in that range (we will remove 0 recurrence offsets at the end)
        for jdx in cell_validity_range {
            let Operation::Offset { cell, recurrence: recurence } = &mut ops[jdx] else { continue };
            if *cell != zeroed_cell {
                continue
            }

            *recurence = 0;
        }
    }

    // removing all the offsets we set a 0 recurrence
    let tmp_ops = mem::take(ops);
    *ops = tmp_ops.into_iter()
        .filter(|op| if let Operation::Offset { recurrence: 0, .. } = op {
            false
        } else {
            true
        }).collect();

    // do it recusively
    ops.iter_mut()
        .filter_map(|op| if let Operation::Block { block, .. } = op {
            Some(block)
        } else {
            None
        })
        .for_each(|block| block.apply_optimisation(remove_offsets_before_zeroing));
}

impl<'a> Block<'a> {
    /// Returns `true` if the block zeroes its cell.
    /// Aka if it is `[-]` or `[+]`.
    fn is_zeroing(&self) -> bool {
        let mut non_text_operations = self.operations.iter()
            .filter(|op| if let Operation::Text { .. } = op { false } else { true });

        if non_text_operations.clone().count() != 1 {
            return false;
        }

        if self.is_dynamic() {
            return false;
        }

        // we know from the check above that the the lenght of this iterator is of 1.
        let Operation::Offset { cell: 0, recurrence: 1 | -1 } = non_text_operations.next().unwrap() else {
            return false;
        };

        true
    }
}

#[cfg(test)]
mod tests {
    use crate::optimiser::{operations_to_brainfuck, parse_operations};

    use super::*;

    #[test]
    fn block_is_zeroing() {
        // basic negative zero
        let ops = parse_operations("[-]").0;
        let Operation::Block { block, .. } = &ops[0] else { panic!() };
        assert!(block.is_zeroing());

        // basic positive zero
        let ops = parse_operations("[+]").0;
        let Operation::Block { block, .. } = &ops[0] else { panic!() };
        assert!(block.is_zeroing());

        // not zero because dynamic
        let ops = parse_operations("[->]").0;
        let Operation::Block { block, .. } = &ops[0] else { panic!() };
        assert!(!block.is_zeroing());

        // not zero because not on cell 0
        let ops = parse_operations("[<->]").0;
        let Operation::Block { block, .. } = &ops[0] else { panic!() };
        assert!(!block.is_zeroing());

        // text does not block detection
        let ops = parse_operations("[ i love cats - :3c]").0;
        let Operation::Block { block, .. } = &ops[0] else { panic!() };
        assert!(block.is_zeroing());

        let ops = parse_operations("[< omg - figha > ]").0;
        let Operation::Block { block, .. } = &ops[0] else { panic!() };
        assert!(!block.is_zeroing());
    }

    #[test]
    fn remove_offsets_before_zeroing_works() {
        // shirple example
        let mut ops = parse_operations("+++++[-]").0;
        remove_offsets_before_zeroing(&mut ops);
        assert_eq!(operations_to_brainfuck(&ops), "[-]");

        // other operations don't mess with this
        let mut ops = parse_operations("++>,- hi <+++[+]").0;
        remove_offsets_before_zeroing(&mut ops);
        assert_eq!(operations_to_brainfuck(&ops), ">,- hi <[+]");

        // longshot
        let mut ops = parse_operations(">++>,+++<<[-]>[+]").0;
        remove_offsets_before_zeroing(&mut ops);
        assert_eq!(operations_to_brainfuck(&ops), ">>,+++<<[-]>[+]");

        // does nothing when there is nothing to do
        let mut ops = parse_operations(">++>,<+++<[-]").0;
        remove_offsets_before_zeroing(&mut ops);
        assert_eq!(operations_to_brainfuck(&ops), ">++>,<+++<[-]");

        // dynamic blocks
        let mut ops = parse_operations("+++[<]++[-]").0;
        remove_offsets_before_zeroing(&mut ops);
        assert_eq!(operations_to_brainfuck(&ops), "+++[<][-]");

        // recursive
        let mut ops = parse_operations("[++>,- hi <+++[+]]").0;
        remove_offsets_before_zeroing(&mut ops);
        assert_eq!(operations_to_brainfuck(&ops), "[>,- hi <[+]]");
    }
}