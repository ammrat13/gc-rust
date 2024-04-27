use std::cell::RefCell;

use crate::refs::GCRef;
use crate::GCContext;

// -----------------------------------------------------------------------------

#[test]
fn retain() {
    let mut gc = GCContext::new();
    let _l = gc.allocate(0);
    assert_eq!(gc.allocations().len(), 1);
    assert!(gc.collect().is_ok());
    assert_eq!(gc.allocations().len(), 1);
}

#[test]
fn leak() {
    let mut gc = GCContext::new();
    dut_leak(&mut gc);
    assert_eq!(gc.allocations().len(), 1);
    assert!(gc.collect().is_ok());
    assert_eq!(gc.allocations().len(), 0);
}
#[inline(never)]
fn dut_leak(gc: &mut GCContext) {
    let _l = gc.allocate(0);
}

// -----------------------------------------------------------------------------

#[test]
fn chain() {
    let mut gc = GCContext::new();
    let _c = dut_chain(&mut gc);
    assert_eq!(gc.allocations().len(), 2);
    assert!(gc.collect().is_ok());
    assert_eq!(gc.allocations().len(), 2);
}
#[inline(never)]
fn dut_chain(gc: &mut GCContext) -> GCRef<GCRef<i32>> {
    let v = gc.allocate(0);
    let p = gc.allocate(v.clone());
    p
}

#[test]
fn leak_chain() {
    let mut gc = GCContext::new();
    dut_leak_chain(&mut gc);
    assert_eq!(gc.allocations().len(), 2);
    assert!(gc.collect().is_ok());
    assert_eq!(gc.allocations().len(), 0);
}
#[inline(never)]
fn dut_leak_chain(gc: &mut GCContext) {
    let v = gc.allocate(0);
    let _p = gc.allocate(v.clone());
}

// -----------------------------------------------------------------------------

struct Parent {
    child: RefCell<Option<GCRef<Child>>>,
}

struct Child {
    _parent: GCRef<Parent>,
}

#[test]
fn cycle() {
    let mut gc = GCContext::new();
    let _p = dut_cycle(&mut gc);
    assert_eq!(gc.allocations().len(), 2);
    assert!(gc.collect().is_ok());
    assert_eq!(gc.allocations().len(), 2);
}
#[inline(never)]
fn dut_cycle(gc: &mut GCContext) -> GCRef<Parent> {
    let par = gc.allocate(Parent { child: RefCell::new(None) });
    let chi = gc.allocate(Child { _parent: par.clone() });
    par.child.replace(Some(chi));
    par
}

#[test]
fn leak_cycle() {
    let mut gc = GCContext::new();
    dut_leak_cycle(&mut gc);
    assert_eq!(gc.allocations().len(), 2);
    assert!(gc.collect().is_ok());
    assert_eq!(gc.allocations().len(), 0);
}
#[inline(never)]
fn dut_leak_cycle(gc: &mut GCContext) {
    let par = gc.allocate(Parent { child: RefCell::new(None) });
    let chi = gc.allocate(Child { _parent: par.clone() });
    par.child.replace(Some(chi));
}

// -----------------------------------------------------------------------------
