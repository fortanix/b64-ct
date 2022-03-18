/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rand::{distributions::Distribution, Rng};

struct Base64;

impl Distribution<u8> for Base64 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u8 {
        crate::misc::LUT_STANDARD[(rng.next_u32() & 0x3f) as usize]
    }
}

pub fn rand_base64_size(s: usize) -> std::vec::Vec<u8> {
    rand::thread_rng().sample_iter(&Base64).take(s).collect()
}

// `with_cartesian_products!(m (a b) (c d));` expands to `m!(a c); m!(a d); m!(b c); m!(b d);`.
#[macro_export]
macro_rules! with_cartesian_products {
    (@$m:ident $acc:tt ($($choice:tt)*) $rest:tt) => {
        $(with_cartesian_products!(@$rest $acc $choice $m);)*
    };
    (@() ($($acc:tt)*) $choice:tt $m:ident) => {
        $m!($($acc)* $choice);
    };
    (@($next:tt $($rest:tt)*) ($($acc:tt)*) $choice:tt $m:ident) => {
        with_cartesian_products!(@$m ($($acc)* $choice) $next ($($rest)*));
    };
    ($m:ident $next:tt $($rest:tt)*) => {
        with_cartesian_products!(@$m () $next ($($rest)*));
    };
}

#[macro_export]
macro_rules! generate_tests {
    (
        $_a:ident<$a:ident>: { $($(#[$am:meta])* $an:ident, $at:expr;)* },
        $_b:ident<$b:ident>: { $($(#[$bm:meta])* $bn:ident, $bt:expr;)* },
        tests: { $($tn:ident,)* },
    ) => {
        with_cartesian_products!( generate_tests ((@ $a $b)) ($($tn)*) ($(($(#[$am])* $an, $at))*) ($(($(#[$bm])*$bn, $bt))*) );
    };
    ((@ $a:ident $b:ident) $tn:ident ($(#[$am:meta])* $an:ident, $at:expr) ($(#[$bm:meta])* $bn:ident, $bt:expr)) => {
        paste::item! {
            $(#[$am])*
            $(#[$bm])*
            #[test]
            #[allow(non_snake_case)]
            fn [< $tn _ $a $an _ $b $bn >]() {
                $tn($at, $bt);
            }
        }
    }
}
