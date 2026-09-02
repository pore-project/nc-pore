#![allow(dead_code)]

//! Local recording preservation boundary.
//!
//! Preservation is the first owned representation after capture. It keeps
//! the capture representation and its provenance intact and does not choose
//! a transport format. In particular, an F32 capture remains F32 here rather
//! than being silently quantized to PCM24 merely because V1 transport uses
//! FLAC.
//!
//! This boundary is intentionally in-memory for now. Durable local storage,
//! completion jobs and upload orchestration remain subsequent layers.

use crate::audio::{CaptureResult, CaptureStatus, CaptureTrack};

/// Owned, transport-neutral representation of a completed local capture.
///
/// The preservation boundary deliberately retains the exact capture chunks,
/// technical configuration and source provenance supplied by the capture
/// layer. No sample-format conversion occurs here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreservedCapture {
    inner: CaptureResult,
}

impl PreservedCapture {
    /// Creates a preservation snapshot from a completed capture result.
    pub fn from_capture_result(capture_result: CaptureResult) -> Self {
        Self {
            inner: capture_result,
        }
    }

    /// Returns the capture identifier.
    pub fn id(&self) -> &str {
        self.inner.id()
    }

    /// Returns the capture status retained at the preservation boundary.
    pub fn status(&self) -> &CaptureStatus {
        self.inner.status()
    }

    /// Returns the preserved technical tracks.
    pub fn tracks(&self) -> &[CaptureTrack] {
        self.inner.tracks()
    }

    /// Returns the preserved capture as an owned capture result.
    ///
    /// This is an explicit boundary crossing for consumers that still use the
    /// capture-result representation while the preservation/storage pipeline
    /// is being introduced incrementally.
    pub fn into_capture_result(self) -> CaptureResult {
        self.inner
    }
}

/// Creates transport-neutral preservation snapshots from completed captures.
pub struct CapturePreserver;

impl CapturePreserver {
    /// Preserves the capture representation without changing sample format,
    /// payload bytes, track identity or source provenance.
    pub fn preserve(capture_result: CaptureResult) -> PreservedCapture {
        PreservedCapture::from_capture_result(capture_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{
        CaptureChunk, CaptureSourceProvenance, RecordingConfiguration, SampleFormat,
    };

    // TEST-42
    //
    // Protects the capture -> preservation boundary:
    // preservation retains the capture representation instead of converting it.
    #[test]
    fn preservation_retains_capture_representation_and_provenance() {
        let configuration = RecordingConfiguration::new(48_000, 1, SampleFormat::F32);
        let provenance = CaptureSourceProvenance::new("device-1", 1_762_000_000_000)
            .with_label("Microphone")
            .ended_at(1_762_000_005_000);

        let mut track = CaptureTrack::with_configuration("track-1", configuration);
        track.set_source_provenance(provenance.clone());
        track.add_chunk(CaptureChunk::with_payload(1, vec![1, 2, 3, 4]));

        let mut capture = CaptureResult::new("capture-001");
        capture.add_track(track);

        let preserved = CapturePreserver::preserve(capture);

        assert_eq!(preserved.id(), "capture-001");
        assert_eq!(preserved.tracks().len(), 1);
        assert_eq!(preserved.tracks()[0].configuration(), Some(configuration));
        assert_eq!(preserved.tracks()[0].source_provenance(), Some(&provenance));
        assert_eq!(preserved.tracks()[0].chunks()[0].payload(), &[1, 2, 3, 4]);
    }

    // TEST-43
    //
    // Protects the preservation boundary:
    // F32 remains F32 until a later transport conversion explicitly chooses
    // a transport representation.
    #[test]
    fn preservation_does_not_convert_f32_to_pcm24() {
        let configuration = RecordingConfiguration::new(48_000, 1, SampleFormat::F32);
        let track = CaptureTrack::with_configuration("track-1", configuration);
        let mut capture = CaptureResult::new("capture-002");
        capture.add_track(track);

        let preserved = CapturePreserver::preserve(capture);

        assert_eq!(
            preserved.tracks()[0].configuration().unwrap().sample_format(),
            SampleFormat::F32
        );
    }

    // TEST-44
    //
    // Protects the incremental architecture:
    // a preserved capture can cross back into the legacy capture-result API
    // without changing its technical data.
    #[test]
    fn preserved_capture_can_be_recovered_as_capture_result() {
        let mut capture = CaptureResult::new("capture-003");
        let mut track = CaptureTrack::new("track-1");
        track.add_chunk(CaptureChunk::with_payload(1, vec![9, 8, 7]));
        capture.add_track(track);

        let preserved = CapturePreserver::preserve(capture);
        let recovered = preserved.into_capture_result();

        assert_eq!(recovered.id(), "capture-003");
        assert_eq!(recovered.tracks()[0].chunks()[0].payload(), &[9, 8, 7]);
    }
}
