;; Exceptions: try_table/throw/throw_ref.
;; (Legacy try/catch/rethrow/delegate text is rejected by the wat crate —
;; that proposal was removed; lifter arms remain for old binaries.)
(module
  (tag $e (param i32))
  (func (export "th") (param i32)
    (throw $e (local.get 0)))
  (func (export "thr") (param exnref)
    (throw_ref (local.get 0)))
  (func (export "tryt") (param i32) (result i32)
    (block (result i32)
      (try_table (result i32) (catch $e 0) (local.get 0))
      (drop)
      (i32.const 7))))
