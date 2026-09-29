(module
  (memory 1)
  (func (export "mk") (param i32) (result i32)
    ;; struct S { x @0, y @4 }: two stores then a load
    (i32.store offset=4 (local.get 0) (i32.const 41))
    (i32.store (local.get 0) (i32.const 1))
    (i32.load offset=4 (local.get 0))))
