use std::collections::HashSet;

use super::Operation;

/// Represents a matched bracket block.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Block<'a> {
    operations: Vec<Operation<'a>>,
    dynamic_endpoint: Option<isize>,
}

impl<'a> Block<'a> {
    /// Takes in the source of a matched bracket block.
    /// The slice should start and end with '[' and ']' respectively.
    pub fn new(src: &str) -> Block {
        debug_assert!(src.starts_with('['));
        debug_assert!(src.ends_with(']'));
        
        // remove the brackets
        let content = &src[1..src.len()-1];

        let (operations, end_point) = super::parse_operations(content);

        Block {
            operations,
            dynamic_endpoint: end_point,
        }
    }

    /// Returns true if the section is dynamic, aka does it offset the tape pointer.
    pub fn is_dynamic(&self) -> bool {
        self.dynamic_endpoint.is_some()
    }

    pub fn fences_cell(&self, idx: isize) -> bool {
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

    pub fn modified_cells(&self) -> HashSet<isize> {
        self.operations.iter().fold(HashSet::new(), |mut buf, op| {
            for cell in op.modified_cells() {
                buf.insert(cell);
            };

            buf
        })
    }

    pub fn to_brainfuck(&self) -> String {
        let mut buf = super::operations_to_brainfuck(&self.operations);
        
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

    /// Applies the specified optimisation to `Operation` contained in this block.
    pub fn apply_optimisation(&mut self, opti: impl FnOnce(&mut Vec<Operation>)) {
        opti(&mut self.operations)
    }
}

#[cfg(test)] 
mod tests {
    use crate::optimiser::parse_operations;

    use super::*;

    #[test]
    fn block_fences() {                     // 0   3 2
        let block = Block::new("[>>>,<[-]<<]");
        assert!(!block.is_dynamic());
        assert!(block.fences_cell(0));
        assert!(!block.fences_cell(1));
        assert!(block.fences_cell(2));
        assert!(block.fences_cell(3));

        // offset in context
                                                          // 2   5  4  
        let block = &parse_operations(">>[>>>,<[-]<<]").0[0];
        assert!(block.fences_cell(2));
        assert!(!block.fences_cell(3));
        assert!(block.fences_cell(4));
        assert!(block.fences_cell(5));
    }

    #[test]
    fn block_modifies() {                   // 0   3  2
        let block = Block::new("[>>>+<[-]<<]");
        assert!(!block.is_dynamic());
        assert!(!block.modified_cells().contains(&0));
        assert!(!block.modified_cells().contains(&1));
        assert!(block.modified_cells().contains(&2));
        assert!(block.modified_cells().contains(&3));

        // offset in context
                                                          // 2   5  4  
        let block = &parse_operations(">>[>>>+<[-]<<]").0[0];
        assert!(!block.modified_cells().contains(&2));
        assert!(!block.modified_cells().contains(&3));
        assert!(block.modified_cells().contains(&4));
        assert!(block.modified_cells().contains(&5));
    }
}
