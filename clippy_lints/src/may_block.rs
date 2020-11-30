use crate::utils::{match_def_path, paths, span_lint_and_note};
use rustc_hir::def_id::DefId;
use rustc_hir::{AsyncGeneratorKind, Body, BodyId, GeneratorKind};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::GeneratorInteriorTypeCause;
use rustc_session::{declare_lint_pass, declare_tool_lint};
use rustc_span::Span;

declare_clippy_lint! {
    /// **What it does:** Checks for calls to await while holding a
    /// non-async-aware MutexGuard.
    ///
    /// **Why is this bad?** The Mutex types found in std::sync and parking_lot
    /// are not designed to operate in an async context across await points.
    ///
    /// There are two potential solutions. One is to use an asynx-aware Mutex
    /// type. Many asynchronous foundation crates provide such a Mutex type. The
    /// other solution is to ensure the mutex is unlocked before calling await,
    /// either by introducing a scope or an explicit call to Drop::drop.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust,ignore
    /// use std::sync::Mutex;
    ///
    /// async fn foo(x: &Mutex<u32>) {
    ///   let guard = x.lock().unwrap();
    ///   *guard += 1;
    ///   bar.await;
    /// }
    /// ```
    ///
    /// Use instead:
    /// ```rust,ignore
    /// use std::sync::Mutex;
    ///
    /// async fn foo(x: &Mutex<u32>) {
    ///   {
    ///     let guard = x.lock().unwrap();
    ///     *guard += 1;
    ///   }
    ///   bar.await;
    /// }
    /// ```
    pub MAY_BLOCK,
    correctness,
    "Using blocking functions in async code can slow down the async runtime"
}

declare_lint_pass!(MayBlock => [MAY_BLOCK]);

impl LateLintPass<'_> for MayBlock {
    fn check_body(&mut self, cx: &LateContext<'_>, body: &'_ Body<'_>) {
        use AsyncGeneratorKind::{Block, Closure, Fn};
        if let Some(GeneratorKind::Async(Block | Closure | Fn)) = body.generator_kind {
            dbg!(body);
            let body_id = BodyId {
                hir_id: body.value.hir_id,
            };
            let def_id = cx.tcx.hir().body_owner_def_id(body_id);
            let typeck_results = cx.tcx.typeck(def_id);
            check_interior_types(cx, &typeck_results.generator_interior_types, body.value.span);
        }
    }
}

fn check_interior_types(cx: &LateContext<'_>, ty_causes: &[GeneratorInteriorTypeCause<'_>], span: Span) {
    for ty_cause in ty_causes {
        if let rustc_middle::ty::Adt(adt, _) = ty_cause.ty.kind() {
            if is_blocking(cx, adt.did) {
                span_lint_and_note(
                    cx,
                    MAY_BLOCK,
                    ty_cause.span,
                    "this blocking function can slow down the async runtime. consider using a non-blocking alternative from the async ecosystem of your runtime.",
                    ty_cause.scope_span.or(Some(span)),
                    "these are all the await points this lock is held through",
                );
            }
        }
    }
}

fn is_blocking(cx: &LateContext<'_>, def_id: DefId) -> bool {
    match_def_path(cx, def_id, &paths::THREAD_SLEEP)
}
