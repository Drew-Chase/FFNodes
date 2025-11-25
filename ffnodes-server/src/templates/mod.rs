/// FFmpeg template parser for simple placeholder replacement
#[allow(dead_code)]
pub struct TemplateParser {
    template: String,
}

#[allow(dead_code)]
impl TemplateParser {
    pub fn new(template: String) -> Self {
        Self { template }
    }

    /// Parse template and replace placeholders
    /// Supported placeholders:
    /// - {HWACCEL_CODE}: Hardware acceleration code (e.g., "_nvenc", "_amf", "")
    /// - {INPUT}: Input file path
    /// - {OUTPUT}: Output file path
    /// - {PRESET}: Encoding preset (e.g., "fast", "medium", "slow")
    /// - {CRF}: Constant Rate Factor for quality
    pub fn parse(&self, replacements: &[(& str, &str)]) -> String {
        let mut result = self.template.clone();

        for (placeholder, value) in replacements {
            result = result.replace(placeholder, value);
        }

        result
    }

    /// Build complete FFmpeg command arguments from template
    pub fn build_args(
        &self,
        hwaccel_code: &str,
        input: &str,
        output: &str,
    ) -> Vec<String> {
        let command = self.parse(&[
            ("{HWACCEL_CODE}", hwaccel_code),
            ("{INPUT}", input),
            ("{OUTPUT}", output),
        ]);

        // Split command into arguments
        command
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_parsing() {
        let template = TemplateParser::new(
            "-c:v h264{HWACCEL_CODE} -preset medium -crf 23 -i {INPUT} {OUTPUT}".to_string(),
        );

        let args = template.build_args("_nvenc", "input.mp4", "output.mp4");

        assert_eq!(
            args,
            vec![
                "-c:v",
                "h264_nvenc",
                "-preset",
                "medium",
                "-crf",
                "23",
                "-i",
                "input.mp4",
                "output.mp4"
            ]
        );
    }

    #[test]
    fn test_template_no_hwaccel() {
        let template = TemplateParser::new(
            "-c:v h264{HWACCEL_CODE} -preset medium -i {INPUT} {OUTPUT}".to_string(),
        );

        let args = template.build_args("", "input.mp4", "output.mp4");

        assert_eq!(
            args,
            vec![
                "-c:v",
                "h264",
                "-preset",
                "medium",
                "-i",
                "input.mp4",
                "output.mp4"
            ]
        );
    }
}
