use crate::GCContext;
use crate::refs::GCRef;

#[test]
fn smoke() {
    let mut gc = GCContext::new();
    let _l = gc.allocate(0);
    assert_eq!(gc.allocations().len(), 1);
}

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
fn leak() {
    let mut gc = GCContext::new();
    dut_leak(&mut gc);
    assert_eq!(gc.allocations().len(), 1);
    assert!(gc.collect().is_ok());
    assert_eq!(gc.allocations().len(), 0);
}
#[inline(never)]
fn dut_leak(gc: &mut GCContext) {
    let _ = gc.allocate(0);
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
