use std::num::Wrapping;

pub struct OrangeyCtx {
    state: u128,
    inc: u128,
}

impl OrangeyCtx {
    pub fn new() -> Self {
        OrangeyCtx {
            state: 0xce84809586cf8d1f17e1e9805a1b4141,
            inc: 0xb0a3e85a992afe5a280af6fdeecf029f,
        }
    }

    /// Jumps `delta` values ahead in the rng stream
    ///
    /// # Examples
    ///
    /// ```
    /// use orangey::OrangeyCtx;
    ///
    /// let mut orangey_ctx = OrangeyCtx::new();
    /// orangey_ctx.skip(32);
    /// println!("{}", orangey_ctx.rand());
    /// // 2947149625353530425
    /// ```
    pub fn skip(&mut self, delta: u128) {
        self.state = Self::advance(self.state, delta, Self::MUL, self.inc);
    }

    /// Peeks at the `delta`-th value ahead in the rng stream. Unlike `.skip()`, this doesn't modify the rng state
    ///
    /// # Examples
    ///
    /// ```
    /// use orangey::OrangeyCtx;
    ///
    /// let mut orangey_ctx = OrangeyCtx::new();
    /// println!("{}", orangey_ctx.peek(32));
    /// // 2947149625353530425, the 32nd number after rand()
    /// println!("{}", orangey_ctx.rand());
    /// // 18017628057179154148, as if peek were never called
    /// ```
    pub fn peek(&mut self, delta: u128) -> u64 {
        Self::output(
            Self::advance(
                self.state,
                delta + 1,
                Self::MUL,
                self.inc,
            )
        )
    }

    /// Seeds the generator with new initial state and sequence values
    ///
    /// # Examples
    ///
    /// ```
    /// use orangey::OrangeyCtx;
    ///
    /// let mut orangey_ctx = OrangeyCtx::new();
    /// orangey_ctx.srand(0, 0);
    /// println!("{}", orangey_ctx.rand());
    /// // 12455822396014146421
    /// ```
    pub fn srand(&mut self, initstate: u128, initseq: u128) {
        self.state = 0;
        self.inc = (initseq << 1) | 1;
        self.step();
        self.state += initstate;
        self.step();
    }

    /// Runs the generator and return a random number
    ///
    /// # Examples
    ///
    /// ```
    /// use orangey::OrangeyCtx;
    ///
    /// let mut orangey_ctx = OrangeyCtx::new();
    /// println!("{}", orangey_ctx.rand());
    /// // 18017628057179154148
    /// ```
    pub fn rand(&mut self) -> u64 {
        self.step();
        Self::output(self.state)
    }

    /// Generates a number in the range [min, max)
    pub fn rand_range(&mut self, min: u64, max: u64) -> u64 {
        let (min, max) = (min.min(max), min.max(max));
        let range = max - min;
        if max == min {
            return max;
        }
        if range.count_ones() == 1 {
            return (self.rand() & (range - 1)) + min;
        }
        let limit = range.wrapping_neg() % range;
        let mut r = 0;
        for i in 0.. {
            r = self.peek(i);
            if r >= limit {
                break;
            }
        }
        r %= range;
        r + min
    }

    /// Generates a float in the range [0, 1) with uniform density.
    /// This does not have an equal chance of hitting every float
    /// in range, but you usually don't want that.
    ///
    /// For those who need that functionality, use .all_doubles()
    pub fn uniform_double(&mut self) -> f64 {
        const MASK: u64 = 0x000FFFFFFFFFFFFF;
        const S_EXP: u64 = 0x3FF0000000000000;
        let mut i = self.rand();
        i &= MASK;
        i |= S_EXP;
        f64::from_bits(i) - 1.0
    }

