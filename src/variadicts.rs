#![expect(unused_macros, unused_imports, reason = "maybe these macros get used")]

#[rustfmt::skip]
macro_rules! indexed_vec {
    ($e:expr, 10) => { vec![$e.0, $e.1, $e.2, $e.3, $e.4, $e.5, $e.6, $e.7, $e.8, $e.9] };
    ($e:expr,  9) => { vec![$e.0, $e.1, $e.2, $e.3, $e.4, $e.5, $e.6, $e.7, $e.8] };
    ($e:expr,  8) => { vec![$e.0, $e.1, $e.2, $e.3, $e.4, $e.5, $e.6, $e.7] };
    ($e:expr,  7) => { vec![$e.0, $e.1, $e.2, $e.3, $e.4, $e.5, $e.6] };
    ($e:expr,  6) => { vec![$e.0, $e.1, $e.2, $e.3, $e.4, $e.5] };
    ($e:expr,  5) => { vec![$e.0, $e.1, $e.2, $e.3, $e.4] };
    ($e:expr,  4) => { vec![$e.0, $e.1, $e.2, $e.3] };
    ($e:expr,  3) => { vec![$e.0, $e.1, $e.2] };
    ($e:expr,  2) => { vec![$e.0, $e.1] };
    ($e:expr,  1) => { vec![$e.0] };
    ($e:expr,  0) => { vec![] };
}

#[rustfmt::skip]
macro_rules! indexed_tuple {
    ($e:expr, 10) => { ( $e[0], $e[1], $e[2], $e[3], $e[4], $e[5], $e[6], $e[7], $e[8], $e[9],) };
    ($e:expr,  9) => { ( $e[0], $e[1], $e[2], $e[3], $e[4], $e[5], $e[6], $e[7], $e[8],) };
    ($e:expr,  8) => { ($e[0], $e[1], $e[2], $e[3], $e[4], $e[5], $e[6], $e[7]) };
    ($e:expr,  7) => { ($e[0], $e[1], $e[2], $e[3], $e[4], $e[5], $e[6]) };
    ($e:expr,  6) => { ($e[0], $e[1], $e[2], $e[3], $e[4], $e[5]) };
    ($e:expr,  5) => { ($e[0], $e[1], $e[2], $e[3], $e[4]) };
    ($e:expr,  4) => { ($e[0], $e[1], $e[2], $e[3]) };
    ($e:expr,  3) => { ($e[0], $e[1], $e[2]) };
    ($e:expr,  2) => { ($e[0], $e[1]) };
    ($e:expr,  1) => { ($e[0],) };
    ($e:expr,  0) => { () };
}

#[rustfmt::skip]
macro_rules! indexed_slice {
    ($e:expr, 10) => {[$e[0], $e[1], $e[2], $e[3], $e[4], $e[5], $e[6], $e[7], $e[8], $e[9]]};
    ($e:expr,  9) => {[$e[0], $e[1], $e[2], $e[3], $e[4], $e[5], $e[6], $e[7], $e[8]]};
    ($e:expr,  8) => {[$e[0], $e[1], $e[2], $e[3], $e[4], $e[5], $e[6], $e[7]]};
    ($e:expr,  7) => {[$e[0], $e[1], $e[2], $e[3], $e[4], $e[5], $e[6]]};
    ($e:expr,  6) => {[$e[0], $e[1], $e[2], $e[3], $e[4], $e[5]]};
    ($e:expr,  5) => {[$e[0], $e[1], $e[2], $e[3], $e[4]]};
    ($e:expr,  4) => {[$e[0], $e[1], $e[2], $e[3]]};
    ($e:expr,  3) => {[$e[0], $e[1], $e[2]]};
    ($e:expr,  2) => {[$e[0], $e[1]]};
    ($e:expr,  1) => {[$e[0]]};
    ($e:expr,  0) => {[]};
}

