(module
  (memory 1)
  (data (i32.const 32) "auth-ok")
  (data (i32.const 64) "\00\01\02\03binary-\ff-data")
  (func (export "check") (param i32) (result i32)
    (i32.load8_u (i32.add (i32.const 32) (local.get 0))))
  (func (export "blob") (param i32) (result i32)
    (i32.load (i32.add (i32.const 64) (local.get 0)))))
