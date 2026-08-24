//! Intake: raw bytes → the S-2 gate → a retina → one `DocIr`.
//!
//! ```text
//!   HASH BEFORE YOU SPEND.  TWO RETINAS, ONE SHAPE.
//! ```
//!
//! # The two retinas, and why neither is written here
//!
//! Both already exist and both already emit [`ogar_doc_ir::DocIr`]:
//!
//! | retina | producer | this crate calls |
//! |---|---|---|
//! | DOM (a web page) | `spider_doc_ir` (`AdaWorldAPI/spider`) | `ingest_html` (feature `dom`) |
//! | pixel (a scan) | tesseract-rs → `doc.v1` → `ogar-from-docv1` | [`ingest_doc_v1`] |
//!
//! ⚠ The DOM leg is behind the OFF-by-default `dom` feature because
//! `spider_doc_ir` does not currently compile against `ogar-doc-ir` main — it
//! omits `TableCell::confidence`, a field added upstream after spider's only
//! commit (2026-07-14). One line upstream fixes it; the manifest carries the
//! detail. Nothing here works around it, because working around a broken
//! producer is how two IRs start diverging.
//!
//! So intake's job is not to build a producer. It is to run the **gate** in
//! front of them, hand back one shape, and be honest about which retina
//! produced it — which [`DocIr::source`] already records
//! ([`ogar_doc_ir::Provenance`]), so nothing here has to track it separately.
//!
//! # The pixel retina has two legs, and the split is deliberate
//!
//! [`ingest_doc_v1`] takes `doc.v1` JSON from ANY producer and needs no OCR
//! dependency at all. [`ingest_image`] runs the recognizer in-process and is
//! behind the **off-by-default `ocr` feature**, because pulling the recognizer
//! (and `ndarray` beneath it) into a consumer that only ingests web pages is a
//! real cost for nothing.
//!
//! # The gate runs FIRST, and that is the whole point of S-2
//!
//! `OGAR-DOC-INGESTION-SPINE` S-2: dedup must precede recognition SPEND. Every
//! entry point here hashes and asks the index *before* touching a retina —
//! [`ingest_image`] especially, where the retina is the expensive one. A
//! duplicate costs a hash and a lookup. The gate matches BOTH the original and
//! derived-artifact hashes (S-2's second half), which [`paperless_kv::preflight`]
//! already implements.
//!
//! # What `content_sha256` means, stated because the IR's own docs correct
//! their plan on it
//!
//! It is the hash of the ORIGINAL bytes — a **per-acquisition dedup key**, not
//! a cross-retina identity. The same invoice as a scan and as HTML has
//! different bytes and therefore different hashes; cross-retina convergence is
//! a facts question (`ogar_doc_ir::converges_on_facts`). Per-acquisition is
//! exactly what a dedup gate wants, so the two agree here rather than merely
//! coexisting: `spider_doc_ir::harvest` hashes `content.as_bytes()`, which is
//! the same hash [`paperless_kv::preflight`] computes over the same bytes —
//! asserted, not assumed (see `dom_identity_matches_the_gate`).

#![forbid(unsafe_code)]

use ogar_doc_ir::DocIr;
use paperless_kv::{ContentSha256, DedupIndex, MatchedOn, Verdict};

/// What one intake attempt produced.
#[derive(Debug, Clone)]
pub enum Ingested {
    /// The gate matched. No retina ran.
    Duplicate {
        /// The hash that matched.
        hash: ContentSha256,
        /// Which stored hash it matched — original bytes, or a derived
        /// artifact (S-2's second half).
        matched: MatchedOn,
    },
    /// Novel bytes; a retina ran and produced one shape.
    Novel {
        /// The per-acquisition dedup key.
        hash: ContentSha256,
        /// The perceptual IR. [`DocIr::source`] says which retina.
        ir: Box<DocIr>,
    },
}

impl Ingested {
    /// The IR, if a retina ran.
    #[must_use]
    pub fn ir(&self) -> Option<&DocIr> {
        match self {
            Self::Novel { ir, .. } => Some(ir),
            Self::Duplicate { .. } => None,
        }
    }

    /// The hash either way — a duplicate still has an identity.
    #[must_use]
    pub const fn hash(&self) -> &ContentSha256 {
        match self {
            Self::Novel { hash, .. } | Self::Duplicate { hash, .. } => hash,
        }
    }
}

