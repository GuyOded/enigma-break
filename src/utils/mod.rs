use thiserror::Error;
mod permutations;

#[derive(Error, Debug)]
pub enum NChooseKError {
    #[error("n must be less than 32")]
    NTooBig,
    #[error("m must be less than n, (k {k:?}, n {n:?})")]
    KTooBig { n: usize, k: usize },
    #[error("n must be positive")]
    NTooSmall,
    #[error("m must be positive")]
    KTooSmall,
}

pub fn enumerate_n_choose_k<const N: usize, const K: usize>()
-> Result<impl Iterator<Item = [u8; K]>, NChooseKError> {
    if N > usize::BITS.try_into().unwrap() {
        return Err(NChooseKError::NTooBig);
    }
    if K > N {
        return Err(NChooseKError::KTooBig { n: N, k: K });
    }
    if N <= 0 {
        return Err(NChooseKError::NTooSmall);
    }
    if K <= 0 {
        return Err(NChooseKError::KTooSmall);
    }

    let last_permutation: usize = ((1 << K) - 1) << N - K;
    let first_permutation: usize = (1 << K) - 1;

    let mut current_permutation = first_permutation;
    let mut last_element_reached = false;
    Ok(std::iter::from_fn(move || {
        if current_permutation == last_permutation {
            if last_element_reached {
                None
            } else {
                last_element_reached = true;
                Some(positions_of_bits::<N, K>(last_permutation))
            }
        } else {
            let next = positions_of_bits::<N, K>(current_permutation);
            current_permutation = gospers_hack(current_permutation);
            Some(next)
        }
    }))
}



fn gospers_hack(x: usize) -> usize {
    let c = x & x.wrapping_neg();
    let r = x + c;
    (((r ^ x) >> 2) / c) | r
}

fn positions_of_bits<const N: usize, const K: usize>(number: usize) -> [u8; K] {
    let mut result: [u8; K] = [0; K];
    let mut position = 0;

    for i in 0..N {
        if number & (1 << i) != 0 {
            result[position] = (i + 1).try_into().unwrap();
            position += 1;
        }
    }

    result
}

#[cfg(test)]
mod test_gospers_hack {
    use super::*;

    #[test]
    fn test_gospers_hack_with_1() {
        let k: usize = 1;
        let result: Vec<usize> = (1..=11)
            .scan(k, |acc, _| {
                let val = *acc;
                *acc = gospers_hack(*acc);
                Some(val)
            })
            .collect();

        assert_eq!(result, [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024])
    }
}

