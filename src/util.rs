// MView6 -- High-performance PDF and photo viewer built with Rust and GTK4
//
// Copyright (c) 2024-2025 Martin van der Werff <github (at) newinnovations.nl>
//
// This file is part of MView6.
//
// MView6 is free software: you can redistribute it and/or modify it under the terms of
// the GNU Affero General Public License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR
// IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
// DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR
// BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
// STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use glib::{ffi::g_source_remove, result_from_gboolean, BoolError, SourceId};

/// Safer alternative to SourceId::remove()
pub fn remove_source_id(id: SourceId) -> Result<(), BoolError> {
    unsafe { result_from_gboolean!(g_source_remove(id.as_raw()), "Failed to remove source") }
}

pub fn has_changed_by_percentage(original: f64, new: f64, threshold_percent: f64) -> bool {
    if original == 0.0 {
        return new != 0.0;
    }
    let percent_change = ((new - original) / original).abs();
    percent_change >= (threshold_percent / 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_changed_by_percentage() {
        // 5% increase: 100 -> 105
        assert!(has_changed_by_percentage(100.0, 105.0, 5.0));

        // 5% decrease: 100 -> 95
        assert!(has_changed_by_percentage(100.0, 95.0, 5.0));

        // 10% increase: 100 -> 110 (checking for 5% threshold)
        assert!(has_changed_by_percentage(100.0, 110.0, 5.0));

        // 3% increase: 100 -> 103 (checking for 5% threshold - should be false)
        assert!(!has_changed_by_percentage(100.0, 103.0, 5.0));

        // 15% increase: 100 -> 115 (checking for 10% threshold)
        assert!(has_changed_by_percentage(100.0, 115.0, 10.0));

        // 8% increase: 100 -> 108 (checking for 10% threshold - should be false)
        assert!(!has_changed_by_percentage(100.0, 108.0, 10.0));

        // Edge case: zero original value
        assert!(has_changed_by_percentage(0.0, 1.0, 5.0));
        assert!(!has_changed_by_percentage(0.0, 0.0, 5.0));
    }
}
