# Usability — what a person can actually do when the machine is wrong

> **Scope.** This is about the product surface: the archive a person uses.
> Not the IR, not the recognizer. `ogar-doc-ir` is a perceptual IR and takes
> no UX; `tesseract-rs` is faithful recognition and takes no storage. What
> the person sees, understands, and can act on is *this* layer's problem,
> which is why it is written here.
>
> **The render path is `AdaWorldAPI/a2ui-rs`** — the OGAR render target
> (charter: OGAR `docs/A2UI-SCREEN-ADDRESSING-PROPOSAL.md`, #204/#205).
> *"Don't push pixels — address the screen."* Down the wire go `NodeDelta`
> frames (16-byte GUID key, wide mask delta, ClassView-carved LE values), up
> go `ActionInvoke` frames; the client holds the ClassView/template codebook
> and renders locally from borrowed memory
> (`a2ui-rs/crates/a2ui-core/src/lib.rs:1-24`). Its three iron boundaries
> bind everything in §5: **T1** widget skins are ClassView templates, never a
> second vocabulary; **T2** the surface never carries behavior — actions
> travel by ADDRESS (`ActionDef` ordinal on the Core node); **T3** no
> serialization in the hot path.
>
> So every gap below is *"which field does the ClassView project, and which
> `ActionDef` does the person fire"* — **not** "which Askama template gets
> another conditional." The current hand-rolled templates in
> `tesseract-paperless-web` are the stand-in, not the target.
>
> Worth naming, because it is not a coincidence: a2ui-rs's own killer probe
> **P-REHOST** re-renders *a harvested MedCare screen* from its
> `CompiledClass` × ClassView × askama and fires one harvested `ActionDef`
> (`a2ui-rs/README.md`). The system this document treats as the usability
> teacher is the same system a2ui-rs is built to re-host.
>
> **Sources.** `AdaWorldAPI/MedCare-rs` (a real clinical app with real users
> — the teacher), `papermerge/papermerge-core` (a mature DMS), and this
> stack's own measured failures. Every claim carries a citation or says it
> is an inference.
>
> **Honesty about method.** No user research was done. These are inferences
> from two working systems plus failures this workspace has already
> measured on real pages. Treat them as reasoned, not validated.

## §0 The premise

A person opening a document archive is not reading the archive. They are
trying to do one thing: find a value, check a figure, file an invoice, see
whether a result is normal. The archive is in the way of that, and its job
is to be in the way as little as possible.

The machine is sometimes wrong. That is not the usability problem — it is
the *precondition*. **The usability problem is a wrong answer that nothing
in the interface lets a person notice, check, or fix.** Every finding below
is a variation on that one sentence.

The failure this stack has measured most often is not garbled text. It is
**confident and wrong**: `mean_conf 99.47` on a page measuring `CER 0.6154`;
`mean_conf 91.47` on cells reading `$142` where the page says `14.2` and
`O09` where it says `0.9` (both in `tesseract-rs/CLAUDE.md`, § table
extraction). Text that *looks* fine, a number that *says* fine, and a value
that is wrong. A person cannot catch that by reading more carefully. They
can only catch it if the interface helps.

## §1 The five questions a real user asks

1. **Can I find it?** — 400 pages, one number.
2. **Can I see why it says that?** — the value against the ink it came from.
3. **What should I look at?** — attention routed to what is probably wrong.
4. **Can I fix it?** — when they spot the error, is there anything to do.
5. **Does it admit what it doesn't know?** — silence vs. a stated gap.

Question 1 is the one every document system solves first, and the only one
this archive currently answers well. Questions 2-5 are where usability
actually lives.

## §2 What MedCare-rs already answers, and why it is the teacher

MedCare-rs is not a document system, and that is exactly why it is
instructive: it is a real app whose users are clinicians with no patience
for an interface that makes them do the machine's work.

**A number becomes a status.** `LabParameter`
(`crates/medcare-core/src/labor.rs:5-16`) carries `ref_low`/`ref_high` **and**
`ref_low_female`/`ref_high_female`/`ref_low_male`/`ref_high_male`;
`LabValue::classify` (`:80-86`) turns a value plus its range into
`LabFlag::{Normal, Low, High, Critical}` (`:19-25`).

The transferable idea is not the enum. It is this: **a clinician does not
read "Hb 13.2". They read "normal."** The number is identical either way;
what usability adds is the *reading*. And the correct reading is
sex-stratified — it depends on a fact the number alone does not carry, so a
system that shows only the number has silently delegated the lookup to the
person. (Honest note: `classify` as written returns `Normal`/`Low`/`High`;
`Critical` is in the enum but not produced by this function.)

**Refusing to guess is a feature, and they say so.** `ground_loinc`
(`crates/medcare-cohorts/src/lab_ingest.rs:101-106`) resolves an OCR'd label
to a LOINC id and returns `None` when it cannot. The crate's own words, in
the doc comment immediately below it: *"an honest ungrounded label for
review"* — and in its test, *"review, not fabrication"*
(`:169-181`, asserting `ground_loinc("Vitamin-D-25-OH").is_none()`).

That is the whole doctrine in one function. Faced with a label it does not
recognise, the system does not pick the nearest plausible match. It says
*I don't know this one, a person should look* — and that verdict is
structured data a UI can route on, not a log line.

**The pattern underneath both:** MedCare-rs never asks the user to be the
interpreter. It interprets, and where it cannot, it says so in a form the
interface can act on.

## §3 What this stack already knows — and does not surface

The uncomfortable part. These are not gaps in knowledge; they are findings
already written down here, not reaching the screen.

**Confidence is aggregated by MIN at cell level and by MEAN at page level.**
`ogar-doc-ir`'s `TableCell::confidence` doc comment spells out the reason:
aggregate as *"the **minimum** over the cell's words, not the mean: one
misread digit invalidates a whole value, and a mean dilutes exactly the
signal the gate exists to catch (a cell of 95 and 40 reports 40, not 67)."*

The archive then reads `pages[0].quality.mean_conf` —
a **mean** — as the document's quality signal
(`tesseract-paperless-web/src/ingest.rs:216, 330`), and that mean is what
both templates print. The codebase knows means dilute, at one layer, and
shows a mean at the layer a person reads.

**`low_confidence` exists and does almost nothing.** It survives ingestion
as a bool and renders as an orange *"(low)"* next to the number
(`document.html:69`, `documents.html:82`). It sorts nothing, filters
nothing, routes nothing.

**Loss is counted and never shown.** `Document.drop_caps` was added
specifically to *"make the loss LOUD"* — a page that dropped an initial now
says so — and the archive UI never reads it. Same for the "fails loudly
(empty set, not wrong data)" principle already stated for borderless
tables: the honesty is in the data model, not in front of anyone.

**Per-word confidence exists.** `doc.v1` carries `conf` per word with a
bbox. Nothing in the UI uses either.

So the pattern across all four: **the system's self-knowledge is real and
structured, and stops at the template.**

## §4 The current surface, measured

Read from the templates and routes as they stand:

| moment | what exists | what a person can do |
|---|---|---|
| find | BM25 search, `<b>`-highlighted snippets (`documents.html:74-78`) | **works** — the one genuinely usable feature |
| verify | recognized text in a `<pre>` block (`document.html:87`) | nothing — **the page image is never shown** |
| attend | `confidence 99.47`, orange "(low)" | judge a raw number themselves |
| fix | — | nothing |
| trust | harvested fields, regions, extracted facts, all rendered flat | assume all of it |
| act | delete (`document.html:115-117`) | delete |

**The single largest gap is verification.** There is no image anywhere in
the archive UI. A person who suspects `$142` should be `14.2` has no way to
check without leaving the system and finding the original. Both reference
systems show the document; this one stores the raw bytes (content-addressed,
by design) and never renders them.

Everything else compounds from that: attention routing is worth little if
the thing you route someone *to* cannot be checked when they get there.

**And the one action that exists is shaped wrong.** Delete is a bespoke
`POST /documents/:hash/delete` with a form and a JS `confirm()` — the
surface carrying behavior, which is exactly what **T2** forbids. Under the
render path this is an `ActionInvoke` naming an `ActionDef` ordinal on the
document node. That matters beyond tidiness: an action that travels by
address is one the archive can authorize, log, and *offer conditionally*
(only where a role permits it) without the template knowing anything about
it — which is the mechanism every "route this to review" item below needs.

## §5 The gaps, ranked by usability bought

Ranked by what each buys a person, not by implementation cost. Each names
what already exists to build it from — in every case the data is present
and unsurfaced, which is why these are projection decisions rather than
pipeline work — and what it is in render-path terms (a projected **field**,
or an **action** by address).

**1. Show the page image beside the text.** *Field.* Makes every other
signal actionable. The raw bytes are already stored and content-addressed;
the document node needs a field the ClassView can project as the source
surface, and the word bboxes `doc.v1` already carries are what let a
highlight land on the ink rather than beside it. Without this, nothing
below can be confirmed by the person it is shown to.

**2. Turn confidence into a status, and route on it.** *Field.* The
MedCare-rs transplant: stop printing `99.47`. A document is *clean* /
*check this* / *probably wrong* — and per **T1** that is a ClassView
projection of a field, not a template conditional, which is precisely what
makes it sortable and filterable in the list without every surface
re-deriving the threshold. Two things must change together or it is
cosmetic: the aggregate must stop being a mean (this stack's own cell-level
rule says min; page level should follow its own reasoning), and the status
must be actionable in the list, not decorative on the detail page.

**3. Highlight uncertain words in place.** *Field, at word granularity.*
Per-word `conf` + bbox are in `doc.v1` already, and a wide mask over
per-word positions is the frame shape `NodeDelta` already carries
(`mask_words` + `mask_positions`). Shading the words the recognizer was
least sure about turns "check this document" into "check *this word*" — the
difference between a task and a chore. This is also the honest use of a
confidence number: as a **pointer**, never as a score a person must
interpret.

**4. Show what was dropped.** *Field.* `drop_caps > 0`, empty regions, a
table that classified but yielded no grid. All already computed. A person
who knows something is missing looks; a person shown a clean page does not.
Cheapest item on the list and pure honesty — no new measurement, no new
model, just projecting a count that already exists.

**5. A correction path.** *Action.* The end of the chain: someone spots the
error and can fix it. Under **T2** this is an `ActionDef` on the document
node fired by `ActionInvoke` — the same shape delete should already have —
which is what lets a correction be authorized and recorded rather than
being a bespoke mutation route per fixable thing. Deliberately last:
without 1-4 nobody notices there is anything to correct, and a correction
surface over an unverifiable page is a way to introduce errors, not remove
them.

## §6 What this does not settle

- **No user research.** Everything above is inference from two working
  systems and this workspace's own measured failures.
- **MedCare-rs's answers are domain-shaped.** LOINC grounding and
  sex-stratified reference ranges are clinical; the *principle* (interpret,
  or say you can't, in a form the UI can route on) is what transfers. Do
  not copy `LabFlag` into a generic archive and call it done.
- **What "clean / check / wrong" means in numbers** is unmeasured. The
  thresholds are the kind of thing this workspace insists must be measured
  and pinned rather than asserted, and no fixture here supports them yet.
- **a2ui-rs is seed-stage, and this document does not pretend otherwise.**
  `a2ui-core` is currently a re-export of the upstream frames plus one
  drift-fuse test; the server (W2) and the wasm fieldview client (W3) are
  marked *planned* in its own README. Naming it as the render path is a
  statement about **where these gaps should land**, not a claim that they
  can be built against it today. What is actionable now is not building on
  it — it is *not* accreting more bespoke templates and mutation routes in
  the meantime, since every one of those is work that must later be undone
  to satisfy T1/T2.
- **One IR-shaped observation, recorded and NOT filed:** `DocIr::Region`
  carries no confidence, where `TableCell` and `TypedField` both do — so a
  renderer wanting to shade an uncertain *line* has nothing to read. That is
  the only thing in this document that could ever be a render primitive
  rather than a UX concern, and it would need its own evidence before it
  went anywhere near the IR.
