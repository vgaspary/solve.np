#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rank(pub usize);

pub struct RankedBy<I, F, K> {
    iter: I,
    f: F,
    rank: Rank,
    prev_key: Option<K>,
}

impl<I, F, K> Iterator for RankedBy<I, F, K>
where
    I: Iterator,
    F: FnMut(&I::Item) -> K,
    K: Ord + Eq,
{
    type Item = (Rank, I::Item);
    fn next(&mut self) -> Option<Self::Item> {
        let Some(item) = self.iter.next() else {
            return None;
        };
        let key = (self.f)(&item);
        if self.prev_key.as_ref() != Some(&key) {
            self.rank = Rank(self.rank.0 + 1);
            self.prev_key = Some(key);
        }
        Some((self.rank, item))
    }
}

pub trait RankedExt: Iterator {
    fn rank_by<F, K>(self, f: F) -> RankedBy<impl Iterator<Item = Self::Item>, F, K>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> K,
        K: Ord + Eq;
}

impl<I> RankedExt for I
where
    I: Iterator,
{
    fn rank_by<F, K>(self, mut f: F) -> RankedBy<impl Iterator<Item = Self::Item>, F, K>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> K,
        K: Ord + Eq,
    {
        let mut v = Vec::from_iter(self);
        v.sort_by_key(&mut f);
        RankedBy {
            iter: v.into_iter(),
            f,
            rank: Rank(0),
            prev_key: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_by_unique_keys() {
        let data = vec![3, 7, 4, 1, 5, 9, 2, 6];
        let ranked: Vec<(Rank, i32)> = data.into_iter().rank_by(|&x| x).collect();
        let expected = vec![
            (Rank(1), 1),
            (Rank(2), 2),
            (Rank(3), 3),
            (Rank(4), 4),
            (Rank(5), 5),
            (Rank(6), 6),
            (Rank(7), 7),
            (Rank(8), 9),
        ];
        assert_eq!(ranked, expected);
    }

    #[test]
    fn test_rank_by_duplicate_keys() {
        let data = vec![10, 20, 10, 30, 20, 10];
        let ranked: Vec<(Rank, i32)> = data.into_iter().rank_by(|&x| x).collect();
        let expected = vec![
            (Rank(1), 10),
            (Rank(1), 10),
            (Rank(1), 10),
            (Rank(2), 20),
            (Rank(2), 20),
            (Rank(3), 30),
        ];
        assert_eq!(ranked, expected);
    }

    #[test]
    fn test_rank_by_empty_iterator() {
        let ranked: Vec<(Rank, i32)> = std::iter::empty().rank_by(|&x| x).collect();
        assert!(ranked.is_empty());
    }

    #[test]
    fn test_rank_by_pre_sorted_data() {
        let data = vec![1, 2, 3, 4, 5];
        let ranked: Vec<(Rank, i32)> = data.into_iter().rank_by(|&x| x).collect();
        let expected = vec![
            (Rank(1), 1),
            (Rank(2), 2),
            (Rank(3), 3),
            (Rank(4), 4),
            (Rank(5), 5),
        ];
        assert_eq!(ranked, expected);
    }

    #[test]
    fn test_rank_by_reverse_sorted_data() {
        let data = vec![5, 4, 3, 2, 1];
        let ranked: Vec<(Rank, i32)> = data.into_iter().rank_by(|&x| x).collect();
        let expected = vec![
            (Rank(1), 1),
            (Rank(2), 2),
            (Rank(3), 3),
            (Rank(4), 4),
            (Rank(5), 5),
        ];
        assert_eq!(ranked, expected);
    }

    #[test]
    fn test_rank_by_complex_key() {
        #[derive(Debug, PartialEq)]
        struct Item {
            id: usize,
            value: i32,
        }

        let data = vec![
            Item { id: 1, value: 30 },
            Item { id: 2, value: 10 },
            Item { id: 3, value: 20 },
            Item { id: 4, value: 10 },
            Item { id: 5, value: 30 },
        ];

        let ranked: Vec<(Rank, Item)> = data.into_iter().rank_by(|item| item.value).collect();
        let expected = vec![
            (Rank(1), Item { id: 2, value: 10 }),
            (Rank(1), Item { id: 4, value: 10 }),
            (Rank(2), Item { id: 3, value: 20 }),
            (Rank(3), Item { id: 1, value: 30 }),
            (Rank(3), Item { id: 5, value: 30 }),
        ];
        assert_eq!(ranked, expected);
    }
}