    /// Has an equal chance of generating any representable float in the range [0, 1).
    /// This is biased towards lower values.
    ///
    /// For those who need an even distribution of float values, use .uniform_double()
    pub fn all_doubles(&mut self) -> f64 {
        let mut exponent = -64;
        let mut significand;
        loop {
            exponent -= 64;
            if exponent < -1074 {
                return 0.0;
            }
            significand = self.rand();
            if significand != 0 {
                break;
            }
        }
        let shift = significand.leading_zeros();
        if shift != 0 {
            exponent -= shift as i32;
            significand <<= shift;
            significand |= self.peek(1) >> (64 - shift);
        }
        significand |= 1;
        (significand as f64) * (exponent as f64).exp2()
    }

    /// Generates floats with standard gaussian density.
    pub fn gaussian(&mut self) -> f64 {
        let mut rsq;
        loop {
            rsq = self.uniform_double();
            if rsq != 0.0 {
                break;
            }
        }
        self.peek_uniform_double(1) * (-2.0 * rsq.ln() / rsq).sqrt()
    }

    /// Generates floats matching a poisson distribution with an expected value of `ev`
    pub fn poisson(&mut self, ev: f64) -> u64 {
        let mut n = 0;
        let em = (-ev).exp();
        let mut x = self.uniform_double();
        while x > em {
            n += 1;
            x *= self.peek_uniform_double(1);
        }
        n
    }

    /// Peeks at the `delta`-th future result of `.rand_range(min, max)` without changing the rng state
    pub fn peek_range(&self, delta: u128, min: u64, max: u64) -> u64 {
        let mut new_self = OrangeyCtx{..*self};
        new_self.skip(delta);
        new_self.rand_range(min, max)
    }

    /// Peeks at the `delta`-th future result of `.uniform_double()` without changing the rng state
    pub fn peek_uniform_double(&self, delta: u128) -> f64 {
        let mut new_self = OrangeyCtx{..*self};
        new_self.skip(delta);
        new_self.uniform_double()
    }

    /// Peeks at the `delta`-th future result of `.all_doubles()` without changing the rng state
    pub fn peek_all_doubles(&self, delta: u128) -> f64 {
        let mut new_self = OrangeyCtx{..*self};
        new_self.skip(delta);
        new_self.all_doubles()
    }

    /// Peeks at the `delta`-th future result of `.gaussian()` without changing the rng state
    pub fn peek_gaussian(&self, delta: u128) -> f64 {
        let mut new_self = OrangeyCtx{..*self};
        new_self.skip(delta);
        new_self.gaussian()
    }

    /// Peeks at the `delta`-th future result of `.poisson(ev)` without changing the rng state
    pub fn peek_poisson(&self, delta: u128, ev: f64) -> u64 {
        let mut new_self = OrangeyCtx{..*self};
        new_self.skip(delta);
        new_self.poisson(ev)
    }

    const MUL: u128 = 0x2360ed051fc65da44385df649fccf645;

    fn output(state: u128) -> u64 {
        ((state >> 64) as u64 ^ state as u64).rotate_right((state >> 122) as _)
    }

    fn step(&mut self) {
        self.state = (Wrapping(self.state) * Wrapping(Self::MUL) + Wrapping(self.inc)).0;
    }

    fn advance(
        state: u128,
        delta: u128,
        cur_mult: u128,
        cur_plus: u128,
    ) -> u128 {
        let state = Wrapping(state);
        let mut delta = Wrapping(delta);
        let mut cur_mult = Wrapping(cur_mult);
        let mut cur_plus = Wrapping(cur_plus);

        let mut acc_mult = Wrapping(1);
        let mut acc_plus = Wrapping(0);
        while delta > Wrapping(0) {
            if delta & Wrapping(1) != Wrapping(0) {
                acc_mult *= cur_mult;
                acc_plus = acc_plus * cur_mult + cur_plus;
            }
            cur_plus *= cur_mult + Wrapping(1);
            cur_mult *= cur_mult;
            delta /= 2;
        }
        (acc_mult * state + acc_plus).0
    }
}

impl Default for OrangeyCtx {
    fn default() -> Self {
        Self::new()
    }
}
