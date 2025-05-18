use proc_macros::define_delegator;

#[define_delegator]
pub trait MyOtherTrait {
    fn method_d(&self) -> String {
        "default".to_string()
    }
}
