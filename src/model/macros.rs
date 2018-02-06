macro_rules! define_id {
    ($($id:ident: $t:ty),*) => {$(
        #[derive(
            Clone,
            Constructor,
            Debug,
            Deserialize,
            Default,
            From,
            FromSegment,
            Into,
            PartialEq,
            Serialize,
        )]
        pub struct $id(pub $t);
    )*};

    ($($id:ident: $t:ty,)*) => {
        define_id! {
            $($id: $t),*
        }
    };
}