/// Why an intake attempt failed.
#[derive(Debug)]
pub enum IntakeError {
    /// The DOM retina needs UTF-8; these bytes are not.
    #[cfg(feature = "dom")]
    NotUtf8(core::str::Utf8Error),
    /// The pixel retina's adapter refused the JSON — malformed, wrong schema,
    /// or a region kind outside the closed vocabulary. Fail-loud is the point:
    /// a producer that drifts is caught at the seam.
    DocV1(ogar_from_docv1::FromDocV1Error),
    /// The recognizer refused the page (`ocr` feature only).
    #[cfg(feature = "ocr")]
    Ocr(tesseract_ogar::OcrExecError),
    /// The recognizer answered, but not with a document (`ocr` feature only).
    /// Structurally unreachable for a `RecognizeDocument` request; kept
    /// because an enum match that cannot fail is a claim, not a guarantee.
    #[cfg(feature = "ocr")]
    NotADocument,
}

impl core::fmt::Display for IntakeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            #[cfg(feature = "dom")]
            Self::NotUtf8(e) => write!(f, "the DOM retina needs UTF-8: {e}"),
            Self::DocV1(e) => write!(f, "doc.v1 refused at the seam: {e}"),
            #[cfg(feature = "ocr")]
            Self::Ocr(e) => write!(f, "recognition failed: {e:?}"),
            #[cfg(feature = "ocr")]
            Self::NotADocument => {
                write!(f, "the recognizer returned a non-document response")
            }
        }
    }
}

impl std::error::Error for IntakeError {}

/// DOM retina: an HTML page → one `DocIr`, gated.
///
/// # Errors
/// [`IntakeError::NotUtf8`] if the bytes are not UTF-8.
#[cfg(feature = "dom")]
pub fn ingest_html<I: DedupIndex>(bytes: &[u8], index: &I) -> Result<Ingested, IntakeError> {
    let (hash, verdict) = paperless_kv::preflight(bytes, index);
    if let Verdict::Duplicate { matched } = verdict {
        return Ok(Ingested::Duplicate { hash, matched });
    }
    let html = core::str::from_utf8(bytes).map_err(IntakeError::NotUtf8)?;
    Ok(Ingested::Novel {
        hash,
        ir: Box::new(spider_doc_ir::harvest(html)),
    })
}

/// Pixel retina, JSON leg: `doc.v1` from any producer → one `DocIr`, gated.
///
/// `source_bytes` are the ORIGINAL image bytes — what the gate hashes and what
/// the IR's identity is taken over. The JSON is a rendition of them, so
/// hashing the JSON instead would make two renderings of one scan look like
/// two documents.
///
/// # Errors
/// [`IntakeError::DocV1`] if the JSON is malformed, carries the wrong schema
/// marker, or names a region kind outside the closed vocabulary.
pub fn ingest_doc_v1<I: DedupIndex>(
    source_bytes: &[u8],
    doc_v1_json: &str,
    mime: &str,
    index: &I,
) -> Result<Ingested, IntakeError> {
    let (hash, verdict) = paperless_kv::preflight(source_bytes, index);
    if let Verdict::Duplicate { matched } = verdict {
        return Ok(Ingested::Duplicate { hash, matched });
    }
    let ir = ogar_from_docv1::from_doc_v1(doc_v1_json, hash.0, mime).map_err(IntakeError::DocV1)?;
    Ok(Ingested::Novel {
        hash,
        ir: Box::new(ir),
    })
}

