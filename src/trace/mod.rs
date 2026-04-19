use std::collections::HashMap;

use ethers::{
    providers::{Middleware, Provider, Ws},
    types::{
        Address, BlockId, BlockNumber, Bytes, CallConfig, CallFrame, GethDebugBuiltInTracerConfig,
        GethDebugBuiltInTracerType, GethDebugTracerConfig, GethDebugTracerType,
        GethDebugTracingCallOptions, GethTrace, GethTraceFrame, NameOrAddress, Transaction,
    },
};

fn trace_opts() -> GethDebugTracingCallOptions {
    let mut opts = GethDebugTracingCallOptions::default();
    opts.tracing_options.tracer = Some(GethDebugTracerType::BuiltInTracer(
        GethDebugBuiltInTracerType::CallTracer,
    ));
    opts.tracing_options.tracer_config = Some(GethDebugTracerConfig::BuiltInTracer(
        GethDebugBuiltInTracerConfig::CallTracer(CallConfig {
            only_top_call: Some(false),
            with_log: Some(false),
        }),
    ));
    opts.tracing_options.timeout = Some("1s".into());
    opts
}

fn flatten(frame: &CallFrame, out: &mut HashMap<Address, Bytes>) {
    if let Some(NameOrAddress::Address(addr)) = &frame.to {
        out.insert(*addr, frame.input.clone());
    }
    for child in frame.calls.iter().flatten() {
        flatten(child, out);
    }
}

pub async fn get_flattened_trace(
    tx: Transaction,
    provider: Provider<Ws>,
) -> Option<HashMap<Address, Bytes>> {
    let trace = provider
        .debug_trace_call(&tx, Some(BlockId::Number(BlockNumber::Latest)), trace_opts())
        .await
        .ok()?;
    let GethTrace::Known(GethTraceFrame::CallTracer(frame)) = trace else {
        return None;
    };
    let mut out = HashMap::new();
    flatten(&frame, &mut out);
    Some(out)
}
