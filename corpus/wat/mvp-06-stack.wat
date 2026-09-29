(module
  (memory 1)
  (global $sp (mut i32) (i32.const 1024))
  ;; Emscripten-style stack frame: alloc 16 bytes, spill, use, free
  (func (export "frame") (param i32) (result i32)
    (local i32)
    (global.set $sp (i32.sub (global.get $sp) (i32.const 16)))
    (local.set 1 (global.get $sp))
    (i32.store offset=0 (local.get 1) (local.get 0))
    (i32.store offset=4 (local.get 1) (i32.const 7))
    (i32.load offset=0 (local.get 1))
    (global.set $sp (i32.add (global.get $sp) (i32.const 16)))
  ))