macro_rules! all_tuples_repeated {
    ($macro:ident, $min:tt, 10, $ty:tt) => {
        $crate::variadicts::all_tuples_repeated!($macro [$min] [10] ($ty, $ty, $ty, $ty, $ty, $ty, $ty, $ty, $ty, $ty));
    };

    ($macro:ident [$min:tt] [$n:tt] ($ty:tt $(, $tail:tt)*)) => {
        $macro!($n, $ty $(,$tail)*);
        $crate::variadicts::all_tuples_repeated!($macro @ [$min] [$n] ($($tail),*));
    };

    ($macro:ident @ [10] [10] $_:tt) => {};
    ($macro:ident @ [ 9] [ 9] $_:tt) => {};
    ($macro:ident @ [ 8] [ 8] $_:tt) => {};
    ($macro:ident @ [ 7] [ 7] $_:tt) => {};
    ($macro:ident @ [ 6] [ 6] $_:tt) => {};
    ($macro:ident @ [ 5] [ 5] $_:tt) => {};
    ($macro:ident @ [ 4] [ 4] $_:tt) => {};
    ($macro:ident @ [ 3] [ 3] $_:tt) => {};
    ($macro:ident @ [ 2] [ 2] $_:tt) => {};
    ($macro:ident @ [ 1] [ 1] $_:tt) => {};
    ($macro:ident @ [ 0] [ 0] $_:tt) => {};

    ($macro:ident @ [$min:tt] [10] ($($tail:tt),*)) => { $crate::variadicts::all_tuples_repeated!($macro [$min] [9] ($($tail),*)); };
    ($macro:ident @ [$min:tt] [ 9] ($($tail:tt),*)) => { $crate::variadicts::all_tuples_repeated!($macro [$min] [8] ($($tail),*)); };
    ($macro:ident @ [$min:tt] [ 8] ($($tail:tt),*)) => { $crate::variadicts::all_tuples_repeated!($macro [$min] [7] ($($tail),*)); };
    ($macro:ident @ [$min:tt] [ 7] ($($tail:tt),*)) => { $crate::variadicts::all_tuples_repeated!($macro [$min] [6] ($($tail),*)); };
    ($macro:ident @ [$min:tt] [ 6] ($($tail:tt),*)) => { $crate::variadicts::all_tuples_repeated!($macro [$min] [5] ($($tail),*)); };
    ($macro:ident @ [$min:tt] [ 5] ($($tail:tt),*)) => { $crate::variadicts::all_tuples_repeated!($macro [$min] [4] ($($tail),*)); };
    ($macro:ident @ [$min:tt] [ 4] ($($tail:tt),*)) => { $crate::variadicts::all_tuples_repeated!($macro [$min] [3] ($($tail),*)); };
    ($macro:ident @ [$min:tt] [ 3] ($($tail:tt),*)) => { $crate::variadicts::all_tuples_repeated!($macro [$min] [2] ($($tail),*)); };
    ($macro:ident @ [$min:tt] [ 2] ($($tail:tt),*)) => { $crate::variadicts::all_tuples_repeated!($macro [$min] [1] ($($tail),*)); };
    ($macro:ident @ [$min:tt] [ 1] ($($tail:tt),*)) => { $crate::variadicts::all_tuples_repeated!($macro [$min] [0] ($($tail),*)); };

}

/// ```ignore
/// dual_combination!(macro, [x, y, z]);
/// => macro!(x, y);
/// => macro!(x, z);
/// => macro!(y, z);
///
/// dual_combination!(macro, [x, y, z, w]);
/// => macro!(x, y);
/// => macro!(x, z);
/// => macro!(x, w);
/// => macro!(y, z);
/// => macro!(y, w);
/// => macro!(z, w);
/// ```
macro_rules! dual_combination {
    ($m:ident, [$start:tt, $($tail:tt),+ $(,)?]) => {
        $crate::variadicts::dual_combination!([$m] branch [$start] [$([$tail])+]);
    };

    ([$m:ident] $_:ident [$start:tt] []) => {};

    ([$m:ident] branch [$start:tt] [[$item:tt] $([$tail:tt])*]) => {
        $crate::variadicts::dual_combination!([$m] no_branch [$start] [$([$tail])*]);
        $crate::variadicts::dual_combination!([$m] branch [$item] [$([$tail])*]);
        $m!($start, $item);
    };

    ([$m:ident] no_branch [$start:tt] [[$item:tt] $([$tail:tt])*]) => {
        $crate::variadicts::dual_combination!([$m] no_branch [$start] [$([$tail])*]);
        $m!($start, $item);
    };
}

pub(crate) use all_tuples_repeated;
pub(crate) use dual_combination;
pub(crate) use indexed_slice;
pub(crate) use indexed_tuple;
pub(crate) use indexed_vec;
