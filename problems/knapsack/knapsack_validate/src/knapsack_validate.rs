use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use thiserror::Error;

type Score = u64;
type Input = (u64, Vec<(u64, u64)>);

pub const SAMPLE_NUM_ITEMS: usize = 10;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("No item with index {idx}. Note: indices must be in [0, {num_items}) range.")]
    UnknownItem { idx: usize, num_items: usize },

    #[error("Index {idx} appears more than once. Indices must be unique.")]
    RepeatingItem { idx: usize },

    #[error("Total weight of {total_weight} exceeds capacity {capacity}.")]
    OverCapacity { total_weight: u64, capacity: u64 },
}

pub fn pre_validate(solution: &[usize], num_items: usize) -> Result<(), Error> {
    solution
        .iter()
        .try_fold(HashSet::new(), |mut items, &idx| {
            if idx >= num_items {
                return Err(Error::UnknownItem { idx, num_items });
            }
            if items.insert(idx) {
                Ok(items)
            } else {
                Err(Error::RepeatingItem { idx })
            }
        })
        .map(|_| ())
}

/// Assumes `solution` was passed through `pre_validate`.
pub fn validate(input: &Input, solution: &[usize]) -> Result<Score, Error> {
    let (capacity, items) = input;
    let (total_weight, total_value) = solution
        .iter()
        .map(|&idx| items[idx])
        .fold((0, 0), |(total_weight, total_value), (weight, value)| {
            (total_weight + weight, total_value + value)
        });
    if total_weight > *capacity {
        return Err(Error::OverCapacity {
            total_weight,
            capacity: *capacity,
        });
    }
    Ok(total_value)
}

pub fn parse_binary_input(path: &Path) -> anyhow::Result<Input> {
    Ok(ciborium::from_reader(BufReader::new(File::open(path)?))?)
}

pub fn parse_sample_input(input: &str) -> anyhow::Result<Input> {
    let mut lines = input.lines();
    let capacity = lines
        .next()
        .map(|n| n.parse())
        .expect("Sample input is not empty")?;
    lines.next();
    let items = lines
        .filter_map(|item_line| item_line.split_once(' '))
        .map(|(weight, value)| anyhow::Ok((weight.parse()?, value.parse()?)))
        .collect::<Result<_, _>>()?;
    Ok((capacity, items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_pre_validate_ok() {
        let solution = vec![0, 1, 2];
        let num_items = 3;
        assert!(pre_validate(&solution, num_items).is_ok());
    }

    #[test]
    fn test_pre_validate_unknown_item() {
        let solution = vec![0, 1, 3];
        let num_items = 3;
        let err = pre_validate(&solution, num_items).unwrap_err();
        assert_eq!(err, Error::UnknownItem { idx: 3, num_items });
    }

    #[test]
    fn test_pre_validate_repeating_item() {
        let solution = vec![0, 1, 0];
        let num_items = 3;
        let err = pre_validate(&solution, num_items).unwrap_err();
        assert_eq!(err, Error::RepeatingItem { idx: 0 });
    }

    #[test]
    fn test_validate_ok() {
        let input = (10, vec![(2, 3), (3, 4), (4, 5)]);
        let solution = vec![0, 1]; // items (2,3) and (3,4) -> total_weight = 5, total_value = 7
        assert_eq!(validate(&input, &solution).unwrap(), 7);
    }

    #[test]
    fn test_validate_at_capacity() {
        let input = (5, vec![(2, 3), (3, 4), (4, 5)]);
        let solution = vec![0, 1]; // items (2,3) and (3,4) -> total_weight = 5, total_value = 7
        assert_eq!(validate(&input, &solution).unwrap(), 7);
    }

    #[test]
    fn test_validate_over_capacity() {
        let input = (4, vec![(2, 3), (3, 4), (4, 5)]);
        let solution = vec![0, 1]; // items (2,3) and (3,4) -> total_weight = 5, total_value = 7
        let err = validate(&input, &solution).unwrap_err();
        assert_eq!(
            err,
            Error::OverCapacity {
                total_weight: 5,
                capacity: 4
            }
        );
    }

    #[test]
    fn test_parse_sample_input_ok() -> Result<()> {
        let input_str = "10\n\n2 3\n3 4\n4 5";
        let (capacity, items) = parse_sample_input(input_str)?;
        assert_eq!(capacity, 10);
        assert_eq!(items, vec![(2, 3), (3, 4), (4, 5)]);
        Ok(())
    }

    #[test]
    fn test_parse_sample_input_malformed_capacity() {
        let input_str = "abc\n\n2 3";
        let err = parse_sample_input(input_str).unwrap_err();
        assert!(err.to_string().contains("invalid digit"));
    }

    #[test]
    fn test_parse_sample_input_malformed_item() {
        let input_str = "10\n\n2 3\n3 four";
        let err = parse_sample_input(input_str).unwrap_err();
        assert!(err.to_string().contains("invalid digit"));
    }
}
