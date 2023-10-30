(module
  (type $t0 (func (param i32 i32) (result i32)))
  
(import "dylibso_observe" "instrument_enter" (func $instrument_enter (type $t0)))
(import "dylibso_observe" "instrument_exit" (func $instrument_exit (type $t0)))
(import "dylibso_observe" "instrument_memory_grow" (func $instrument_memory_grow (type $t0)))
(func $add (export "add") (type $t0) (param $p0 i32) (param $p1 i32) (result i32)
    (i32.add
      (local.get $p1)
      (local.get $p0)))
            (func $instrument_exp_add (export "add") (type $t0) (param $p0 i32) (param $p1 i32) (result i32)
            (local $l2 i32)
            (call $instrument_enter
                (i32.const 3))
            (local.set $l2
              (call $add
            (local.get $p0)(local.get $p1)))
            (call $instrument_exit
                (i32.const 3))
            (local.get $l2))
  (table $T0 1 1 funcref)
  (memory $memory (export "memory") 16)
  (global $__data_end (export "__data_end") i32 (i32.const 1048576))
  (global $__heap_base (export "__heap_base") i32 (i32.const 1048576)))