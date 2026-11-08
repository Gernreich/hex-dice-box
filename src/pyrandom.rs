//! From trumpet-elbows-allowed (github.com/Gernreich/trumpet-elbows-allowed), tools/pyrandom.rs.
#![allow(dead_code)]
//! Python's random.Random, so a seed means here what it meant to the Python
//! these tools were: the Mersenne Twister, seeded from an int as init_by_array
//! seeds it, and random(), shuffle(), uniform(), randrange() and randint()
//! drawing from it in the same way.
pub struct Mt {
    mt: [u32; 624],
    i: usize,
}

impl Mt {
    /// random.Random(seed) for an int seed: init_by_array over its 32-bit words
    pub fn new(seed: i64) -> Mt {
        let mut n = seed.unsigned_abs();
        let mut key = vec![];
        while n > 0 {
            key.push(n as u32);
            n >>= 32;
        }
        if key.is_empty() {
            key.push(0);
        }
        let mut m = Mt { mt: [0; 624], i: 624 };
        m.mt[0] = 19650218;
        for i in 1..624 {
            m.mt[i] = 1812433253u32.wrapping_mul(m.mt[i - 1] ^ (m.mt[i - 1] >> 30)).wrapping_add(i as u32);
        }
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..624.max(key.len()) {
            m.mt[i] = (m.mt[i] ^ (m.mt[i - 1] ^ (m.mt[i - 1] >> 30)).wrapping_mul(1664525)).wrapping_add(key[j]).wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= 624 {
                m.mt[0] = m.mt[623];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..623 {
            m.mt[i] = (m.mt[i] ^ (m.mt[i - 1] ^ (m.mt[i - 1] >> 30)).wrapping_mul(1566083941)).wrapping_sub(i as u32);
            i += 1;
            if i >= 624 {
                m.mt[0] = m.mt[623];
                i = 1;
            }
        }
        m.mt[0] = 0x80000000;
        m
    }
    pub fn next(&mut self) -> u32 {
        if self.i >= 624 {
            for k in 0..624 {
                let y = (self.mt[k] & 0x80000000) | (self.mt[(k + 1) % 624] & 0x7fffffff);
                self.mt[k] = self.mt[(k + 397) % 624] ^ (y >> 1) ^ if y & 1 != 0 { 0x9908b0df } else { 0 };
            }
            self.i = 0;
        }
        let mut y = self.mt[self.i];
        self.i += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^ (y >> 18)
    }
    pub fn random(&mut self) -> f64 {
        let a = (self.next() >> 5) as f64;
        let b = (self.next() >> 6) as f64;
        (a * 67108864.0 + b) * (1.0 / 9007199254740992.0)
    }
    pub fn below(&mut self, n: usize) -> usize {
        let k = usize::BITS - n.leading_zeros();
        loop {
            let r = (self.next() >> (32 - k)) as usize;
            if r < n {
                return r;
            }
        }
    }
    pub fn shuffle<T>(&mut self, x: &mut [T]) {
        for i in (1..x.len()).rev() {
            let j = self.below(i + 1);
            x.swap(i, j);
        }
    }
    pub fn uniform(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.random()
    }

    /// random.randrange(n)
    pub fn randrange(&mut self, n: usize) -> usize {
        self.below(n)
    }
    /// random.randint(a, b)
    pub fn randint(&mut self, a: i64, b: i64) -> i64 {
        a + self.below((b - a + 1) as usize) as i64
    }
}
