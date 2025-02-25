use std::mem;

use crate::optimiser::{self, Operation};

/// Merges offset operations togheter as much as fencing allows.
pub fn merge_offsets<'a, 'b>(ops: &'a mut Vec<Operation<'b>>) {
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
        let range = optimiser::operation_validity_range(ops, offset_index);
        
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

    // do this recursively for all blocks
    ops.iter_mut()
        .filter_map(|op| if let Operation::Block { block, .. } = op {
            Some(block)
        } else {
            None
        })
        .for_each(|block| block.apply_optimisation(merge_offsets));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merging_offset_optimisation() {
        // does nothing when there is nothing to do
        let (mut ops, _) = optimiser::parse_operations("[>-  the harsh advice of those who love you rots within this hell<<]");
        merge_offsets(&mut ops);
        let string = optimiser::operations_to_brainfuck(&ops);
        assert_eq!(string, "[>-  the harsh advice of those who love you rots within this hell<<]");

        // does nothing when there is nothing to do
        let (mut ops, _) = optimiser::parse_operations("+++[-]--");
        assert_eq!(optimiser::operation_validity_range(&ops, 0), 0..1);
        merge_offsets(&mut ops);
        let string = optimiser::operations_to_brainfuck(&ops);
        assert_eq!(string, "+++[-]--");

        // does something when it needs to
        let (mut ops, _) = optimiser::parse_operations("+++>[-]<--");
        assert_eq!(optimiser::operation_validity_range(&ops, 0), 0..3);
        merge_offsets(&mut ops);
        let string = optimiser::operations_to_brainfuck(&ops);
        assert_eq!(string, "+>[-]");

        let (mut ops, _) = optimiser::parse_operations("++>>[-][-<<+>>]<<++");
        merge_offsets(&mut ops);
        let string = optimiser::operations_to_brainfuck(&ops);
        assert_eq!(string, "++++>>[-][-<<+>>]");
    }
}