/// Pixel retina, full leg: a grey page → recognition → `doc.v1` → one `DocIr`,
/// gated. Behind the off-by-default `ocr` feature.
///
/// The gate runs before the executor is touched, so a duplicate page costs a
/// hash and a lookup rather than a recognition pass. That ordering is the
/// reason S-2 exists and it is asserted, not assumed
/// (see `duplicate_never_reaches_the_retina`).
///
/// # Errors
/// [`IntakeError::Ocr`] if recognition fails, [`IntakeError::NotADocument`] if
/// the executor answers with another response shape, [`IntakeError::DocV1`] if
/// its own `doc.v1` does not pass the seam.
#[cfg(feature = "ocr")]
pub fn ingest_image<I: DedupIndex>(
    grey: &[u8],
    width: usize,
    height: usize,
    mime: &str,
    executor: &tesseract_ogar::OcrExecutor,
    index: &I,
) -> Result<Ingested, IntakeError> {
    use tesseract_ogar::{BinarizeMode, OcrRequest, OcrResponse};

    let (hash, verdict) = paperless_kv::preflight(grey, index);
    if let Verdict::Duplicate { matched } = verdict {
        return Ok(Ingested::Duplicate { hash, matched });
    }
    let resp = executor
        .execute(OcrRequest::RecognizeDocument {
            grey,
            width,
            height,
            with_dict: false,
            harvest_profile: None,
            binarize: BinarizeMode::default(),
        })
        .map_err(IntakeError::Ocr)?;
    let OcrResponse::DocumentOut { doc_json, .. } = resp else {
        return Err(IntakeError::NotADocument);
    };
    let ir = ogar_from_docv1::from_doc_v1(&doc_json, hash.0, mime).map_err(IntakeError::DocV1)?;
    Ok(Ingested::Novel {
        hash,
        ir: Box::new(ir),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use paperless_kv::DocumentGuid;
    use std::cell::Cell;

    /// An index that knows nothing.
    struct Empty;
    impl DedupIndex for Empty {
        fn look_up(&self, _: &ContentSha256) -> Option<(DocumentGuid, MatchedOn)> {
            None
        }
    }

    /// An index that has seen everything, and counts how often it was asked.
    struct SeenAll {
        asked: Cell<usize>,
        matched: MatchedOn,
    }
    impl DedupIndex for SeenAll {
        fn look_up(&self, _: &ContentSha256) -> Option<(DocumentGuid, MatchedOn)> {
            self.asked.set(self.asked.get() + 1);
            Some((
                DocumentGuid(lance_graph_contract::facet::FacetCascade::default()),
                self.matched,
            ))
        }
    }

    /// The DOM fixture. Gated with its tests: without `dom` nothing reads it,
    /// and an unused-const warning in the DEFAULT build would be noise a real
    /// warning could hide behind.
    #[cfg(feature = "dom")]
    const PAGE: &str = "<html><body><header>Acme GmbH</header><main>Invoice body text \
                        here</main><footer>Seite 1</footer></body></html>";

    #[cfg(feature = "dom")]
    #[test]
    fn dom_retina_produces_one_shape() {
        let out = ingest_html(PAGE.as_bytes(), &Empty).expect("utf-8");
        let ir = out.ir().expect("novel");
        assert_eq!(ir.source, ogar_doc_ir::Provenance::Dom);
        assert_eq!(ir.version, ogar_doc_ir::DOC_IR_VERSION);
        // And it passes the IR's OWN load gate, which is the contract both
        // retinas are held to.
        let json = ogar_doc_ir::to_json(ir).expect("serialize");
        assert!(ogar_doc_ir::from_json(&json).is_ok());
        assert!(
            !ir.pages[0].regions.is_empty(),
            "the DOM landmarks must become regions, or this proves nothing"
        );
    }

    #[cfg(feature = "dom")]
    #[test]
    fn dom_identity_matches_the_gate() {
        // `spider_doc_ir::harvest` hashes `content.as_bytes()`; `preflight`
        // hashes the same bytes. If these ever diverge, a document's gate
        // identity and its IR identity would disagree — silently.
        let out = ingest_html(PAGE.as_bytes(), &Empty).expect("utf-8");
        let ir = out.ir().expect("novel");
        assert_eq!(
            &ir.content_sha256,
            &out.hash().0,
            "the retina's identity and the S-2 gate's must be the same hash"
        );
    }

    #[cfg(feature = "dom")]
    #[test]
    fn duplicate_never_reaches_the_retina() {
        // Deliberately NOT valid HTML and NOT valid UTF-8 tail: if the retina
        // ran at all, `ingest_html` would have to decode these bytes and would
        // fail. Reaching `Duplicate` proves the gate short-circuits FIRST,
        // which is the whole of S-2.
        let bytes = b"\xff\xfe not utf-8 at all";
        let index = SeenAll {
            asked: Cell::new(0),
            matched: MatchedOn::Original,
        };
        let out = ingest_html(bytes, &index).expect("the gate answers before decoding");
        assert!(matches!(out, Ingested::Duplicate { .. }));
        assert_eq!(index.asked.get(), 1, "the gate is asked exactly once");
        // The same bytes as a NOVEL input must fail — proving the test above
        // is not passing because the bytes are somehow acceptable.
        assert!(matches!(
            ingest_html(bytes, &Empty),
            Err(IntakeError::NotUtf8(_))
        ));
    }

    #[cfg(feature = "dom")]
    #[test]
    fn derived_artifact_match_is_reported_as_such() {
        let index = SeenAll {
            asked: Cell::new(0),
            matched: MatchedOn::Derived,
        };
        let out = ingest_html(PAGE.as_bytes(), &index).expect("gate");
        assert!(matches!(
            out,
            Ingested::Duplicate {
                matched: MatchedOn::Derived,
                ..
            }
        ));
    }

    #[test]
    fn doc_v1_seam_fails_loud_on_a_drifted_producer() {
        let good = r#"{"schema":"tesseract-rs/doc.v1","pages":[{"page":0,"width":100,
            "height":100,"regions":[{"type":"text","bbox":[0,0,50,50],"lines":[]}]}]}"#;
        let img = b"pretend-these-are-image-bytes";
        let ok = ingest_doc_v1(img, good, "image/png", &Empty).expect("valid doc.v1");
        let ir = ok.ir().expect("novel");
        assert_eq!(ir.source, ogar_doc_ir::Provenance::Ocr);
        assert_eq!(ir.mime, "image/png");
        // Identity is over the IMAGE bytes, not the JSON: two renderings of one
        // scan must not look like two documents.
        assert_eq!(&ir.content_sha256, &ContentSha256::of(img).0);

        // An off-vocabulary region kind is refused at the seam.
        let drifted = good.replace(r#""type":"text""#, r#""type":"paragraph""#);
        assert!(matches!(
            ingest_doc_v1(img, &drifted, "image/png", &Empty),
            Err(IntakeError::DocV1(_))
        ));
        // ...and so is a wrong schema marker.
        let wrong = good.replace("tesseract-rs/doc.v1", "tesseract-rs/doc.v2");
        assert!(matches!(
            ingest_doc_v1(img, &wrong, "image/png", &Empty),
            Err(IntakeError::DocV1(_))
        ));
    }

    #[test]
    fn a_duplicate_never_reaches_the_doc_v1_seam() {
        // The pixel leg's own S-2 proof. The JSON below is garbage: if the
        // seam ran at all, `from_doc_v1` would refuse it and this would be
        // `Err(DocV1)`. Landing on `Duplicate` proves the gate answered
        // FIRST — which is the whole of S-2, and is a per-entry-point
        // property, not something the DOM leg's test can establish for this
        // one.
        let junk = "}{ not json";
        let img = b"already-seen-image-bytes";
        let index = SeenAll {
            asked: Cell::new(0),
            matched: MatchedOn::Original,
        };
        let out = ingest_doc_v1(img, junk, "image/png", &index).expect("the gate answers first");
        assert!(matches!(out, Ingested::Duplicate { .. }));
        assert_eq!(index.asked.get(), 1, "the gate is asked exactly once");
        // ...and the same junk as a NOVEL input must fail, or the assertion
        // above would pass for the wrong reason.
        assert!(matches!(
            ingest_doc_v1(img, junk, "image/png", &Empty),
            Err(IntakeError::DocV1(_))
        ));
    }

    #[cfg(feature = "dom")]
    #[test]
    fn both_retinas_land_in_one_type() {
        // The point of the whole crate: a web page and a scan are the same
        // Rust type, distinguishable only by the field that says so.
        let dom = ingest_html(PAGE.as_bytes(), &Empty).expect("utf-8");
        let pixel = ingest_doc_v1(
            b"img",
            r#"{"schema":"tesseract-rs/doc.v1","pages":[{"page":0,"width":10,"height":10,
               "regions":[{"type":"text","bbox":[0,0,5,5],"lines":[]}]}]}"#,
            "image/png",
            &Empty,
        )
        .expect("valid doc.v1");
        let irs: Vec<&DocIr> = vec![dom.ir().expect("novel"), pixel.ir().expect("novel")];
        assert_eq!(irs.len(), 2);
        assert_eq!(irs[0].source, ogar_doc_ir::Provenance::Dom);
        assert_eq!(irs[1].source, ogar_doc_ir::Provenance::Ocr);
        assert!(irs.iter().all(|i| i.version == ogar_doc_ir::DOC_IR_VERSION));
    }
}
