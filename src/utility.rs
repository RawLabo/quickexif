pub(crate) trait GetNumFromBytes {
    fn u16(&self, is_le: bool, start: usize) -> u16;
    fn u16le(&self, start: usize) -> u16;
    fn u16be(&self, start: usize) -> u16;
    fn u32(&self, is_le: bool, start: usize) -> u32;
    fn u32le(&self, start: usize) -> u32;
    fn u32be(&self, start: usize) -> u32;
    fn i32(&self, is_le: bool, start: usize) -> i32;
    fn r64(&self, is_le: bool, start: usize) -> f64;
}

pub(crate) trait GetBytesFromInt<T> {
    fn to_bytes(self, is_le: bool) -> T;
}

macro_rules! gen_impl_get_int {
    ($t:tt, $len:expr) => {
        fn $t(&self, is_le: bool, start: usize) -> $t {
            let bytes: [u8; $len] = *&self[start..start + $len].try_into().unwrap();
            if is_le {
                $t::from_le_bytes(bytes)
            } else {
                $t::from_be_bytes(bytes)
            }
        }
    };
}
impl GetNumFromBytes for &[u8] {
    gen_impl_get_int!(u16, 2);
    gen_impl_get_int!(u32, 4);
    gen_impl_get_int!(i32, 4);

    fn u16le(&self, start: usize) -> u16 {
        let bytes: [u8; 2] = self[start..start + 2].try_into().unwrap();
        u16::from_le_bytes(bytes)
    }
    fn u16be(&self, start: usize) -> u16 {
        let bytes: [u8; 2] = self[start..start + 2].try_into().unwrap();
        u16::from_be_bytes(bytes)
    }
    fn u32le(&self, start: usize) -> u32 {
        let bytes: [u8; 4] = self[start..start + 4].try_into().unwrap();
        u32::from_le_bytes(bytes)
    }
    fn u32be(&self, start: usize) -> u32 {
        let bytes: [u8; 4] = self[start..start + 4].try_into().unwrap();
        u32::from_be_bytes(bytes)
    }

    fn r64(&self, is_le: bool, start: usize) -> f64 {
        let n = self.i32(is_le, start) as f64;
        let d = self.u32(is_le, start + 4) as f64;
        n / d
    }
}

macro_rules! gen_get_bytes_impls {
    ($t:ty, $n:expr) => {
        impl GetBytesFromInt<[u8; $n]> for $t {
            fn to_bytes(self, is_le: bool) -> [u8; $n] {
                if is_le {
                    self.to_le_bytes()
                } else {
                    self.to_be_bytes()
                }
            }
        }
    };
}
gen_get_bytes_impls!(u16, 2);
gen_get_bytes_impls!(u32, 4);

pub(crate) fn sony_decrypt(data: &[u8], mut key: u32, is_le: bool) -> Vec<u8> {
    let mut pad = [0u32; 128];
    for item in pad.iter_mut().take(4) {
        key = key.wrapping_mul(48828125).wrapping_add(1);
        *item = key;
    }
    pad[3] = pad[3] << 1 | (pad[0] ^ pad[2]) >> 31;
    for i in 4..127 {
        pad[i] = (pad[i - 4] ^ pad[i - 2]) << 1 | (pad[i - 3] ^ pad[i - 1]) >> 31;
    }
    for item in pad.iter_mut().take(127) {
        *item = item.swap_bytes();
    }

    data.chunks_exact(4)
        .map(|x| x.u32(is_le, 0))
        .zip(127..)
        .flat_map(|(x, p)| {
            pad[p & 127] = pad[(p + 1) & 127] ^ pad[(p + 65) & 127];
            (x ^ pad[p & 127]).to_bytes(is_le)
        })
        .collect::<Vec<u8>>()
}