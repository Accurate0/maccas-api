#[macro_export]
macro_rules! name_of {
    // Covers Bindings
    ($n: ident) => {{
        let _ = || {
            let _ = &$n;
        };
        stringify!($n)
    }};

    // Covers Types
    (type $t: ty) => {{ $crate::name_of_type!($t) }};

    // Covers Struct Fields
    ($n: ident in $t: ty) => {{
        let _ = |f: $t| {
            let _ = &f.$n;
        };
        stringify!($n)
    }};

    // Covers Struct Constants
    (const $n: ident in $t: ty) => {{
        let _ = || {
            let _ = &<$t>::$n;
        };
        stringify!($n)
    }};
}
