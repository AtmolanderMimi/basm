use crate::optimiser::{self, Operation};

// TODO: This does not take into account the forced offset of dynamic blocks
// (assumes that the pointer always ends at 0)
/// Reorders operations as much as fencing allows to minimise the amount of tape pointer movement.
pub fn reorder_operations(ops: &mut Vec<Operation>) {
    let mut ops_tracker = OperationProcessTracker::new(ops);

    // moves around all operations once (the once part is handled by OperationProcessTracker) to minimise distance
    while let Some(unprocessed_index) = ops_tracker.get_unprocessed_index() {
        let validity_range = optimiser::operation_validity_range(
            &ops_tracker.operations(),
            unprocessed_index,
        );
        
        let operation = ops_tracker.remove(unprocessed_index);

        // we fold to find the pair `(index, lost_distance)` with the lowest lost_distance.
        let (best_index, _) = validity_range.fold((unprocessed_index, usize::MAX), |(best_index, best_distance), i| {
            let current_distance = calculate_lost_distance(&operation, ops_tracker.operations(), i);
            if current_distance < best_distance {
                (i, current_distance)
            } else {
                (best_index, best_distance)
            }
        });

        ops_tracker.insert(best_index, operation);
        ops_tracker.mark_processed(best_index);
    }

    // make sure it's recursive
    ops.iter_mut()
        .filter_map(|op| if let Operation::Block { block, .. } = op {
            Some(block)
        } else {
            None
        })
        .for_each(|block| block.apply_optimisation(reorder_operations));
}

/// Calculates the extra distance that would need to be traveled to reach the operation if it were to be inserted at `idx`.
/// Ex: positions with positions `[0, 5, 4]` adding an operation with position 2 at 1 would have a lost distance of 0,
/// but inserting at 2 would have a lost distance of 5 because we would need to move `-3` and `+2` rather than the `-1` prior.
/// This assumes that the operations end and start at 0.
fn calculate_lost_distance(op: &Operation, ops: &[Operation], idx: usize) -> usize {
    // finds the cell position of the first item before it that has a cell position
    // or default to 0
    let position_before = ops[..idx].iter()
        .rfind(|op| op.cell_position().is_some())
        .map(|op| op.cell_position().unwrap())
        .unwrap_or(0);

    // same but for after
    let position_after = ops[idx..].iter()
        .find(|op| op.cell_position().is_some())
        .map(|op| op.cell_position().unwrap())
        .unwrap_or(0);

    let distance_before_insert = position_before.abs_diff(position_after);

    let Some(self_position) = op.cell_position() else { return 0; };
    let diff_self_before = self_position-position_before;
    let diff_after_self = position_after-self_position;
    let distance_after_insert = (diff_self_before.abs() + diff_after_self.abs()) as usize;

    // distance_after_insert cannot be smaller than distance_before_insert
    // any excess is our "lost distance"
    let lost_distance = distance_after_insert - distance_before_insert;
    lost_distance
}

/// An struct to abstract over the "processed" book keeping that we need to avoid reprocessing the same operation twice.
/// Normally i would have programmed this by wrapping operations in enums with variants `Unprocessed` and `Processed`,
/// but that would not work since we need the operation to be in a contiguous array in order to call functions like
/// `operation_validity_range` on it. So we seperate the "processed" state to another array matching our operations array.
#[derive(Debug, PartialEq)]
struct OperationProcessTracker<'a, 'b> {
    operations: &'a mut Vec<Operation<'b>>,
    // a vector of the same size as operations tracking which operations have been processed
    // we need this so we can keep a operations as a contiguous section of memory
    processed: Vec<bool>,
}

impl<'a, 'b> OperationProcessTracker<'a, 'b> {
    fn new(operations: &'a mut Vec<Operation<'b>>) -> OperationProcessTracker<'a, 'b> {
        OperationProcessTracker {
            processed: Vec::from_iter((0..operations.len()).map(|_| false)),
            operations,
        }
    }

    fn get_unprocessed_index(&self) -> Option<usize> {
        self.processed.iter()
            .enumerate()
            .find_map(|(i, processed)| if !processed { Some(i) } else { None })
    }

    fn operations(&self) -> &[Operation<'b>] {
        &self.operations
    }

    fn remove(&mut self, idx: usize) -> Operation<'b> {
        self.processed.remove(idx);
        self.operations.remove(idx)
    }

    fn insert(&mut self, idx: usize, op: Operation<'b>) {
        self.processed.insert(idx, false);
        self.operations.insert(idx, op);
    }

    fn mark_processed(&mut self, idx: usize) {
        self.processed.get_mut(idx).map(|processed| { *processed = true; processed });
    }
}

#[cfg(test)]
mod tests {
    use crate::optimiser;

    use super::*;

    #[test]
    fn reorder_operations_optimisation() {
        // does nothing when there is nothing to do
        let (mut ops, _) = optimiser::parse_operations("+++[-]--");
        reorder_operations(&mut ops);
        let string = optimiser::operations_to_brainfuck(&ops);
        assert_eq!(string, "+++[-]--");

        // simple usecase
        let (mut ops, _) = optimiser::parse_operations("+>>-<+");
        reorder_operations(&mut ops);
        let string = optimiser::operations_to_brainfuck(&ops);
        assert_eq!(string, "+>+>-");

        // other simple usecase
        let (mut ops, _) = optimiser::parse_operations("+>>-<<+>>-");
        assert!(ops[0].can_swap(&ops[1]));
        reorder_operations(&mut ops);
        let string = optimiser::operations_to_brainfuck(&ops);
        assert_eq!(string, "++>>--");

        // recursive
        let (mut ops, _) = optimiser::parse_operations("[+>>-<<+>>-<<]");
        reorder_operations(&mut ops);
        let string = optimiser::operations_to_brainfuck(&ops);
        assert_eq!(string, "[++>>--<<]");

        // stops at fences
        let (mut ops, _) = optimiser::parse_operations(">++<[-]+");
        reorder_operations(&mut ops);
        let string = optimiser::operations_to_brainfuck(&ops);
        assert_eq!(string, "[-]+>++");
    }
}