#[derive(Debug, PartialEq)]
enum PermutationsError {
    ElementIsBiggest,
    ElementNotFound,
}

struct Permutations<const K: usize> {
    initial_state: [u8; K],
    current_permutation: [u8; K],
}

impl<const K: usize> Permutations<K> {
    pub fn new(array: [u8; K]) -> Permutations<K> {
        let mut array = array.clone();
        array.sort();
        Self {
            initial_state: array,
            current_permutation: array.clone(),
        }
    }

    fn find_next_bigger_neighbor(&self, number: u8) -> Result<u8, PermutationsError> {
        let result = self.current_permutation.binary_search(&number);
        let Ok(i) = result else {
            return Err(PermutationsError::ElementNotFound);
        };

        self.initial_state
            .get(i + 1)
            .map_or(Err(PermutationsError::ElementIsBiggest), |x| Ok(*x))
    }
}

impl<const K: usize> Iterator for Permutations<K> {
    type Item = [u8; K];

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.current_permutation)
    }
}

#[cfg(test)]
mod test_permutations {
    use super::*;

    #[test]
    fn test_permutations_find_next() {
        let a = [5, 3, 2, 4];
        let p = Permutations::new(a);
        assert_eq!(p.find_next_bigger_neighbor(2).unwrap(), 3);
        assert_eq!(p.find_next_bigger_neighbor(4).unwrap(), 5);
        assert_eq!(
            p.find_next_bigger_neighbor(5).unwrap_err(),
            PermutationsError::ElementIsBiggest
        );
        assert_eq!(
            p.find_next_bigger_neighbor(17).unwrap_err(),
            PermutationsError::ElementNotFound
        );
    }
}
