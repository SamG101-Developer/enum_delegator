extern crate enum_delegator;
extern crate proc_macros;
mod other;

use other::MyOtherTrait;

use proc_macros::{define_delegation, define_delegator};


#[define_delegator]
trait MyTrait {
    fn method_a(&self) -> i32;
    fn method_b(&self) -> i32;
    fn method_c(&self, a: bool) -> i64;
}

struct TypeA { a: i32 }
impl MyOtherTrait for TypeA { }
impl MyTrait for TypeA {
    fn method_a(&self) -> i32 { self.a }
    fn method_b(&self) -> i32 { self.a + 1 }
    fn method_c(&self, a: bool) -> i64 { if a { 1 } else { 0 } }
}

struct TypeB { b: i32 }
impl MyOtherTrait for TypeB { }
impl MyTrait for TypeB {
    fn method_a(&self) -> i32 { self.b }
    fn method_b(&self) -> i32 { self.b - 1 }
    fn method_c(&self, a: bool) -> i64 { if a { 2 } else { 0 } }
}

#[define_delegation(MyTrait, MyOtherTrait)]
enum Combinator {
    TypeA(TypeA),
    TypeB(TypeB),
}

#[test]
fn test_delegation() {
    let a = Combinator::TypeA(TypeA { a: 10 });
    let b = Combinator::TypeB(TypeB { b: 20 });

    assert_eq!(a.method_a(), 10);
    assert_eq!(a.method_b(), 11);
    assert_eq!(a.method_c(true), 1);
    assert_eq!(a.method_d(), "default");
    assert_eq!(b.method_a(), 20);
    assert_eq!(b.method_b(), 19);
    assert_eq!(b.method_c(false), 0);
    assert_eq!(b.method_d(), "default");
}

