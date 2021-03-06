mod cache;
mod gemm;
mod microkernel;
mod pack_operands;

pub use gemm::gemm;
pub use gemm::gemm_req;

pub(crate) struct Ptr<T>(*mut T);

impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Ptr<T> {}

unsafe impl<T> Send for Ptr<T> {}
unsafe impl<T> Sync for Ptr<T> {}

impl<T> Ptr<T> {
    #[inline(always)]
    pub fn wrapping_offset(self, offset: isize) -> Self {
        Ptr::<T>(self.0.wrapping_offset(offset))
    }
    #[inline(always)]
    pub fn wrapping_add(self, offset: usize) -> Self {
        Ptr::<T>(self.0.wrapping_add(offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemm() {
        use dyn_stack::{uninit_mem_in_global, DynStack, ReborrowMut};

        let mut mnks = vec![];
        mnks.push((1, 1, 2));
        mnks.push((4, 4, 4));
        mnks.push((256, 256, 256));
        mnks.push((1024, 1024, 1));
        mnks.push((4096, 4096, 1));

        let n_threads = rayon::current_num_threads();

        for (m, n, k) in mnks {
            let a_vec: Vec<f64> = (0..(m * k)).map(|_| rand::random()).collect();
            let b_vec: Vec<f64> = (0..(k * n)).map(|_| rand::random()).collect();
            let mut c_vec = vec![0.0; m * n];
            let mut d_vec = vec![0.0; m * n];

            let mut mem = uninit_mem_in_global(gemm::gemm_req::<f64>(m, n, k, n_threads).unwrap());
            let mut stack = DynStack::new(&mut mem);
            unsafe {
                gemm::gemm(
                    m,
                    n,
                    k,
                    c_vec.as_mut_ptr(),
                    m as isize,
                    1,
                    true,
                    a_vec.as_ptr(),
                    m as isize,
                    1,
                    b_vec.as_ptr(),
                    k as isize,
                    1,
                    0.0,
                    1.0,
                    n_threads,
                    stack.rb_mut(),
                );

                gemm::gemm_correct(
                    m,
                    n,
                    k,
                    d_vec.as_mut_ptr(),
                    m as isize,
                    1,
                    true,
                    a_vec.as_ptr(),
                    m as isize,
                    1,
                    b_vec.as_ptr(),
                    k as isize,
                    1,
                    0.0,
                    1.0,
                    n_threads,
                    stack.rb_mut(),
                );
            }
            for (c, d) in c_vec.iter().zip(d_vec.iter()) {
                assert_approx_eq::assert_approx_eq!(c, d);
            }
        }
    }
}
