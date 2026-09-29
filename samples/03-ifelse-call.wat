(module
  (func $add (param i32 i32) (result i32)
    (i32.add (local.get 0) (local.get 1)))
  (func (export "main") (param i32) (result i32)
    (if (result i32) (i32.gt_s (local.get 0) (i32.const 10))
      (then (call $add (local.get 0) (i32.const 1)))
      (else (call $add (local.get 0) (i32.const -1))))))
