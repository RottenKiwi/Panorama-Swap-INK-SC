; ModuleID = 'probe2.e39d7f4e-cgu.0'
source_filename = "probe2.e39d7f4e-cgu.0"
target datalayout = "e-m:e-p:32:32-p10:8:8-p20:8:8-i64:64-n32:64-S128-ni:1:10:20"
target triple = "wasm32-unknown-unknown"

; probe2::probe
; Function Attrs: nounwind
define hidden void @_ZN6probe25probe17h19838ae8bd8c1515E() unnamed_addr #0 {
start:
  %0 = alloca i32, align 4
  store i32 -2147483648, i32* %0, align 4
  %1 = load i32, i32* %0, align 4
  br label %bb1

bb1:                                              ; preds = %start
  ret void
}

; Function Attrs: nofree nosync nounwind readnone speculatable willreturn
declare i32 @llvm.bitreverse.i32(i32) #1

attributes #0 = { nounwind "target-cpu"="generic" }
attributes #1 = { nofree nosync nounwind readnone speculatable willreturn }
