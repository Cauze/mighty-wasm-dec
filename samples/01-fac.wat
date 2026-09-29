(module
  (func $fac (export "fac") (param i32) (result i32)
    (local i32)
    i32.const 1 local.set 1
    (block
      (loop
        (br_if 1 (i32.eqz (local.get 0)))
        (local.set 1 (i32.mul (local.get 1) (local.get 0)))
        (local.set 0 (i32.sub (local.get 0) (i32.const 1)))
        (br 0))
    )
    local.get 1))
