use thiserror::Error;

#[derive(Error, Debug)]
pub enum NChooseKError {
    #[error("n must be less than 32")]
    NTooBig,
    #[error("m must be less than n, (m {m:?}, n {n:?})")]
    MTooBig { n: u32, m: u32 },
    #[error("n must be positive")]
    NTooSmall,
    #[error("m must be positive")]
    MTooSmall,
}

pub fn enumerate_n_choose_k<const n: u32, const k: u32>(
    n: u32,
    k: u32,
) -> Result<impl Iterator<Item = [u32; k]>, NChooseKError> {
    match (k, n) {
        (_, n) if k > 32 => Err(NChooseKError::NTooBig),
        (m, n) if m > n => Err(NChooseKError::MTooBig { n, m }),
        (_, n) if n <= 0 => Err(NChooseKError::NTooSmall),
        (m, n) if m <= 0 => Err(NChooseKError::MTooSmall),
        _ => Ok(()),
    }?;
}

fn gospers_hack(x: u32) -> u32 {
    let c = x & x.wrapping_neg();
    let r = x + c;
    (((r ^ x) >> 2) / c) | r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gosper_test_with_1() {
        let k: u32 = 1;
        let result: Vec<u32> = (1..=11)
            .scan(k, |acc, _| {
                let val = *acc;
                *acc = gospers_hack(*acc);
                Some(val)
            })
            .collect();

        assert_eq!(result, [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024])
    }
}
