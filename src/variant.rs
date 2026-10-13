/*

    Copyright (C) 2026  Stevens Benavides

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum CompilerVariant {
    #[default]
    Release,
    Stripped,
}

impl CompilerVariant {
    #[inline]
    pub fn binary_stem(self) -> &'static str {
        match self {
            Self::Release => "thrustc",
            Self::Stripped => "thrustc-stripped",
        }
    }
}

impl std::fmt::Display for CompilerVariant {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: &str = match self {
            Self::Release => "release",
            Self::Stripped => "stripped",
        };

        formatter.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_binary_stems() {
        assert_eq!(CompilerVariant::Release.binary_stem(), "thrustc");
        assert_eq!(CompilerVariant::Stripped.binary_stem(), "thrustc-stripped");
    }

    #[test]
    fn renders_names() {
        assert_eq!(CompilerVariant::Release.to_string(), "release");
        assert_eq!(CompilerVariant::Stripped.to_string(), "stripped");
    }

    #[test]
    fn defaults_to_release() {
        assert_eq!(CompilerVariant::default(), CompilerVariant::Release);
    }

    #[test]
    fn deserializes_lowercase_names() {
        let variant: CompilerVariant = serde_yaml::from_str("stripped").unwrap();

        assert_eq!(variant, CompilerVariant::Stripped);
    }
}
