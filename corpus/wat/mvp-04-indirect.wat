(module
  (type $t (func (param i32) (result i32)))
  (table 2 funcref)
  (elem (i32.const 0) $a $b)
  (func $a (param i32) (result i32) (i32.add (local.get 0) (i32.const 1)))
  (func $b (param i32) (result i32) (i32.mul (local.get 0) (i32.const 2)))
  (func (export "run") (param i32 i32) (result i32)
    (call_indirect (type $t) (local.get 1) (local.get 0))))
