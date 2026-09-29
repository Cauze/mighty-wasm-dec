(module
  (memory 1)
  (data (i32.const 16) "hello-wasm")
  (func (export "getbyte") (param i32) (result i32)
    (i32.load8_u (i32.add (i32.const 16) (local.get 0)))))
