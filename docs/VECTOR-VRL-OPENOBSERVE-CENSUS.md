# Vector × lance-graph-vrl × OpenObserve: reuse census and integration plan

Status: CENSUS + PLAN. No implementation has landed from this document.
Date: 2026-11-25

Scope note. This repo (`paperless-rs`) owns only the Dockerfile and deployment settings (see `CLAUDE.md`). The census covers sibling repos at pinned commits. Every factual claim below is tied to a source link at those commits. Where a fact could not be verified, the text says UNVERIFIED or NOT FOUND.

## 0. Decision in one paragraph

Vector is the transport and normalization layer. It is a deployed binary, not a library: all Vector lib crates (vector-core, vector-lib, codecs, vector-buffers and others) are `publish = false` ([vector Cargo.toml:10](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/Cargo.toml#L10), [vector lib/vector-core/Cargo.toml:6](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/vector-core/Cargo.toml#L6), [vector lib/vector-lib/Cargo.toml:6](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/vector-lib/Cargo.toml#L6), [vector lib/codecs/Cargo.toml:6](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/codecs/Cargo.toml#L6)). The `vrl` crate is publishable and is the only Vector-side library seam ([vrl src/lib.rs:4-52](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/lib.rs#L4-L52) gates the modules). OpenObserve is retention, historical evidence and reference SQL. lance-graph is native typed evaluation over resident lanes. `lance-graph-vrl` and Quack are two frontends over one field seam, decided as D-RSP-5 ([lance-graph .claude/plans/registered-source-profiles-v1.md:110-155](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/.claude/plans/registered-source-profiles-v1.md#L110-L155)).

Headline findings:

- (a) `lance-graph-vrl` has no dependency on upstream `vrl` and no differential test against it ([lance-graph crates/lance-graph-vrl/src/lib.rs:376](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-vrl/src/lib.rs#L376) is its own parser). No Cargo.toml in the sources depends on `vrl`.
- (b) MEASURED: its core construct `if .f > N { abort }` is rejected by upstream VRL with E100 unless the field is declared a non-optional integer through `ExternalEnv`. With that declaration, an absent value is a runtime error upstream. `lance-graph-vrl` answers UNKNOWN. See section 4.1.
- (c) `==` and `!=` diverge on absent values. Upstream is two-valued; `lance-graph-vrl` and D-RSP-5 answer UNKNOWN.
- (d) VRL version skew: Vector runs vrl 0.39.0 at 7738f74 ([vector Cargo.lock:13892-13894](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/Cargo.lock#L13892-L13894)). OpenObserve runs vrl 0.31.0 ([openobserve Cargo.lock:14506](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/Cargo.lock#L14506)).
- (e) No time windows, watermarks or incremental aggregate maintenance exist in lance-graph. The reuse path is to recompute the affected window over resident lanes.
- (f) `sentinel-o2` is read-only ([sentinel-rs crates/sentinel-o2/src/lib.rs:6-7](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/lib.rs#L6-L7)). No loop-prevention marker exists anywhere.
- (g) Datadog trace redirection to Vector is UNVERIFIED on the agent side. The keys exist; no consumer was found in the sparse checkout.

## 1. Pinned sources

| repo | commit | license |
|---|---|---|
| vectordotdev/vector | c879bd677429193410389be29207da6c44c5aa21 | MPL-2.0 |
| vectordotdev/vrl | 2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7 (crate 0.39.0) | MPL-2.0, publishable |
| DataDog/datadog-agent | d92d1f35a782d230bb2fc9ddbb0ef03c71893cca (sparse checkout) | not checked (Apache-2.0 per its repo; only config and endpoint code was read) |
| DataDog/agent-payload | b88220d9592d43ed8a5a95a3ea90255bc2ce1f91 | BSD-3-Clause (`LICENSE`) |
| openobserve/openobserve | 761446c5b1b45c5e20ad69c94ef6cd2df3962622 | AGPL-3.0 ([openobserve Cargo.toml:17](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/Cargo.toml#L17)) |
| AdaWorldAPI/lance-graph | f5e625f131ff0f39de3b3207cf7ae7cb92c23f04 (toolchain 1.99.0) | crates MIT OR Apache-2.0 (e.g. `crates/lance-graph-vrl/Cargo.toml`) |
| AdaWorldAPI/sentinel-rs | 24807d29619023d3ce5b623413401376988719e9 | none declared |
| AdaWorldAPI/OGAR | ac1441427f166d630e488e934a37742495935bbc | MIT |
| AdaWorldAPI/xlog | 28b41f5e5c74eb915c89a68c9a6d004368920983 | MIT OR Apache-2.0 |

Version skew. Vector depends on vrl by git `branch=main` ([vector Cargo.toml:267](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/Cargo.toml#L267)) and the lock resolves 0.39.0 at 7738f74e039df4d92ee5370d1f0d7d522f73e97a ([vector Cargo.lock:13892-13894](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/Cargo.lock#L13892-L13894)). OpenObserve depends on `vrl = "0.31"` from crates.io ([openobserve Cargo.toml:604-609](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/Cargo.toml#L604-L609)), locked at 0.31.0. "Upstream VRL semantics" therefore means two versions in the target pipeline. A differential test must pin both.

License note. Vector and vrl are MPL-2.0. OpenObserve is AGPL-3.0, so it is reused as a deployed service, not linked.

## 2. A: External reuse census

### 2.1 Datadog Agent to Vector

Agent redirect keys: `observability_pipelines_worker.{metrics,logs,traces}.{enabled,url}` and `vector.*` ([datadog-agent pkg/config/setup/generatedsettings/all_settings.go:1214-1222](https://github.com/DataDog/datadog-agent/blob/d92d1f35a782d230bb2fc9ddbb0ef03c71893cca/pkg/config/setup/generatedsettings/all_settings.go#L1214-L1222), [datadog-agent pkg/config/setup/generatedsettings/all_settings.go:2452-2460](https://github.com/DataDog/datadog-agent/blob/d92d1f35a782d230bb2fc9ddbb0ef03c71893cca/pkg/config/setup/generatedsettings/all_settings.go#L2452-L2460)).

| signal | agent side | status |
|---|---|---|
| metrics | OPW first, vector fallback ([datadog-agent comp/forwarder/defaultforwarder/impl/default_forwarder.go:119-125](https://github.com/DataDog/datadog-agent/blob/d92d1f35a782d230bb2fc9ddbb0ef03c71893cca/comp/forwarder/defaultforwarder/impl/default_forwarder.go#L119-L125)). Resolver registers vector destinations for series v1/v2/v3 and sketches ([datadog-agent comp/forwarder/defaultforwarder/resolver/domain_resolver.go:438-449](https://github.com/DataDog/datadog-agent/blob/d92d1f35a782d230bb2fc9ddbb0ef03c71893cca/comp/forwarder/defaultforwarder/resolver/domain_resolver.go#L438-L449)). `vectorSeriesUseV3` in [datadog-agent pkg/serializer/metrics.go:46-50](https://github.com/DataDog/datadog-agent/blob/d92d1f35a782d230bb2fc9ddbb0ef03c71893cca/pkg/serializer/metrics.go#L46-L50) and [datadog-agent pkg/serializer/metrics.go:102-106](https://github.com/DataDog/datadog-agent/blob/d92d1f35a782d230bb2fc9ddbb0ef03c71893cca/pkg/serializer/metrics.go#L102-L106). | verified in source |
| logs | keys incl. `dual_ship` / `dual_ship_reliable` ([datadog-agent comp/logs/agent/config/config_keys.go:323-369](https://github.com/DataDog/datadog-agent/blob/d92d1f35a782d230bb2fc9ddbb0ef03c71893cca/comp/logs/agent/config/config_keys.go#L323-L369)) | verified in source |
| traces | keys declared; no consumer found in the sparse checkout (`pkg/trace/writer` has no vector/OPW reference) | UNVERIFIED |

Agent endpoints: `/api/v1/series` ([datadog-agent comp/forwarder/defaultforwarder/endpoints/endpoints.go:14](https://github.com/DataDog/datadog-agent/blob/d92d1f35a782d230bb2fc9ddbb0ef03c71893cca/comp/forwarder/defaultforwarder/endpoints/endpoints.go#L14)), `/api/v2/series` ([datadog-agent comp/forwarder/defaultforwarder/endpoints/endpoints.go:27](https://github.com/DataDog/datadog-agent/blob/d92d1f35a782d230bb2fc9ddbb0ef03c71893cca/comp/forwarder/defaultforwarder/endpoints/endpoints.go#L27)), v3 ([datadog-agent comp/forwarder/defaultforwarder/endpoints/endpoints.go:29-31](https://github.com/DataDog/datadog-agent/blob/d92d1f35a782d230bb2fc9ddbb0ef03c71893cca/comp/forwarder/defaultforwarder/endpoints/endpoints.go#L29-L31)), `/api/beta/sketches` ([datadog-agent comp/forwarder/defaultforwarder/endpoints/endpoints.go:37](https://github.com/DataDog/datadog-agent/blob/d92d1f35a782d230bb2fc9ddbb0ef03c71893cca/comp/forwarder/defaultforwarder/endpoints/endpoints.go#L37)). Traces writer: [datadog-agent pkg/trace/writer/trace.go:26](https://github.com/DataDog/datadog-agent/blob/d92d1f35a782d230bb2fc9ddbb0ef03c71893cca/pkg/trace/writer/trace.go#L26). Stats writer: [datadog-agent pkg/trace/writer/stats.go:31](https://github.com/DataDog/datadog-agent/blob/d92d1f35a782d230bb2fc9ddbb0ef03c71893cca/pkg/trace/writer/stats.go#L31).

Payload definitions: `MetricType` UNSPECIFIED/COUNT/RATE/GAUGE ([agent-payload proto/metrics/agent_payload.proto:31-36](https://github.com/DataDog/agent-payload/blob/b88220d9592d43ed8a5a95a3ea90255bc2ce1f91/proto/metrics/agent_payload.proto#L31-L36)), `MetricSeries` interval ([agent-payload proto/metrics/agent_payload.proto:67](https://github.com/DataDog/agent-payload/blob/b88220d9592d43ed8a5a95a3ea90255bc2ce1f91/proto/metrics/agent_payload.proto#L67)), `SketchPayload` ([agent-payload proto/metrics/agent_payload.proto:90](https://github.com/DataDog/agent-payload/blob/b88220d9592d43ed8a5a95a3ea90255bc2ce1f91/proto/metrics/agent_payload.proto#L90)). v3 `metricType` includes DDSketch, ExplicitHistogram, ExponentialHistogram ([agent-payload proto/metrics/intake_v3.proto:120-128](https://github.com/DataDog/agent-payload/blob/b88220d9592d43ed8a5a95a3ea90255bc2ce1f91/proto/metrics/intake_v3.proto#L120-L128)). Logs: `proto/logs/agent_logs_payload.proto`. agent-payload has no traces proto; Vector uses its own `lib/datadog-proto`.

Vector `datadog_agent` source endpoints:

| route | decode | status |
|---|---|---|
| `/v1/input/..`, `/api/v2/logs/..` ([vector src/sources/datadog_agent/logs.rs:31](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/logs.rs#L31)) | JSON `Vec<LogMsg>` ([vector src/sources/datadog_agent/logs.rs:83](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/logs.rs#L83)) | served |
| `/api/v1/series` ([vector src/sources/datadog_agent/metrics.rs:106](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics.rs#L106)) | JSON ([vector src/sources/datadog_agent/metrics.rs:580](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics.rs#L580)) | served |
| `/api/v2/series` ([vector src/sources/datadog_agent/metrics.rs:146](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics.rs#L146)) | protobuf `MetricPayload` ([vector src/sources/datadog_agent/metrics.rs:355](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics.rs#L355)) | served |
| `/api/beta/sketches` ([vector src/sources/datadog_agent/metrics.rs:69](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics.rs#L69)) | protobuf `SketchPayload` ([vector src/sources/datadog_agent/metrics.rs:768](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics.rs#L768)) | served |
| `/api/intake/metrics/v3/series` ([vector src/sources/datadog_agent/metrics.rs:183](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics.rs#L183), [vector src/sources/datadog_agent/metrics/v3.rs:54](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics/v3.rs#L54)) | v3 payload; a v3 payload containing a sketch is rejected ([vector src/sources/datadog_agent/metrics/v3.rs:572](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics/v3.rs#L572)) | served, with that rejection |
| `/api/v0.2/traces` ([vector src/sources/datadog_agent/traces/handler.rs:49](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/traces/handler.rs#L49), [vector src/sources/datadog_agent/traces/handler.rs:111](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/traces/handler.rs#L111)) | protobuf | served |
| `/api/v0.2/stats` ([vector src/sources/datadog_agent/apm_stats.rs:22](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/apm_stats.rs#L22)) | not recorded | only with `multiple_outputs` ([vector src/sources/datadog_agent/mod.rs:286-288](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/mod.rs#L286-L288)) |
| `/api/v2/llmobs` ([vector src/sources/datadog_agent/llmobs.rs:30](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/llmobs.rs#L30)) | not recorded | served |
| check_run, `/api/v1/events`, `/api/v1/metadata`, `/api/v1/validate`, v0.4/v0.5 traces | n/a | not served |

Metric type mapping in the source:

| Datadog type | MetricKind | MetricValue | citation |
|---|---|---|---|
| COUNT | Incremental | Counter | [vector src/sources/datadog_agent/metrics.rs:463-469](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics.rs#L463-L469) |
| GAUGE | Absolute | Gauge | [vector src/sources/datadog_agent/metrics.rs:484-490](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics.rs#L484-L490) |
| RATE | Incremental | Counter, value multiplied by interval, `interval_ms` set | [vector src/sources/datadog_agent/metrics.rs:506-530](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics.rs#L506-L530) |
| UNSPECIFIED | dropped with a warning | none | [vector src/sources/datadog_agent/metrics.rs:532-535](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics.rs#L532-L535) |

RATE semantics are converted, not preserved. The v1 JSON path uses the same mapping ([vector src/sources/datadog_agent/metrics.rs:674-721](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics.rs#L674-L721)).

Source config `DatadogAgentConfig` ([vector src/sources/datadog_agent/mod.rs:87](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/mod.rs#L87)): `store_api_key` (:98), `enabled_data_types` (:106), `multiple_outputs` (:145; outputs `.logs .metrics .traces .apm_stats`), `parse_ddtags` (:150), `split_metric_namespace` (:157), `log_namespace` (:162), `acknowledgements` (:173), `send_timeout_secs` (:188). Content-Encoding identity/gzip/zstd/deflate, otherwise 415 ([vector src/sources/datadog_agent/mod.rs:690-748](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/mod.rs#L690-L748)). The API key is placed in event metadata ([vector src/sources/datadog_agent/mod.rs:581-621](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/mod.rs#L581-L621)).

### 2.2 Vector event model

- `Event` is `Log`, `Metric` or `Trace` ([vector lib/vector-core/src/event/mod.rs:57-61](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/vector-core/src/event/mod.rs#L57-L61)). `Value` is re-exported from vrl ([vector lib/vector-core/src/event/mod.rs:23](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/vector-core/src/event/mod.rs#L23)).
- `MetricKind` is `Incremental` or `Absolute` ([vector lib/vector-core/src/event/metric/mod.rs:533-538](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/vector-core/src/event/metric/mod.rs#L533-L538)).
- `MetricValue`: Counter, Gauge, Set, Distribution, AggregatedHistogram, AggregatedSummary, Sketch ([vector lib/vector-core/src/event/metric/value.rs:16-86](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/vector-core/src/event/metric/value.rs#L16-L86)).
- `EventMetadata`: value, secrets, finalizers, `source_id`, `schema_definition` ([vector lib/vector-core/src/event/metadata.rs:80-113](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/vector-core/src/event/metadata.rs#L80-L113)); `datadog_api_key` ([vector lib/vector-core/src/event/metadata.rs:298](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/vector-core/src/event/metadata.rs#L298)).
- Finalizers: `EventFinalizers` ([vector lib/vector-common/src/finalization.rs:17](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/vector-common/src/finalization.rs#L17)), `BatchNotifier` (:261), `BatchStatus` (:350), `EventStatus` (:385).

### 2.3 The VRL crate

- Parser: `vrl::parser::parse` ([vrl src/parser/mod.rs:40](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/parser/mod.rs#L40)) returns a public AST. `ast::Expr` ([vrl src/parser/ast.rs:225](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/parser/ast.rs#L225)) has Literal, Container, IfStatement, Op, Assignment, Query, FunctionCall, Variable, Unary, Abort, Return, Break. Also `Opcode` (:711), `Query{target,path}` (:950), `QueryTarget` Internal/External(PathPrefix)/FunctionCall/Container (:968).
- Compiler: `compile` ([vrl src/compiler/mod.rs:76](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/compiler/mod.rs#L76)), `compile_with_external(source, fns, &ExternalEnv, CompileConfig)` (around line 103; approximate, so unanchored: [vrl src/compiler/mod.rs](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/compiler/mod.rs)), `compile_with_state` ([vrl src/compiler/mod.rs:135](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/compiler/mod.rs#L135)). `ExternalEnv::new_with_kind(target: Kind, metadata: Kind)` ([vrl src/compiler/state.rs:134](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/compiler/state.rs#L134)).
- `Program.expressions` is `pub(crate)` ([vrl src/compiler/program.rs:10](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/compiler/program.rs#L10)). The type-checked tree is not walkable by an external crate. Public surface: `info()` returning `ProgramInfo{fallible, abortable, target_queries, target_assignments}` ([vrl src/compiler/program.rs:47-66](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/compiler/program.rs#L47-L66)) and `final_type_info` (:23).
- Runtime: `Runtime::resolve(&mut self, target: &mut dyn Target, program, timezone)` ([vrl src/compiler/runtime.rs:121](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/compiler/runtime.rs#L121)); `Target` trait ([vrl src/compiler/target.rs:9](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/compiler/target.rs#L9)); `TargetValue` (:163).
- `Kind` has `null` and `undefined` flags ([vrl src/value/kind.rs:25-39](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/value/kind.rs#L25-L39)). `Value` has no Undefined variant ([vrl src/value/value.rs:41-74](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/value/value.rs#L41-L74)). A missing field query returns `Value::Null` ([vrl src/compiler/expression/query.rs:111-118](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/compiler/expression/query.rs#L111-L118)). `exists()` distinguishes missing from null ([vrl src/stdlib/exists.rs:81-91](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/stdlib/exists.rs#L81-L91)).
- `??` triggers on error, not on null ([vrl src/compiler/expression/op.rs:128-136](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/compiler/expression/op.rs#L128-L136)). E651 reports an unnecessary coalesce when the left side is infallible.
- Fallibility is `CannotFail`, `MightFail` or `AlwaysFails` ([vrl src/compiler/type_def.rs:35-39](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/compiler/type_def.rs#L35-L39)).
- Tests: case directories under `lib/tests` and `src/test`; fuzzing in `lib/fuzz` checks for panics only. No external differential harness exists.

### 2.4 Transforms

Stateless VRL expressions run per event. Stateful Vector transforms hold state across events. They must not be conflated.

| transform | state | key config | citation | role in this plan |
|---|---|---|---|---|
| remap | stateless semantics (the `stateful` flag only covers runner scratch) | `drop_on_error` default false, `drop_on_abort` default true, `reroute_dropped` to output `dropped` | [vector src/transforms/remap.rs:139](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/transforms/remap.rs#L139), [vector src/transforms/remap.rs:152](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/transforms/remap.rs#L152), [vector src/transforms/remap.rs:48](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/transforms/remap.rs#L48), [vector src/transforms/remap.rs:267](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/transforms/remap.rs#L267), compile at [vector src/transforms/remap.rs:229](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/transforms/remap.rs#L229) | normalization, event id, drop rules; runs in Vector only |
| filter | stateless | condition: IsLog, IsMetric, IsTrace, Vrl, DatadogSearch | [vector src/conditions/mod.rs:26-40](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/conditions/mod.rs#L26-L40) | routing |
| reduce | stateful | `group_by`, `merge_strategies`, `expire_after_ms`, `ends_when`/`starts_when` | [vector src/transforms/reduce/config.rs:32-102](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/transforms/reduce/config.rs#L32-L102) | stays in Vector |
| aggregate | stateful | `interval_ms`; modes Auto, Sum, Latest, Count, Diff, Max, Min, Mean, Stdev | [vector src/transforms/aggregate/config.rs:210-225](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/transforms/aggregate/config.rs#L210-L225) | stays in Vector. Auto sums Incremental and keeps latest Absolute ([vector src/transforms/aggregate/transform.rs:175-190](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/transforms/aggregate/transform.rs#L175-L190)); some mode/kind pairs pass through unchanged ([vector src/transforms/aggregate/transform.rs:216-232](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/transforms/aggregate/transform.rs#L216-L232)) |
| dedupe | stateful, LRU | not recorded | [vector src/transforms/dedupe/timed_transform.rs:5](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/transforms/dedupe/timed_transform.rs#L5) | not used; event-id dedup happens at the receiver |
| throttle | stateful | not recorded | no line recorded | not used |
| incremental_to_absolute | stateful | not recorded | no line recorded | not used in slice 1 |
| log_to_metric, metric_to_log | stateless | not recorded | no line recorded | not used in slice 1 |

On remap error ([vector src/transforms/remap.rs:596-665](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/transforms/remap.rs#L596-L665)): with `drop_on_error=false` the original, unmutated event is forwarded on the main output. With `reroute_dropped` the event goes to `dropped`, tagged reason "abort" or "error".

### 2.5 Sinks and delivery

- `http` sink: `HttpSinkConfig` ([vector src/sinks/http/config.rs:49](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sinks/http/config.rs#L49)) with uri, encoding plus framing, batch, request, acknowledgements. JSON with newline framing yields `application/x-ndjson` ([vector src/sinks/http/config.rs:385-389](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sinks/http/config.rs#L385-L389)).
- Codecs include Json, NativeJson, Otlp, Protobuf, ArrowStream, Parquet ([vector lib/codecs/src/encoding/serializer.rs:79](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/codecs/src/encoding/serializer.rs#L79), [vector lib/codecs/src/encoding/serializer.rs:102](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/codecs/src/encoding/serializer.rs#L102), [vector lib/codecs/src/encoding/serializer.rs:112](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/codecs/src/encoding/serializer.rs#L112), [vector lib/codecs/src/encoding/serializer.rs:117](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/codecs/src/encoding/serializer.rs#L117), [vector lib/codecs/src/encoding/serializer.rs:169](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/codecs/src/encoding/serializer.rs#L169), [vector lib/codecs/src/encoding/serializer.rs:175](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/codecs/src/encoding/serializer.rs#L175)).
- OpenTelemetry sink is HTTP only ([vector src/sinks/opentelemetry/mod.rs:16-28](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sinks/opentelemetry/mod.rs#L16-L28)).
- No OpenObserve sink: a grep for `openobserve` in Vector returns zero hits. Vector to O2 goes through the `http` sink to `/_json` or `/_multi`, or through the OTLP sink to `/{org}/v1/logs`.
- Buffers: `WhenFull` Block, DropNewest, Overflow ([vector lib/vector-buffers/src/lib.rs:50-75](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/vector-buffers/src/lib.rs#L50-L75)); Memory and DiskV2 ([vector lib/vector-buffers/src/config.rs:226-256](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/vector-buffers/src/config.rs#L226-L256)).
- Batching defaults: max_bytes 10 MB, timeout 1 s ([vector src/sinks/util/batch.rs:51-56](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sinks/util/batch.rs#L51-L56)).
- Retry: `RetryAction` ([vector src/sinks/util/retries.rs:19](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sinks/util/retries.rs#L19)), Fibonacci backoff ([vector src/sinks/util/retries.rs:80](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sinks/util/retries.rs#L80)). HTTP default retries 429, 408 and 5xx except 501 ([vector src/sinks/util/http.rs:183-195](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sinks/util/http.rs#L183-L195)). Tower defaults: adaptive concurrency, 60 s timeout, unlimited attempts ([vector src/sinks/util/service.rs:90-98](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sinks/util/service.rs#L90-L98)).
- Sink acks: `finalizers.update_status` ([vector src/sinks/util/sink.rs:454-460](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sinks/util/sink.rs#L454-L460)).
- End-to-end acks: the `datadog_agent` source can acknowledge ([vector src/sources/datadog_agent/mod.rs:525](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/mod.rs#L525)). With acks on it uses a `BatchNotifier` ([vector src/sources/datadog_agent/mod.rs:765](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/mod.rs#L765)). Delivered maps to 200, Errored to 500, Rejected to 400, timeout to 503 ([vector src/sources/datadog_agent/mod.rs:776-800](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/mod.rs#L776-L800)). End-to-end acknowledgement from Agent through to the sink is therefore available.
- OTLP source: HTTP POST `/v1/{logs,metrics,traces}` protobuf only ([vector src/sources/opentelemetry/http.rs:228-233](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/opentelemetry/http.rs#L228-L233)), plus gRPC on 4317.

### 2.6 OpenObserve

Ingest routes (all in [openobserve src/api/http/src/handler/http/router/mod.rs](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs)):

| route | line |
|---|---|
| `_bulk` | [openobserve src/api/http/src/handler/http/router/mod.rs:1438](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs#L1438) |
| `{stream}/_multi` | [openobserve src/api/http/src/handler/http/router/mod.rs:1439](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs#L1439) |
| `{stream}/_json` | [openobserve src/api/http/src/handler/http/router/mod.rs:1440](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs#L1440) |
| `_hec` | [openobserve src/api/http/src/handler/http/router/mod.rs:1441](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs#L1441) |
| Loki push | [openobserve src/api/http/src/handler/http/router/mod.rs:1442](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs#L1442) |
| OTLP `/{org}/v1/logs` | [openobserve src/api/http/src/handler/http/router/mod.rs:1443](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs#L1443) |
| OTLP `/v1/metrics` | [openobserve src/api/http/src/handler/http/router/mod.rs:1444](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs#L1444) |
| OTLP `/v1/traces` | [openobserve src/api/http/src/handler/http/router/mod.rs:1448](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs#L1448) |
| metrics `_json` | [openobserve src/api/http/src/handler/http/router/mod.rs:1483](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs#L1483) |
| Prometheus remote write | [openobserve src/api/http/src/handler/http/router/mod.rs:1486](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs#L1486) |

VRL in OpenObserve: `compile_with_external` ([openobserve src/transform/src/lib.rs:254-265](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/transform/src/lib.rs#L254-L265)), `runtime.resolve` ([openobserve src/transform/src/lib.rs:355-358](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/transform/src/lib.rs#L355-L358)). The function table is `stdlib::all()` minus `DENIED_VRL_FUNCTIONS` (http_request, dns_lookup, get_env_var, get_hostname, parse_proto, and others; [openobserve src/transform/src/lib.rs:67-80](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/transform/src/lib.rs#L67-L80), [openobserve src/transform/src/lib.rs:482-492](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/transform/src/lib.rs#L482-L492)). VRL also runs as a DataFusion UDF in search ([openobserve src/search/src/datafusion/udf/transform_udf.rs:95](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/search/src/datafusion/udf/transform_udf.rs#L95), [openobserve src/search/src/datafusion/udf/transform_udf.rs:152](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/search/src/datafusion/udf/transform_udf.rs#L152)). No custom functions.

Pipelines: `Pipeline` ([openobserve src/config/src/meta/pipeline/mod.rs:64-80](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/config/src/meta/pipeline/mod.rs#L64-L80)). Node types: RemoteStream, Stream, Query, Function, Condition, LlmEvaluation, WorkflowTrigger, Destination, Branch ([openobserve src/config/src/meta/pipeline/components.rs:264-276](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/config/src/meta/pipeline/components.rs#L264-L276)); `Edge` (:214). Execution code is under `src/core/src/pipeline/`.

Search: `POST /{org}/_search` ([openobserve src/api/http/src/handler/http/router/mod.rs:1497](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs#L1497)) takes `sql`, `start_time`/`end_time` in microseconds, `from`, `size` ([openobserve src/config/src/meta/search.rs:229-242](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/config/src/meta/search.rs#L229-L242)). `_search_partition` ([openobserve src/api/http/src/handler/http/router/mod.rs:1499](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs#L1499)). `_search_stream` ([openobserve src/api/http/src/handler/http/router/mod.rs:1506](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs#L1506)). There is no bulk export or replay endpoint; only alerts export exists ([openobserve src/api/http/src/handler/http/router/mod.rs:1576](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/api/http/src/handler/http/router/mod.rs#L1576)). Historical recovery means time-partitioned `_search` pagination.

## 3. B: Internal reuse census

Tags: EXISTS-TESTED, EXISTS-UNTESTED, STUB, DOC-ONLY, NOT FOUND. Test counts are `#[test]` / `#[tokio::test]` attributes in the source at the pinned commit; no suite was run for this census, so EXISTS-TESTED means a falsifiable test exists, not that it passed.

| component | what exists | tag | citation |
|---|---|---|---|
| lance-graph-vrl | Own parser `parse(src) -> Result<Vec<Stmt>, Refusal>`. Grammar: `.out = .path`, `del(.out)`, `if .path <op> <i32> { abort }`, text `==`/`!=` only. Computing constructs refused by name (`Refusal::Computes`). Lowers to one mask-risc Program over one DT64 `LaneBinding::borrow_all`. Terminals: compile, `run_into`, `count`, `sum`, `materialize_rows`, `rows_at`. Absent = `(value==0) & !presence`; a drop term over an absent value is UNDECIDED and counted as `unknown`. 3 lib tests, 11 in tests/intake.rs. Not a dependent of upstream `vrl`. | EXISTS-TESTED | [lance-graph crates/lance-graph-vrl/src/lib.rs:60-80](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-vrl/src/lib.rs#L60-L80), [lance-graph crates/lance-graph-vrl/src/lib.rs:376](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-vrl/src/lib.rs#L376), [lance-graph crates/lance-graph-vrl/src/lib.rs:14-22](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-vrl/src/lib.rs#L14-L22), [lance-graph crates/lance-graph-vrl/src/lib.rs:491](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-vrl/src/lib.rs#L491), [lance-graph crates/lance-graph-vrl/src/lib.rs:805](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-vrl/src/lib.rs#L805), [lance-graph crates/lance-graph-vrl/src/lib.rs:893](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-vrl/src/lib.rs#L893), [lance-graph crates/lance-graph-vrl/src/lib.rs:924](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-vrl/src/lib.rs#L924), [lance-graph crates/lance-graph-vrl/src/lib.rs:941](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-vrl/src/lib.rs#L941), [lance-graph crates/lance-graph-vrl/src/lib.rs:987](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-vrl/src/lib.rs#L987) |
| vrl-recipe-probe | Lab crate in its own `[workspace]`. Naive reference interpreter (`interpret`, `interpret_present`) shares the parser with Recipe. Tests: recipe 16, presence 6, alloc 2, r2il_parity 1. Parity is against its own interpreter and r2il, not upstream VRL. Differential against upstream VRL: NOT FOUND. | EXISTS-TESTED (self-parity only) | [lance-graph crates/vrl-recipe-probe/src/lib.rs:1-60](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/vrl-recipe-probe/src/lib.rs#L1-L60), [Cargo.toml:1-6](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/vrl-recipe-probe/Cargo.toml#L1-L6) |
| BindStore, Resolved, MethodBinding | `MethodBinding`, `ClassSchema`, `MethodRef`, `Resolved`, `BindStore` with `register`, `resolve`, `method_ref`, `check_issuer`, `admit`. Refusals: SignatureRevisionMismatch, GuardMismatch, BindingMismatch, ForeignStore. Consumers: mailbox-lanes, quack-boundary-lab, vrl-recipe-probe, xlog-coverage-probe, intake. | EXISTS-TESTED | [lance-graph crates/lance-graph-dt64/src/lib.rs:233](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L233), [lance-graph crates/lance-graph-dt64/src/lib.rs:272](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L272), [lance-graph crates/lance-graph-dt64/src/lib.rs:371](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L371), [lance-graph crates/lance-graph-dt64/src/lib.rs:681](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L681), [lance-graph crates/lance-graph-dt64/src/lib.rs:807](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L807), [lance-graph crates/lance-graph-dt64/src/lib.rs:849](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L849), [lance-graph crates/lance-graph-dt64/src/lib.rs:1108](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L1108), [lance-graph crates/lance-graph-dt64/src/lib.rs:1176](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L1176), [lance-graph crates/lance-graph-dt64/src/lib.rs:1201](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L1201), [lance-graph crates/lance-graph-dt64/src/lib.rs:1239](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L1239), [lance-graph crates/lance-graph-dt64/src/lib.rs:581](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L581), [lance-graph crates/lance-graph-dt64/src/lib.rs:592](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L592), [lance-graph crates/lance-graph-dt64/src/lib.rs:597](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L597), [lance-graph crates/lance-graph-dt64/src/lib.rs:600](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L600) |
| ClassView | Trait with `fields`, `lens_fields`, `fields_at`, `field_label`, `field_count`; `FieldMask`, `WideFieldMask`. 63 tests in the file. | EXISTS-TESTED | [lance-graph crates/lance-graph-contract/src/class_view.rs:1057](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-contract/src/class_view.rs#L1057), [lance-graph crates/lance-graph-contract/src/class_view.rs:1084](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-contract/src/class_view.rs#L1084), [lance-graph crates/lance-graph-contract/src/class_view.rs:1105](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-contract/src/class_view.rs#L1105), [lance-graph crates/lance-graph-contract/src/class_view.rs:70](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-contract/src/class_view.rs#L70), [lance-graph crates/lance-graph-contract/src/class_view.rs:243](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-contract/src/class_view.rs#L243) |
| DT64 lane socket (D-LANESOCK-1) | `LaneSource`, `SelectionSource`, `PresenceSource`, `LaneBinding`, `borrow_all`. 31 tests in `tests/lane_socket.rs`. No type named `LaneSocket`. | EXISTS-TESTED | [lance-graph crates/lance-graph-dt64/src/lane.rs:54](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lane.rs#L54), [lance-graph crates/lance-graph-dt64/src/lane.rs:113](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lane.rs#L113), [lance-graph crates/lance-graph-dt64/src/lane.rs:140](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lane.rs#L140), [lance-graph crates/lance-graph-dt64/src/lane.rs:259](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lane.rs#L259), [lance-graph crates/lance-graph-dt64/src/lane.rs:655](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lane.rs#L655) |
| DR layer ("DR64") | `lance-graph-dt64::dr`. "Nothing here holds a value." `RouteLog`, `FrameStore`, `Frame`, `LocalMap`, `MapTable`; `PairPlane` in `dr/pair.rs`. "DR64" appears only as the extent label `DR64_CUBE`. No separate DR64 crate. | EXISTS-TESTED (46 in tests/dr.rs) | [lance-graph crates/lance-graph-dt64/src/dr.rs:109](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/dr.rs#L109), [lance-graph crates/lance-graph-dt64/src/dr.rs:213](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/dr.rs#L213), [lance-graph crates/lance-graph-dt64/src/dr.rs:382](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/dr.rs#L382), [lance-graph crates/lance-graph-dt64/src/dr.rs:489](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/dr.rs#L489), [lance-graph crates/lance-graph-dt64/src/dr.rs:576](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/dr.rs#L576), [lance-graph crates/lance-graph-dt64/src/dr.rs:874](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/dr.rs#L874), [lance-graph crates/lance-graph-dt64/src/dr/pair.rs:50](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/dr/pair.rs#L50) |
| dt64-lance | Durable `MapTable`. `tests/map_log.rs` contains zero `#[test]`. | EXISTS-UNTESTED | [lance-graph crates/lance-graph-dt64-lance/tests/map_log.rs](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64-lance/tests/map_log.rs) |
| contract CodeSpace, AbsentAs, is_absent | `CodeSpace`; `AbsentAs {Null, EmptyText}`; `is_absent(present, value) = !present && value==0`; `CodeText`; `write_cell`. No `RowType` identifier exists. | EXISTS-TESTED (pg_row.rs 5, lane_codec.rs 7) | [lance-graph crates/lance-graph-contract/src/lane_codec.rs:80](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-contract/src/lane_codec.rs#L80), [lance-graph crates/lance-graph-contract/src/pg_row.rs:168](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-contract/src/pg_row.rs#L168), [lance-graph crates/lance-graph-contract/src/pg_row.rs:234](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-contract/src/pg_row.rs#L234), [lance-graph crates/lance-graph-contract/src/pg_row.rs:215](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-contract/src/pg_row.rs#L215), [lance-graph crates/lance-graph-contract/src/pg_row.rs:262](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-contract/src/pg_row.rs#L262) |
| D-RSP-5 field seam | One seam for Quack and VRL: `predicate_iri -> (column, code space, exponent, validity)`, owned by the source owner (intake). Operator decision 2026-11-25. Seam itself is a document; pieces exist (`Table::literal_code`, vrl `tests/intake.rs:267`). Status board: "In PR (partial)". | DOC-ONLY | [lance-graph .claude/plans/registered-source-profiles-v1.md:110-155](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/.claude/plans/registered-source-profiles-v1.md#L110-L155), [lance-graph .claude/board/STATUS_BOARD.md:3659](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/.claude/board/STATUS_BOARD.md#L3659), [lance-graph crates/lance-graph-intake/src/lib.rs:720](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-intake/src/lib.rs#L720), [lance-graph crates/lance-graph-vrl/tests/intake.rs:267](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-vrl/tests/intake.rs#L267) |
| lance-graph-intake | `Table`, `from_rows`, `append` returning `AppendReport`, `select_all`, `select_where`, `left_join`, `delete_key`, `code_for`; `detect_column`. 42 tests. Rows "as PostgreSQL prints them", shape-detected class, lanes plus DR key map. This is the existing typed intake seam. | EXISTS-TESTED | [lance-graph crates/lance-graph-intake/src/lib.rs:477](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-intake/src/lib.rs#L477), [lance-graph crates/lance-graph-intake/src/lib.rs:505](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-intake/src/lib.rs#L505), [lance-graph crates/lance-graph-intake/src/lib.rs:1162](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-intake/src/lib.rs#L1162), [lance-graph crates/lance-graph-intake/src/lib.rs:803](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-intake/src/lib.rs#L803), [lance-graph crates/lance-graph-intake/src/lib.rs:812](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-intake/src/lib.rs#L812), [lance-graph crates/lance-graph-intake/src/lib.rs:1038](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-intake/src/lib.rs#L1038), [lance-graph crates/lance-graph-intake/src/lib.rs:1026](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-intake/src/lib.rs#L1026), [lance-graph crates/lance-graph-intake/src/lib.rs:708](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-intake/src/lib.rs#L708), [lance-graph crates/lance-graph-intake/src/detect.rs:166](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-intake/src/detect.rs#L166) |
| lance-graph-xlog cube | Resident hypercube of binary relations on one tile; not an OLAP cube (no CUBE/ROLLUP/GROUPING SETS). Semi-naive closure; `DeltaLog` with ops ADD=1/RETIRE=0 only; `add_edge`, `retire_edge`; refresh equals rebuild. Incremental maintenance of aggregates: NOT FOUND. No XLOG dependency. | EXISTS-TESTED | [lance-graph crates/lance-graph-xlog/src/lib.rs:1-9](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-xlog/src/lib.rs#L1-L9), [lance-graph crates/lance-graph-xlog/src/cube.rs:1-30](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-xlog/src/cube.rs#L1-L30), [lance-graph crates/lance-graph-xlog/src/wide.rs:295-296](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-xlog/src/wide.rs#L295-L296), [lance-graph crates/lance-graph-xlog/src/wide.rs:389](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-xlog/src/wide.rs#L389), [lance-graph crates/lance-graph-xlog/src/wide.rs:418](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-xlog/src/wide.rs#L418), [lance-graph crates/lance-graph-xlog/src/wide.rs:415-416](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-xlog/src/wide.rs#L415-L416), [lance-graph crates/lance-graph-xlog/src/cube.rs:1182](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-xlog/src/cube.rs#L1182), [lance-graph crates/lance-graph-xlog/src/cube.rs:1203](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-xlog/src/cube.rs#L1203), [lance-graph crates/lance-graph-xlog/tests/wide_closure.rs:317](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-xlog/tests/wide_closure.rs#L317), [lance-graph crates/lance-graph-xlog/src/cube.rs:1334](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-xlog/src/cube.rs#L1334), [lance-graph crates/lance-graph-xlog/src/wide.rs:510](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-xlog/src/wide.rs#L510) |
| xlog-cube (XLOG repo) | `CubeExecutor` runs an XLOG `ExecutionPlan` on `lance_graph_xlog::cube::Cube`. Semi-naive SCCs, binary u32 relations only, Route C recipe. Tests: closure_parity, negation, plans, recipes, route_c, sets, soundness. | EXISTS-TESTED | [xlog crates/xlog-cube](https://github.com/AdaWorldAPI/xlog/tree/28b41f5e5c74eb915c89a68c9a6d004368920983/crates/xlog-cube) (directory) |
| mask-risc GroupReduce | `GroupFold` Count, MinI32, MaxI32, SumSymI32; `Terminal::GroupReduce`; `GroupKey` Lane/Via/Pair; `merge_group_sink` applies each fold's own law. Group-by lowered only in a lab test (`xlog-coverage-probe tests/group_by.rs:1-16`, `:282`: `sales_by_cat_region(sum)` to `GroupKey::Pair` and `SumSymI32`). | EXISTS-TESTED (lab test only) | [lance-graph crates/lance-graph-mask-risc/src/ir.rs:453](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-mask-risc/src/ir.rs#L453), [lance-graph crates/lance-graph-mask-risc/src/ir.rs:501](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-mask-risc/src/ir.rs#L501), [lance-graph crates/lance-graph-mask-risc/src/ir.rs:521-538](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-mask-risc/src/ir.rs#L521-L538), [lance-graph crates/lance-graph-mask-risc/src/ir.rs:674](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-mask-risc/src/ir.rs#L674), [lance-graph crates/lance-graph-mask-risc/src/exec.rs:1368-1372](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-mask-risc/src/exec.rs#L1368-L1372) |
| Quack | Rust builder, no parser. `Query{filter,agg}`; lowers to `mask_risc::Program`; "may build a Program and must never evaluate one". `Filter::sql_where` three-valued. `Agg`: Count, Any, All, SumI32, MinI32, MaxI32, GroupReduce, GroupSumI32; AVG = SUM/COUNT. DuckDB differential in `tests/duckdb_differential.rs` (oracle.py, cases.tsv). | EXISTS-TESTED | [lance-graph crates/lance-graph-quack/src/lib.rs:1311](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-quack/src/lib.rs#L1311), [lance-graph crates/lance-graph-quack/src/lib.rs:1393](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-quack/src/lib.rs#L1393), [lance-graph crates/lance-graph-quack/src/lib.rs:550](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-quack/src/lib.rs#L550), [lance-graph crates/lance-graph-quack/src/lib.rs:1137](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-quack/src/lib.rs#L1137), [lance-graph crates/lance-graph-quack/src/lib.rs:1273](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-quack/src/lib.rs#L1273), [lance-graph crates/lance-graph-quack/src/lib.rs:1288](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-quack/src/lib.rs#L1288), [lance-graph crates/lance-graph-quack/src/lib.rs:1572](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-quack/src/lib.rs#L1572) |
| lane-fold law metadata | `Hom` Count, Sum, Min, Max, Exists with law metadata: associative and commutative; Min, Max, Exists idempotent; Count and Sum not idempotent; none invertible. Also `FoldState` in report and `Measure` in relalg. | EXISTS-TESTED (lane-fold 14) | [lance-graph crates/lance-graph-lane-fold/src/query.rs:44-80](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-lane-fold/src/query.rs#L44-L80), [lance-graph crates/lance-graph-report/src/plan.rs:133-186](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-report/src/plan.rs#L133-L186), [lance-graph crates/lance-graph-relalg/src/ir.rs:99-111](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-relalg/src/ir.rs#L99-L111) |
| PmLedger | Signed counter add/absorb. The only signed (invertible) accumulator found. Lives in the `xlog-relcomp-probe` lab. | EXISTS-TESTED (lab, 10 in tests/pm_ledger.rs) | [lance-graph crates/xlog-relcomp-probe/src/pm_ledger.rs:183](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/xlog-relcomp-probe/src/pm_ledger.rs#L183), [:216](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/xlog-relcomp-probe/src/pm_ledger.rs#L216) |
| pg-parity-harness | Drives `psql`; SKIPs without a server. Also `qsp_agg_pg_parity` example against PostgreSQL. | EXISTS (skips without a server) | [lance-graph crates/pg-parity-harness](https://github.com/AdaWorldAPI/lance-graph/tree/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/pg-parity-harness) (directory) |
| ogar-flow-m365 (PowerShell polyfill) | "PowerShell-shaped command labels over the ogar-loco call ABI", e.g. `Get-MgUser -UserId ... \| Get-IamLicenseEvidence \| Select-Object ... \| Export-Report`. `frontend::compile` produces `ogar_loco::Program`, then `DirectoryObservations`, then `Report::to_json`. | EXISTS-TESTED (31) | [OGAR crates/ogar-flow-m365/src/frontend.rs:334](https://github.com/AdaWorldAPI/OGAR/blob/ac1441427f166d630e488e934a37742495935bbc/crates/ogar-flow-m365/src/frontend.rs#L334) |
| ogar-az | Typed Graph user mapping to `DirRecord`. WHO = Entra object id, never objectGUID. `$select` derived from SCHEMA_V1. No HTTP. Examples `az_ingest.rs`, `mailbox_ingest.rs`; modules group, license, mailbox. | EXISTS-TESTED (48) | [OGAR crates/ogar-az](https://github.com/AdaWorldAPI/OGAR/tree/ac1441427f166d630e488e934a37742495935bbc/crates/ogar-az) (directory) |
| sentinel-core | `propose()`; `ProposalLedger::stage` is in-memory dedup over `TicketKey`. `projection.rs` `lower_psobject_v1`: POC PSCustomObject-like lowering (PropertyValue Text/Integer/Bool/Null) to typed `EventProjectionV1`, schema_version 1; `retention_dry_run`. | EXISTS-TESTED (25; projection is POC) | [sentinel-rs crates/sentinel-core/src/lib.rs:114](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-core/src/lib.rs#L114), [sentinel-rs crates/sentinel-core/src/lib.rs:200](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-core/src/lib.rs#L200), [sentinel-rs crates/sentinel-core/src/lib.rs:67](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-core/src/lib.rs#L67), [sentinel-rs crates/sentinel-core/src/projection.rs:100](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-core/src/projection.rs#L100), [sentinel-rs crates/sentinel-core/src/projection.rs:289](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-core/src/projection.rs#L289) |
| sentinel-o2 | OpenObserve adapter, read-only ("no write, no ingest and no action"). `O2Transport` trait with no HTTP client in the crate. `plan()` builds SQL with filters; window max 24 h, size max 1000. `parse_row` refuses foreign tenant labels and any grade other than "fixture". Digest collect/readback/reconcile with `SETTLE_US` 300 s. Live HTTP only in `#[ignore]` tests. | EXISTS-TESTED (44; live tests `#[ignore]`) | [sentinel-rs crates/sentinel-o2/src/lib.rs:6-7](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/lib.rs#L6-L7), [sentinel-rs crates/sentinel-o2/src/lib.rs:263](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/lib.rs#L263), [sentinel-rs crates/sentinel-o2/src/lib.rs:171-187](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/lib.rs#L171-L187), [sentinel-rs crates/sentinel-o2/src/lib.rs:97](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/lib.rs#L97), [sentinel-rs crates/sentinel-o2/src/lib.rs:169](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/lib.rs#L169), [sentinel-rs crates/sentinel-o2/src/lib.rs:368](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/lib.rs#L368), [sentinel-rs crates/sentinel-o2/src/lib.rs:371](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/lib.rs#L371), [sentinel-rs crates/sentinel-o2/src/lib.rs:420](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/lib.rs#L420), [sentinel-rs crates/sentinel-o2/src/digest.rs:436](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/digest.rs#L436), [sentinel-rs crates/sentinel-o2/src/digest.rs:514](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/digest.rs#L514), [sentinel-rs crates/sentinel-o2/src/digest.rs:542](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/digest.rs#L542), [sentinel-rs crates/sentinel-o2/src/digest.rs:57](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/digest.rs#L57), [sentinel-rs crates/sentinel-o2/tests/live_oracle.rs:57](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/tests/live_oracle.rs#L57) |
| sentinel-telemetry | `ProcessObservation`; `SourceIdentity{tenant_id, source_system, object_id}`, "a source-scoped key, not a cross-source identity claim"; `TenantBinding{customer_code, tenant_alias}` plus Entra GUID. Live `admit()` always returns `LiveAdapterNotImplemented`. | STUB (live path) | [sentinel-rs crates/sentinel-telemetry/src/observation.rs:46-68](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-telemetry/src/observation.rs#L46-L68), [sentinel-rs crates/sentinel-telemetry/src/observation.rs:204](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-telemetry/src/observation.rs#L204), [sentinel-rs crates/sentinel-telemetry/src/live.rs:39-50](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-telemetry/src/live.rs#L39-L50) |
| sentinel-xlog | `Projection::build`: disposable, rebuildable cube; `MAX_IDENTITIES` 64; terminals co_occurrence, reachable_from, absence, reach, revalidate. No incremental behaviour. | EXISTS-TESTED (56) | [sentinel-rs crates/sentinel-xlog/src/lib.rs:714](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-xlog/src/lib.rs#L714), [sentinel-rs crates/sentinel-xlog/src/lib.rs:36](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-xlog/src/lib.rs#L36) |
| sentinel-web | axum 0.7, GET-only routes `/healthz /readyz /api/v1/capabilities /api/v1/observations /soc/o2`. Refuses to start if live credential env names are set. | EXISTS-TESTED (40) | [sentinel-rs crates/sentinel-web/src/lib.rs:70-82](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-web/src/lib.rs#L70-L82), [sentinel-rs crates/sentinel-web/src/config.rs:12](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-web/src/config.rs#L12) |
| time windows, watermarks | None in quack, relalg, xlog, report, lane-fold or mailbox-lanes. | NOT FOUND | n/a |
| read-only data API in lance-graph | Planner `serve` is OpenAI-compatible; cognitive-shader-driver is not read-only; callcenter `postgrest.rs` is a STUB ("NO live HTTP server here"). | NOT FOUND | n/a |
| PowerShell in lance-graph | No PowerShell code in lance-graph. | NOT FOUND | n/a |
| Vector/OpenObserve/Datadog/OTLP in lance-graph code | Zero references outside `.claude/` docs. | NOT FOUND | n/a |

Exists versus planned. What exists: a typed row intake, a lane socket, a bind store, a mask-risc group reduce, and two frontends (Quack, `lance-graph-vrl`). What is planned and absent: the shared D-RSP-5 field seam (document only), an NDJSON intake route, event-id dedup, time windows and watermarks, a read-only aggregate API, any O2 write path. Nothing in lance-graph references Vector, OpenObserve or Datadog.

Drift found:

- The `.mermaid/03` and `.mermaid/05` line citations in lance-graph have drifted (for example BindStore is cited at :772, actual :807), and they name `GuardDropped`/`GuardAdded`, which were not found in code.
- Stale board entry: [lance-graph .claude/board/entries/2026-11-16-g2-dr-virtual-projection-reconcile.md:26](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/.claude/board/entries/2026-11-16-g2-dr-virtual-projection-reconcile.md#L26) says VRL is ABSENT. It predates `lance-graph-vrl`.
- sentinel-rs pin hygiene: `sentinel-iam` and `sentinel-xlog` depend on lance-graph with `branch = main` ([sentinel-rs crates/sentinel-iam/Cargo.toml:18-19](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-iam/Cargo.toml#L18-L19), [sentinel-rs crates/sentinel-xlog/Cargo.toml:14-15](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-xlog/Cargo.toml#L14-L15)). `Cargo.lock` pins 932061a. A floating ref violates the pin rule in this repo's `CLAUDE.md`.
- D-RSP-5 says VRL's `code_for("")` does not match the contract: `''` under EmptyText is code 0 in Quack and has no code in VRL, so VRL is wrong against the contract ([lance-graph .claude/plans/registered-source-profiles-v1.md:110-155](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/.claude/plans/registered-source-profiles-v1.md#L110-L155)).

## 4. C: VRL semantic interoperability

### 4.1 The measured probe

Method. A scratch probe (not committed) compiled and ran programs against vrl at 2d5987e (0.39.0) with features `compiler` and `stdlib`. Two target modes: untyped (`compile`, no schema) and typed (`compile_with_external` with `ExternalEnv::new_with_kind(object{cpu: K, host: bytes})`). Events: a `{cpu:5}`, b `{cpu:1}`, c `{cpu:null}`, d `{}` (missing), e `{cpu:"5"}`, f `{cpu:3.5}`. The probe's full source is in Appendix A.

| program | untyped upstream | typed `cpu: integer` upstream | lance-graph-vrl today |
|---|---|---|---|
| `.out = .host` | compiles, infallible | not measured | admitted |
| `if .cpu > 3 { abort }` | E100 compile error (unhandled fallible) for all events | compiles. a aborted, b kept. c (null) and d (missing): runtime error ("can't calculate remainder of type null and integer"). e ("5"): runtime error ("expected string, got integer") | admitted. Absent gives UNKNOWN |
| same, `cpu: integer or null`, or `integer or undefined` | not applicable | E100 again | not applicable |
| `if .cpu > 3 && .mem > 1 { abort }` | E100 | measured as `.cpu > 3 && .host == "a"`: compiles; a aborted, b kept, null/missing error | refused (`&&`) |
| `if is_integer(.cpu) && .cpu > 3 { abort }` | E100 (type narrowing does not flow through `&&`) | not measured | refused (function call) |
| `if (.cpu ?? 0) > 3 { abort }` | E651 (unnecessary coalesce; `??` catches errors, not null) | not measured | refused (`??`) |
| `if int!(.cpu) > 3 { abort }` | a aborted, b kept; c, d, e, f runtime error ("expected integer, got null, string or float") | not measured | refused (`Computes`) |
| `if .cpu == 3 { abort }` | infallible; all kept (null, missing, "5", 3.5 compare false) | unchanged by typing | admitted. Absent gives UNKNOWN |
| `if .cpu != 3 { abort }` | all aborted, including null and missing | unchanged by typing | admitted. Absent gives UNKNOWN |
| `if .host == "a" { abort }` | a aborted; others kept | not measured | admitted (text `==`) |
| `del(.host)` | compiles | not measured | admitted |
| rename chain `.k = .host; .host = .svc; .z = .k` | in-place mutation honoured (z = old host); assigning missing `.svc` writes `host: null` | not measured | admitted; paths resolve at compile time with the same in-place mutation ([lance-graph crates/lance-graph-vrl/src/lib.rs:60-80](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-vrl/src/lib.rs#L60-L80)); a missing source reads as absent, not as an explicit null |
| `exists(.cpu)`, `is_null(.cpu)` | `exists` true for null, false only for missing; `is_null` true for both | not measured | refused (function calls) |

### 4.2 Consequences

1. The core construct `if .path > N { abort }` is not a valid upstream program unless the field is declared a non-optional integer in the external schema. With that declaration, upstream answers an absent value with a runtime error, while `lance-graph-vrl` answers UNKNOWN. In Vector, an error with `drop_on_error=false` forwards the original event, so an absent row would be kept. With `drop_on_error=true` it is dropped. With `reroute_dropped` it lands in `dropped` with reason "error".
2. For `==` and `!=` upstream is two-valued: absent behaves as null, `!= 3` drops absent rows and `== 3` keeps them. `lance-graph-vrl` and D-RSP-5 ("an absent value never establishes a negative fact") answer UNKNOWN. This is a real divergence. A VRL-compatible lowering must either refuse `==`/`!=` on optional fields unless guarded (`exists(.f) && ...`, which needs `&&`, which `lance-graph-vrl` refuses) or evaluate absent as null. The operator must decide (Q1).
3. Upstream's typed compile is the right admission gate: derive `ExternalEnv` from `ClassView` plus D-RSP-5 validity (non-optional if and only if the binding says established-only). Require upstream compile success and lowering success; refuse otherwise.
4. Lowering must read the public parser AST (`vrl::parser::ast`), because the compiled tree is `pub(crate)`. Replacing the own parser with `vrl::parser::parse` adds a `vrl` dependency (`default-features = false`, feature `compiler`; the parser lives behind that feature). The `stdlib` feature is needed for the function table in the admission compile.
5. Missing and null are indistinguishable to comparisons and `is_null`; only `exists()` separates them. The lance-graph presence plane has one absent state and no distinct explicit-null state, so a JSON null and a missing key both become absent at intake unless the binding adds one. Assigning a missing field yields explicit null in VRL output, while lance-graph Rows report absent.

Side finding. The upstream error text for `>` over null names the wrong operation ("remainder"): `try_gt` returns `ValueError::Rem` ([vrl src/compiler/value/arithmetic.rs:235](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/compiler/value/arithmetic.rs#L235); the variant's message is at [src/compiler/value/error.rs:19](https://github.com/vectordotdev/vrl/blob/2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7/src/compiler/value/error.rs#L19)). It is worth an optional upstream issue to vectordotdev/vrl.

### 4.3 Outcome correspondence

| lance-graph outcome | Vector remap equivalent | condition |
|---|---|---|
| KEEP | main output | always |
| DROP | `abort` with `drop_on_abort=true` (default) | always |
| UNKNOWN | runtime error; visible as `dropped` with reason "error" | only when `reroute_dropped=true`. With `drop_on_error=false` the original event is kept on the main output, so UNKNOWN has no faithful equivalent without rerouting |

### 4.4 Preserve-list

| property to preserve | how it is carried |
|---|---|
| field identity | `ClassView` / `FieldRef` plus the D-RSP-5 seam |
| code space | `CodeSpace` and the exponent in the seam tuple |
| row type | `AbsentAs` and the row-type contract (boolean, NaN, text "" semantics); no `RowType` identifier exists today |
| missing versus established | the presence plane (`absent = (value==0) & !presence`) |
| null | VRL `Kind` null/undefined through `ExternalEnv`; lance-graph has no explicit-null state |
| fallibility and refusal | upstream compile E-codes carried in `Refusal` (planned, S2) |

### 4.5 Placement matrix

| construct | executes in Vector | lowers to lance-graph | unsupported or refused | notes |
|---|---|---|---|---|
| field read, projection | yes | yes, now | | |
| rename (`.out = .path`) | yes | yes, now | | |
| `del(.f)` | yes | yes, now | | |
| `==`, `!=` on integer | yes | now, with divergence on absent values | | Q1 |
| `==`, `!=` on text | yes | now (dictionary code) | | unknown literal handling differs, see D-RSP-5 |
| `<`, `<=`, `>`, `>=` | yes | now, typed non-optional only | | upstream needs non-optional integer |
| `&&` | yes | after S2/S3 | refused today | needed for guarded `==`/`!=`; mask-risc `And` exists ([lance-graph crates/lance-graph-mask-risc/src/ir.rs:231-239](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-mask-risc/src/ir.rs#L231-L239)) |
| `\|\|` | yes | after S2/S3 | refused today | mask-risc `Or` exists |
| `!` | yes | after S2/S3 | refused today | mask-risc `Not` exists; UNKNOWN stays UNKNOWN under `!` (D-RSP-5 test 9) |
| `exists(.f)` | yes | after S2/S3, as a presence test | refused today | |
| `is_null` | yes | no | refused | cannot separate missing from null |
| `??` | yes | no | refused | |
| `int!`, `to_int` | yes | no | refused | computes values |
| arithmetic | yes | no | refused | |
| string functions, `parse_*` | yes | no | refused | |
| `now()` | yes | no | refused | |
| metric and tag mutation | yes | no | refused | |
| reduce, aggregate, dedupe (stateful) | yes | no | n/a | stay in Vector |

Lowerable now: field read and projection, rename, `del`, integer comparison (typed non-optional only), text `==`/`!=`. Lowerable after S2/S3: `&&`, `!`, `exists()` as a presence test. Vector-only: everything that computes values, parsing, coercion and the stateful transforms.

### 4.6 Open decision Q1

Question: VRL compatibility for `==`/`!=` over optional fields. Recommendation: refuse by default, which preserves D-RSP-5. Admit only `exists(.f) && .f != N` once `&&` lands. A separate `VrlCompat` mode (absent as null) is the alternative and is not recommended as default.

## 5. D: Typed telemetry intake

Smallest adapter. A Vector `http` sink with `encoding: json`, newline-delimited framing (`application/x-ndjson`), acknowledgements on, POSTing to one route ([vector src/sinks/http/config.rs:385-389](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sinks/http/config.rs#L385-L389)). The receiver decodes each line once, at the boundary, into the row shape `lance-graph-intake` already takes (`Table::append(&[Vec<Option<&str>>])`, [lance-graph crates/lance-graph-intake/src/lib.rs:1162](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-intake/src/lib.rs#L1162)). Column order comes from the registered binding. From there evaluation is lanes only. Hot-path rule: no serde `Value` or VRL `Value` traversal after decode.

Arrow is not proposed. Vector has an `ArrowStream` codec ([vector lib/codecs/src/encoding/serializer.rs:169](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/codecs/src/encoding/serializer.rs#L169)). Measure NDJSON decode cost first and consider Arrow only if it dominates.

Identity table (all proposals):

| item | rule | basis |
|---|---|---|
| tenant | from a verified bearer mapped to a `TenantBinding`; never from the payload | `TenantBinding` ([sentinel-rs crates/sentinel-telemetry/src/observation.rs:46-68](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-telemetry/src/observation.rs#L46-L68)); foreign-tenant refusal pattern in [sentinel-rs crates/sentinel-o2/src/lib.rs:368](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/lib.rs#L368); `propose()` exists at [sentinel-rs crates/sentinel-core/src/lib.rs:114](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-core/src/lib.rs#L114) (its scope parameter is named in the brief, not verified here) |
| source | route segment plus VRL-set field `lg.source`. `source_id` in `EventMetadata` is not serialized | [vector lib/vector-core/src/event/metadata.rs:80-113](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/vector-core/src/event/metadata.rs#L80-L113) |
| event id | sha256 over the canonical normalized line, computed in Vector remap before the fan-out so O2 and lance-graph see the same id | mirrors S-2 (dedup before spend) in this repo's `CLAUDE.md` |
| schema revision | `lg.schema_rev` checked against the BindStore class/signature revision; mismatch refused | `SignatureRevisionMismatch` ([lance-graph crates/lance-graph-dt64/src/lib.rs:581](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-dt64/src/lib.rs#L581)) |
| timestamps | three distinct values: event time, received time, and O2 `_timestamp` in microseconds | O2 search takes microsecond bounds ([openobserve src/config/src/meta/search.rs:229-242](https://github.com/openobserve/openobserve/blob/761446c5b1b45c5e20ad69c94ef6cd2df3962622/src/config/src/meta/search.rs#L229-L242)) |
| dedup | by event id before fold. Count and Sum are not idempotent | lane-fold law metadata (section 3) |
| replay | same route, provenance `lg.origin=replay`, idempotent through the dedup | |
| watermark | per tenant and source: max event time minus allowed lateness. Later events are recorded as late and counted, never silently dropped | no existing home (section 3: NOT FOUND) |

Ack and status mapping:

| receiver outcome | HTTP status | Vector behaviour |
|---|---|---|
| committed | 200 | batch delivered |
| refused (schema, tenant or shape) | 400 | not retried: the HTTP retry defaults cover 429, 408, 5xx except 501 ([vector src/sinks/util/http.rs:183-195](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sinks/util/http.rs#L183-L195)), so 400 is outside the set. The exact delivery status Vector records for the event is set through finalizers ([vector src/sinks/util/sink.rs:454-460](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sinks/util/sink.rs#L454-L460)); the precise value for 400 is not recorded in the inputs |
| transient | 503 or 429 | retried with Fibonacci backoff ([vector src/sinks/util/retries.rs:80](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sinks/util/retries.rs#L80)). Delivery is at-least-once; dedup makes it exactly-once in effect |

## 6. E: Incremental evaluation

Rules:

- No second runtime. "Incremental" means: recompute the affected (tenant, group, window) from resident lanes through the existing `GroupReduce`, and check the result equals a full rebuild. Same pattern as XLOG: [lance-graph crates/lance-graph-xlog/tests/wide_closure.rs:317](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-xlog/tests/wide_closure.rs#L317) (`refresh_equals_rebuild_after_every_step`).
- Corrections are retire plus add (`DeltaLog` ADD/RETIRE, [lance-graph crates/lance-graph-xlog/src/wide.rs:418](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-xlog/src/wide.rs#L418)).
- Summing gauges across time is refused. The plan's gate in `registered-source-profiles-v1.md` (D-RSP-3 Gate) reads "summing a snapshot across dates is refused".
- Histograms, distributions and sketches are unsupported in lance-graph. They stay in Vector and O2.
- Time windows and watermarks are a missing primitive. Smallest proposal: window id as a group-key lane (`GroupKey::Pair`, [lance-graph crates/lance-graph-mask-risc/src/ir.rs:521-538](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-mask-risc/src/ir.rs#L521-L538)).

| signal class | Vector representation | lance-graph fold today | merge law | duplicates | late data and corrections | status |
|---|---|---|---|---|---|---|
| discrete event (log) | `Event::Log` | `GroupFold::Count` | associative, commutative; Count not idempotent | dedup by event id before fold | retire plus add, recompute window | slice 1 |
| gauge | Absolute Gauge | `MinI32`, `MaxI32`; a "latest" fold was NOT FOUND | Min and Max idempotent | dedup still applied for consistent counts | recompute window; summing across time refused | slice 1 (min/max only) |
| delta counter (Incremental, including converted RATE) | Incremental Counter; RATE has value times interval ([vector src/sources/datadog_agent/metrics.rs:506-530](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/metrics.rs#L506-L530)) | `SumSymI32` | associative, commutative; Sum not idempotent, not invertible | dedup before fold is mandatory | retire plus add; `PmLedger` is the only signed accumulator found, in a lab | slice 1 |
| cumulative counter | Absolute Counter | no differencing fold found | not established | not established | not established | not in slice 1; Vector `aggregate` has a Diff mode ([vector src/transforms/aggregate/config.rs:210-225](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/transforms/aggregate/config.rs#L210-L225)), semantics beyond the name not verified |
| histogram, distribution, sketch, set | `MetricValue` Set, Distribution, AggregatedHistogram, AggregatedSummary, Sketch ([vector lib/vector-core/src/event/metric/value.rs:16-86](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/lib/vector-core/src/event/metric/value.rs#L16-L86)) | none | n/a | n/a | n/a | unsupported; stays in Vector and O2 |

## 7. F: OpenObserve interoperability

| flow | transport | owner | consistency | provenance | status |
|---|---|---|---|---|---|
| Vector to O2 | `http` sink to `/api/{org}/{stream}/_json`, or OTLP sink to `/{org}/v1/logs` | Vector config | at-least-once | `lg.origin=vector` | routes exist; no O2 sink in Vector |
| Vector to lance-graph | `http` sink NDJSON to the intake route | Vector config plus intake route (Q3) | at-least-once; dedup makes it exactly-once in effect | `lg.origin=vector` | route not built |
| O2 to lance-graph (replay) | `_search` / `_search_partition` with microsecond windows | replay tool reusing `O2Transport` | idempotent through event-id dedup | `lg.origin=replay` | `sentinel-o2` is read-only and has the transport trait; bounds today are 24 h and 1000 rows |
| lance-graph to O2 (findings) | HTTP write to a separate stream | not decided | at-least-once | `lg.origin=lance-graph` | no O2 write path exists |

Contracts:

- Consistency. Both sinks are at-least-once and independent. Reconcile per closed window: compare `COUNT(*)` and `MIN/MAX(_timestamp)` on both sides, then diff the two event-id sets when the counts disagree, reusing the `sentinel-o2` digest collect, readback and reconcile shape ([sentinel-rs crates/sentinel-o2/src/digest.rs:436](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/digest.rs#L436), [sentinel-rs crates/sentinel-o2/src/digest.rs:514](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/digest.rs#L514), [sentinel-rs crates/sentinel-o2/src/digest.rs:542](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/digest.rs#L542)). Settle time is 300 s in that code ([sentinel-rs crates/sentinel-o2/src/digest.rs:57](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/digest.rs#L57)).
- Provenance fields: `lg.origin` in {vector, replay, lance-graph}, `lg.event_id`, `lg.schema_rev`, `lg.derived_from`.
- Loop prevention. Findings go to a separate stream. lance-graph intake refuses any event with `lg.origin=lance-graph`. Vector and O2 pipelines route on the same field. None of this exists today.
- Replay. Use `_search_partition` and `_search` with microsecond windows, reusing `O2Transport` ([sentinel-rs crates/sentinel-o2/src/lib.rs:263](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/lib.rs#L263)) and the `plan()` bounds ([sentinel-rs crates/sentinel-o2/src/lib.rs:97](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/lib.rs#L97), [sentinel-rs crates/sentinel-o2/src/lib.rs:169](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-o2/src/lib.rs#L169)). Those bounds (24 h window, 1000 rows) are today's code, not a recommendation.

Not assumed. This plan does not claim that all Datadog pipelines are supported. Sketches, APM stats, LLMObs and traces are outside slice 1. Not all Vector signal conversions are supported either: RATE is converted, UNSPECIFIED is dropped, and a v3 payload containing a sketch is rejected.

## 8. G: First vertical slice

```text
Datadog Agent (vector.metrics.url / vector.logs.url)
        |
        v
Vector datadog_agent source
        |
        v
remap: normalize small contract + lg.event_id
        |
        +--> http sink --> O2  /api/{org}/{stream}/_json
        |
        +--> http sink --> lance-graph intake (NDJSON, acks on)
```

Event contract (tenant is not a field; it comes from auth):

| field | meaning |
|---|---|
| `lg.source` | source name, matches the route segment |
| `lg.event_id` | sha256 of the canonical normalized line |
| `lg.schema_rev` | revision checked against the binding |
| `ts` | event time |
| `host` | host name |
| `service` | service name |
| `kind` | one of `log`, `count`, `gauge` |
| `name` | metric or log name |
| `value` | i32, scaled per CodeSpace and exponent |
| `status` | log status |

Live aggregate: count of logs, and sum of a delta counter, per host per 60 s window via `GroupReduce`. Served by a read-only GET API following the GET-only axum pattern in `sentinel-web` ([sentinel-rs crates/sentinel-web/src/lib.rs:70-82](https://github.com/AdaWorldAPI/sentinel-rs/blob/24807d29619023d3ce5b623413401376988719e9/crates/sentinel-web/src/lib.rs#L70-L82)). Consumable from PowerShell and C#:

```text
Invoke-RestMethod -Uri "http://localhost:8080/api/v1/aggregate?window=60s"
await new HttpClient().GetStringAsync("http://localhost:8080/api/v1/aggregate?window=60s");
```

The URL and parameter names are illustrative; no such route exists.

References for correctness:

- O2 `_search` SQL over the same window: `SELECT host, count(*) ... GROUP BY host`.
- DuckDB over the captured NDJSON, following Quack's DuckDB oracle pattern ([lance-graph crates/lance-graph-quack/tests/duckdb_differential.rs](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-quack/tests/duckdb_differential.rs)).
- Upstream VRL differential for the remap (S1).

Measurement table (to be filled when S6 runs):

| measure | how |
|---|---|
| throughput | events per second at the receiver |
| CPU time | process CPU time per million events |
| RSS | peak resident set |
| end-to-end latency | agent timestamp to aggregate visible |
| duplicate behaviour | replay the same batch; counts must not change |
| replay correctness | wipe, replay from O2; result equals the live aggregate |

Second fixture: `ogar-flow-m365` and `ogar-az` observations (for example `accountEnabled`, license assignment) joined to infra hosts through intake `left_join` ([lance-graph crates/lance-graph-intake/src/lib.rs:1038](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-intake/src/lib.rs#L1038)) into a cross-source report. It must pass D-RSP-5's nine row-type tests ([lance-graph .claude/plans/registered-source-profiles-v1.md:144-153](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/.claude/plans/registered-source-profiles-v1.md#L144-L153)). Note the plan's rule: an absent `accountEnabled` never establishes a negative fact. D-RSP-6 plans to harvest field IRIs from `ogar-az` `AttrDef` and `SchemaId{MsGraph,2}`.

## 9. H: Constraints check

| constraint | how this plan satisfies it |
|---|---|
| Reuse first | Vector, O2, intake, mask-risc, `sentinel-o2` transport and digest shape, `sentinel-web` GET pattern are reused; new work is limited to the gaps in section 11 |
| No new Datadog receiver | Vector's `datadog_agent` source is used ([vector src/sources/datadog_agent/mod.rs:87](https://github.com/vectordotdev/vector/blob/c879bd677429193410389be29207da6c44c5aa21/src/sources/datadog_agent/mod.rs#L87)) |
| No new collector framework | Vector is deployed as a binary |
| No new telemetry storage backend | O2 holds retention; lance-graph holds resident lanes only |
| No full VRL interpreter port | Upstream `vrl` is used for parsing and admission; lance-graph lowers a small subset ([lance-graph crates/lance-graph-vrl/src/lib.rs:60-80](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-vrl/src/lib.rs#L60-L80)) |
| No duplicated field-binding authority | D-RSP-5: one seam, owned by the source owner (intake) |
| No generic event materialization in the native hot path | decode once at the boundary; lanes only afterwards; `materialize_rows` is positional ([lance-graph crates/lance-graph-vrl/src/lib.rs:941](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-vrl/src/lib.rs#L941)) |
| Ownership split | Vector: transport and normalization. O2: retention and reference SQL. lance-graph: typed evaluation. This repo: Dockerfile and settings only |

## 10. Compatibility matrix

| signal | Agent to Vector | Vector to O2 | Vector to lance-graph (slice 1) | notes |
|---|---|---|---|---|
| logs | redirect keys and `/api/v2/logs` served | `http` sink to `_json` (route exists) | yes, `kind=log` | Agent side: [datadog-agent comp/logs/agent/config/config_keys.go:323-369](https://github.com/DataDog/datadog-agent/blob/d92d1f35a782d230bb2fc9ddbb0ef03c71893cca/comp/logs/agent/config/config_keys.go#L323-L369) |
| COUNT series | forwarder registers vector for v1/v2/v3 | route exists; metric encoding to O2 not verified | yes, as Incremental Counter | |
| GAUGE series | same | same | yes, as Absolute Gauge | |
| RATE series | same | same | converted to Incremental Counter (value times interval) | rate semantics not preserved |
| sketches | agent registers vector sketches; Vector serves `/api/beta/sketches` | not verified | no | |
| v3 series | agent `vectorSeriesUseV3`; Vector serves v3 | not verified | not in slice 1 | v3 payload with a sketch is rejected |
| traces | UNVERIFIED on agent side; Vector serves only `/api/v0.2/traces` | OTLP `/v1/traces` route exists; Vector trace to OTLP not verified | no | v0.4 and v0.5 not served |
| APM stats | agent writer exists; redirect not verified | not verified | no | Vector serves it only with `multiple_outputs` |
| LLMObs | not recorded on agent side; Vector serves `/api/v2/llmobs` | not verified | no | |
| OTLP logs and metrics | not applicable (OTLP source) | OTLP routes `/v1/logs`, `/v1/metrics` exist | no | Vector OTLP source: HTTP protobuf plus gRPC 4317 |

## 11. Missing primitives

1. Upstream `vrl` dependency plus a differential harness. Owner: lance-graph, in the `vrl-recipe-probe` lab (own workspace, so `vrl` stays out of customer binaries).
2. Typed `ExternalEnv` derived from `ClassView` and D-RSP-5 validity. Owner: lance-graph, `lance-graph-vrl`, behind the D-RSP-5 seam owned by intake.
3. Parser swap to `vrl::parser::ast`. Owner: `lance-graph-vrl`.
4. Lowering for `&&`, `||`, `!`, `exists()`. Owner: `lance-graph-vrl`. No new kernel: mask-risc already has `And`, `Or`, `Xor`, `AndNot` and `Not` ([lance-graph crates/lance-graph-mask-risc/src/ir.rs:231-239](https://github.com/AdaWorldAPI/lance-graph/blob/f5e625f131ff0f39de3b3207cf7ae7cb92c23f04/crates/lance-graph-mask-risc/src/ir.rs#L231-L239)), and `exists()` is a read of the presence plane the witness already borrows. Only the three-valued rule for `!` over UNKNOWN needs deciding (D-RSP-5 test 9: `NOT accountEnabled` never selects an absent value).
5. NDJSON batch intake route. Owner decision Q3: `lance-graph-intake` or a deployment crate. `paperless-rs` and `sentinel-rs` own only Dockerfile and settings.
6. Event-id dedup set. Owner: intake, keyed like S-2.
7. Time window and watermark. Owner: lance-graph; no existing home. Smallest form: window id as a group-key lane.
8. Per-window recompute equal to rebuild. Owner: mask-risc `GroupReduce` with the XLOG refresh-equals-rebuild test pattern.
9. Read-only aggregate API. Owner: reuse the `sentinel-web` axum GET pattern.
10. O2 write path and origin marker. Owner: `sentinel-o2` is read-only today.
11. Optional upstream issue: wrong operation name ("remainder") in the VRL error for `>` over null. Owner: vectordotdev/vrl.
12. Pin `sentinel-rs` lance-graph dependencies to a rev. Owner: sentinel-rs.

## 12. Implementation slices

### S1: Upstream-VRL differential harness

- Goal: a harness in `vrl-recipe-probe` (lab, own workspace) pinning vrl rev 7738f74 (Vector's lock) and a second job on 0.31.0. Run every case of `tests/recipe.rs` and `tests/presence.rs` through upstream with a typed `ExternalEnv`. Assert KEEP/DROP/UNKNOWN corresponds to ok/abort/error. Absent `==`/`!=` cases are recorded as pinned divergences, not passes.
- Owner: lance-graph.
- Acceptance: both pins run in CI; the divergence list is committed and checked.
- Ablation: swap Gt for Ge in the lowering, which must turn red. Drop the `ExternalEnv` typing, so every order compare is refused with E100, which must turn red.
- Not in scope: any change to `lance-graph-vrl`.

### S2: Upstream admission gate

- Goal: in `lance-graph-vrl`, feature-gated: a program is admitted only if upstream typed compile succeeds and the lowering succeeds. `Refusal` carries the upstream E-code.
- Owner: lance-graph.
- Acceptance: `if .cpu > 3 { abort }` over an optional field is refused with E100; over a non-optional integer it is admitted.
- Ablation: disable the gate, so the optional-field program is admitted, which must turn red.
- Not in scope: parser swap.

### S3: Parser swap

- Goal: lower from `vrl::parser::ast`; delete the own parser; same `Stmt` output for the corpus.
- Owner: lance-graph.
- Acceptance: corpus produces identical `Stmt` lists before and after.
- Ablation: mutate one AST mapping, which must turn the corpus diff red.
- Not in scope: new constructs.

### S4: NDJSON intake route, event-id dedup, schema_rev, ack mapping

- Goal: the receiver of section 5.
- Owner: per Q3.
- Acceptance: a real Vector `http` sink config; send a batch twice and counts stay unchanged; kill the receiver mid-batch, Vector retries, exactly-once effect; wrong `schema_rev` returns 400 and nothing is appended.
- Ablation: disable dedup; count doubles, which must turn red.
- Not in scope: windows, aggregates.

### S5: Windowed GroupReduce aggregate and read-only GET API

- Goal: per-host, per-60 s window aggregate with DuckDB and O2 SQL references.
- Owner: lance-graph.
- Acceptance: aggregate equals both references on closed windows.
- Ablation: drop the window lane from the group key, mismatch red. Corrupt one value, mismatch red.
- Not in scope: late-data policy beyond counting.

### S6: Real Datadog Agent vertical slice with measurements

- Goal: container `datadog/agent` with `vector.*` keys; Vector `datadog_agent` source; two `http` sinks; the measurement table of section 8.
- Owner: deployment.
- Acceptance: aggregate equals O2 SQL for closed windows; replay-from-O2 reproduces it.
- Ablation: turn off acknowledgements end to end and kill Vector mid-batch; the loss must be detected by reconcile.
- Not in scope: traces, APM stats, LLMObs, sketches.

### S7: Findings to O2 with loop prevention

- Goal: write findings to a separate stream with `lg.origin=lance-graph`; intake refuses that origin.
- Owner: `sentinel-o2` write path (new).
- Acceptance: a finding never re-enters the count.
- Ablation: remove the intake origin check; the finding re-enters and the count drifts, which must turn red.
- Not in scope: alert routing.

### S8: M365/Graph fixture and cross-source join

- Goal: `ogar-flow-m365` and `ogar-az` observations joined to hosts; D-RSP-5's nine row-type tests on both frontends.
- Owner: lance-graph with OGAR.
- Acceptance: nine tests green on Quack and `lance-graph-vrl`.
- Ablation: treat absent `accountEnabled` as false; the row-type test must turn red.
- Not in scope: live Graph calls (`ogar-az` has no HTTP).

Sequencing. S1 is the next step. It is the only slice that should start before the operator answers Q1 to Q3. Do not begin S4 or later until S1 and S2 establish which constructs lower.

## 13. Open decisions

| id | question | recommendation |
|---|---|---|
| Q1 | VRL-compat for `==`/`!=` over optional fields: refuse, or adopt VRL null semantics in a separate `VrlCompat` mode | refuse by default (preserves D-RSP-5); admit only `exists(.f) && .f != N` once `&&` lands |
| Q2 | Which VRL version is normative: Vector's 7738f74 or OpenObserve's 0.31.0 | Vector's; test both; report drift |
| Q3 | Where the intake adapter crate lives: an extension of `lance-graph-intake`, or a new crate in lance-graph (`paperless-rs` owns only Dockerfile and settings) | operator decision; the census does not favour one, but the receiver must sit behind the D-RSP-5 seam owned by intake |
| Q4 | Datadog trace redirection is unverified on the agent side | keep traces out of slice 1; verify with a real Agent in S6 before claiming support |

## Appendix A: the probe program

Summary. The scratch probe built against `vrl` at 2d5987e (0.39.0) with features `compiler` and `stdlib`. For each program it called `compile` (untyped) or `compile_with_external` with `ExternalEnv::new_with_kind` (typed), then ran `Runtime::resolve` on six events and recorded the outcome per event: kept, aborted, or runtime error text. It is not a crate in any repository; its full source follows, so the run can be repeated with `cargo run --release` (untyped block with `-- all`). The run used a local checkout of the same commit through a path dependency; the manifest below pins the identical commit by git rev instead. S1 replaces it with a committed harness.

`Cargo.toml`:

```toml
[package]
name = "vrl-diff"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]

[dependencies]
vrl = { git = "https://github.com/vectordotdev/vrl", rev = "2d5987eadc3301e58eb107ea3d37ef3ad98b6fb7", default-features = false, features = ["compiler", "stdlib"] }
serde_json = "1"
```

`src/typed.rs`:

```rust
use vrl::compiler::{compile_with_external, runtime::Runtime, state::ExternalEnv, CompileConfig, TargetValue, TimeZone};
use vrl::value::{Kind, Secrets, Value, kind::{Collection, Field}};
use std::collections::BTreeMap;

pub fn schema(cpu: Kind) -> ExternalEnv {
    let mut m: BTreeMap<Field, Kind> = BTreeMap::new();
    m.insert("cpu".into(), cpu);
    m.insert("host".into(), Kind::bytes());
    ExternalEnv::new_with_kind(Kind::object(Collection::from(m)), Kind::object(Collection::empty()))
}

pub fn run(src: &str, ev: &str, ext: &ExternalEnv) -> String {
    let fns = vrl::stdlib::all();
    let res = match compile_with_external(src, &fns, ext, CompileConfig::default()) {
        Ok(r) => r,
        Err(d) => return format!("COMPILE-ERR {}", d.iter().map(|x| format!("E{}", x.code)).collect::<Vec<_>>().join(",")),
    };
    let mut t = TargetValue { value: serde_json::from_str::<Value>(ev).unwrap(), metadata: Value::Object(Default::default()), secrets: Secrets::default() };
    match Runtime::default().resolve(&mut t, &res.program, &TimeZone::default()) {
        Ok(_) => format!("KEEP {}", serde_json::to_string(&t.value).unwrap()),
        Err(e) => format!("{}", e),
    }
}
```

`src/main.rs`:

```rust
mod typed;
use vrl::compiler::{compile, runtime::Runtime, TargetValue, TimeZone};
use vrl::value::{Secrets, Value};


fn run(src: &str, ev: &str) -> String {
    let fns = vrl::stdlib::all();
    let res = match compile(src, &fns) {
        Ok(r) => r,
        Err(d) => {
            let codes: Vec<String> = d.iter().map(|x| format!("E{}", x.code)).collect();
            return format!("COMPILE-ERR {}", codes.join(","));
        }
    };
    let info = res.program.info();
        let mut t = TargetValue { value: serde_json::from_str::<Value>(ev).unwrap(), metadata: Value::Object(Default::default()), secrets: Secrets::default() };
    let mut rt = Runtime::default();
    let tz = TimeZone::default();
    let out = rt.resolve(&mut t, &res.program, &tz);
        let fl = format!("[fallible={} abortable={}]", info.fallible, info.abortable);
    match out {
        Ok(_) => format!("OK {} {}", serde_json::to_string(&t.value).unwrap(), fl),
        Err(e) => format!("RT {:?} {}", e.to_string(), fl),
    }
}

fn main() {
    use vrl::value::Kind;
    for (name, k) in [("cpu: integer", Kind::integer()), ("cpu: integer|null", Kind::integer().or_null()), ("cpu: integer|undefined", Kind::integer().or_undefined())] {
        let ext = typed::schema(k);
        for p in ["if .cpu > 3 { abort }", "if .cpu == 3 { abort }", "if .cpu != 3 { abort }", "if .cpu > 3 && .host == \"a\" { abort }"] {
            println!("### [{name}] {p}");
            for e in [r#"{"host":"a","cpu":5}"#, r#"{"host":"b","cpu":1}"#, r#"{"host":"c","cpu":null}"#, r#"{"host":"d"}"#, r#"{"host":"e","cpu":"5"}"#] { println!("  {e} => {}", typed::run(p, e, &ext)); }
        }
    }
    if std::env::args().nth(1).as_deref() != Some("all") { return; }
    let progs = [
        ".out = .host",
        "if .cpu > 3 { abort }",
        "if int!(.cpu) > 3 { abort }",
        "if (.cpu ?? 0) > 3 { abort }",
        "if is_integer(.cpu) && .cpu > 3 { abort }",
        "if .cpu == 3 { abort }",
        "if .cpu != 3 { abort }",
        "if .host == \"a\" { abort }",
        "if .cpu > 3 && .mem > 1 { abort }",
        "del(.host)",
        ".k = .host; .host = .svc; .z = .k",
        "if exists(.cpu) { abort }",
        "if is_null(.cpu) { abort }",
    ];
    let evs = [r#"{"host":"a","svc":"s","cpu":5,"mem":2}"#, r#"{"host":"b","cpu":1}"#, r#"{"host":"c","cpu":null}"#, r#"{"host":"d"}"#, r#"{"host":"e","cpu":"5"}"#, r#"{"host":"f","cpu":3.5}"#];
    for p in progs { println!("### {p}"); for e in evs { println!("  {e} => {}", run(p, e)); } }
}
```

Exact result lines (from the census inputs):

Untyped target (compile, no schema):

- `.out = .host` OK infallible.
- `if .cpu > 3 { abort }` -> COMPILE ERROR E100 (unhandled fallible) for all. Same for `if .cpu > 3 && .mem > 1 {abort}` and `if is_integer(.cpu) && .cpu > 3 {abort}` (type narrowing does NOT flow through &&).
- `if (.cpu ?? 0) > 3 {abort}` -> E651 (unnecessary coalesce — ?? catches errors, not null).
- `if int!(.cpu) > 3 {abort}`: a aborted, b kept, c/d/e/f runtime error "expected integer, got null|string|float".
- `if .cpu == 3 {abort}`: infallible; all kept (null/missing/"5"/3.5 compare false, no type error).
- `if .cpu != 3 {abort}`: all aborted INCLUDING null and missing.
- `if .host == "a" {abort}`: a aborted; others kept.
- `del(.host)` OK. Rename chain `.k = .host; .host = .svc; .z = .k`: in-place mutation honoured (z = old host); assigning missing .svc writes `host: null` (missing materialises as explicit null on assignment).
- `exists(.cpu)` true for null, false only for missing; `is_null(.cpu)` true for both null and missing.

Typed target via `compile_with_external` + `ExternalEnv::new_with_kind(object{cpu: K, host: bytes})`:

- K = integer (non-nullable): `if .cpu > 3 {abort}` COMPILES; a aborted, b kept, c(null)/d(missing) RUNTIME ERROR ("can't calculate remainder of type null and integer" — note: the error text names the wrong op, an upstream message bug worth reporting), e("5") runtime error "expected string, got integer". `&&` combination also compiles.
- K = integer|null or integer|undefined: `.cpu > 3` -> E100 again.
- `==`/`!=` unchanged by typing: missing/null never error; `!=` aborts them.
