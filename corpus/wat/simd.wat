;; SIMD shape coverage: loads, const, splat, arith, compare,
;; extract/replace, shuffle, converts. One func per shape.
(module
  (memory 1)
  (func (export "vadd") (param i32 i32 i32)
    (v128.store (local.get 2)
      (i32x4.add (v128.load (local.get 0)) (v128.load (local.get 1)))))
  (func (export "vconst") (result v128)
    (v128.const i32x4 1 2 3 4))
  (func (export "vsplat") (param i32) (result v128)
    (i32x4.splat (local.get 0)))
  (func (export "vlanes") (param v128) (result i32)
    (i32.add
      (i32x4.extract_lane 0 (local.get 0))
      (i32x4.extract_lane 3 (local.get 0))))
  (func (export "vreplace") (param v128 i32) (result v128)
    (i32x4.replace_lane 2 (local.get 0) (local.get 1)))
  (func (export "vshuffle") (param v128 v128) (result v128)
    (i8x16.shuffle 0 1 2 3 4 5 6 7 16 17 18 19 20 21 22 23
      (local.get 0) (local.get 1)))
  (func (export "vcmp") (param v128 v128) (result v128)
    (i32x4.eq (local.get 0) (local.get 1)))
  (func (export "vbit") (param v128 v128 v128) (result v128)
    (v128.bitselect (local.get 0) (local.get 1) (local.get 2)))
  (func (export "vmul") (param v128 v128) (result v128)
    (f32x4.mul (local.get 0) (local.get 1)))
  (func (export "vcvt") (param v128) (result v128)
    (f32x4.convert_i32x4_s (local.get 0))))
