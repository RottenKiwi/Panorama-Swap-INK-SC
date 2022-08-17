; ModuleID = 'probe3.a6a93ed0-cgu.0'
source_filename = "probe3.a6a93ed0-cgu.0"
target datalayout = "e-m:e-p:32:32-p10:8:8-p20:8:8-i64:64-n32:64-S128-ni:1:10:20"
target triple = "wasm32-unknown-unknown"

; probe3::probe
; Function Attrs: nounwind
define hidden void @_ZN6probe35probe17hbe3c683fc3679105E() unnamed_addr #0 {
start:
  %0 = alloca i32, align 4
  store i32 1, i32* %0, align 4
  %1 = load i32, i32* %0, align 4
  br label %bb1

bb1:                                              ; preds = %start
  ret void
}

; Function Attrs: nofree nosync nounwind readnone speculatable willreturn
declare i32 @llvm.cttz.i32(i32, i1 immarg) #1

attributes #0 = { nounwind "target-cpu"="generic" }
attributes #1 = { nofree nosync nounwind readnone speculatable willreturn }
