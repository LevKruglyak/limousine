pub mod prelude {
    pub use limousine_core::ImmutableIndex;

    pub mod limousine_macros {
        pub use limousine_derive::create_immutable_hybrid_index;
    }
}

#[doc(hidden)]
pub use limousine_core as private;
