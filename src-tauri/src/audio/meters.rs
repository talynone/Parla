// Calcul RMS et peak en dB pour l'UI (barres d'activite du recorder).
//
// Reference VoiceInk : VoiceInk/CoreAudioRecorder.swift:600-629 (calculateMeters)
// VoiceInk expose des dB dans [-160, 0]. On fait pareil.

const SILENCE_DB: f32 = -160.0;

/// Convertit une amplitude lineaire [0, 1] en dBFS. Clamp au silence pour eviter log(0).
#[inline]
pub fn linear_to_db(amplitude: f32) -> f32 {
    if amplitude <= 1e-8 {
        SILENCE_DB
    } else {
        (20.0 * amplitude.log10()).max(SILENCE_DB)
    }
}

/// Calcule RMS et peak d'un buffer Float32 mono.
pub fn compute_rms_peak(samples: &[f32]) -> (f32, f32) {
    if samples.is_empty() {
        return (SILENCE_DB, SILENCE_DB);
    }
    let mut sum_sq = 0.0f64;
    let mut peak = 0.0f32;
    for &s in samples {
        sum_sq += (s as f64) * (s as f64);
        let abs = s.abs();
        if abs > peak {
            peak = abs;
        }
    }
    let rms = (sum_sq / samples.len() as f64).sqrt() as f32;
    (linear_to_db(rms), linear_to_db(peak))
}
