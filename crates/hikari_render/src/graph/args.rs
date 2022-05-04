use std::marker::PhantomData;

// use std::marker::PhantomData;

// pub trait Fetch<'a>: Sized {
//     type Item;
// }
// pub trait Args {
//     type Fetch: for<'a> Fetch<'a>;
// }
// pub struct RefFetch<T>(PhantomData<T>);

// impl<'a, T: 'static> Fetch<'a> for RefFetch<T> {
//     type Item = &'a T;
// }

// impl<'a, T: 'static> Args for &'a T {
//     type Fetch = RefFetch<T>;
// }

// impl<'a> Fetch<'a> for () {
//     type Item = ();
// }
// impl Args for () {
//     type Fetch = ();
// }

// macro_rules! impl_args {
//     ($($name: ident),*) => {
//         impl<$($name: Args),*> Args for ($($name,)*) {
//             type Fetch = ($($name::Fetch,)*);
//         }

//         impl<'a, $($name: Fetch<'a>),*> Fetch<'a> for ($($name,)*) {
//             type Item = ($($name::Item,)*);
//         }
//     }
// }
// impl_args!(A);
// impl_args!(A, B);
// impl_args!(A, B, C);
// impl_args!(A, B, C, D);
// impl_args!(A, B, C, D, E);
// impl_args!(A, B, C, D, E, F);
// impl_args!(A, B, C, D, E, F, G);
// impl_args!(A, B, C, D, E, F, G, H);

pub trait ByRef<'a>{
    type Item: Copy;
}
pub trait Args {
    type Ref: for<'a> ByRef<'a>;
}

pub struct RefImpl<T>(PhantomData<T>);
impl<'a, T: 'a> ByRef<'a> for RefImpl<T> {
    type Item = &'a T;
}

macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        impl<$($name: 'static),*> Args for ($($name, )+)
        {
            type Ref = ($(RefImpl<$name>,)*);
        }
        impl<'a, $($name: ByRef<'a>),*> ByRef<'a> for ($($name, )+)
        {
            type Item = ($($name::Item,)*);
        }
    }
}

tuple_impls! {A}
tuple_impls! {A B}
tuple_impls! {A B C}
tuple_impls! {A B C D}
tuple_impls! {A B C D E}
tuple_impls! {A B C D E F}
tuple_impls! {A B C D E F G}