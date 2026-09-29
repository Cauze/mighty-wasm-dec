(module
  (func (export "dispatch") (param i32) (result i32)
    (block (block (block
      (br_table 0 1 2 (local.get 0)))
      (return (i32.const 10)))
      (return (i32.const 20)))
    (i32.const 30)))
