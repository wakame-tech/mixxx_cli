fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

#[derive(Debug)]
pub struct SteppedTempoFilter {
    spans: Vec<(f32, f32, f32)>,
}

impl SteppedTempoFilter {
    pub fn new(from: (f32, f32), to: (f32, f32), steps: usize) -> Self {
        let mut spans = vec![];
        for i in 1..=steps {
            let begin = lerp(from.0, to.0, (i - 1) as f32 / steps as f32);
            let end = lerp(from.0, to.0, i as f32 / steps as f32);
            let v = lerp(from.1, to.1, (i - 1) as f32 / steps as f32);
            spans.push((begin, end, v));
        }
        Self { spans }
    }

    pub fn to_filters(&self, index: usize) -> Vec<String> {
        let mut i = index;
        let mut filters = vec![];
        let mut src_labels = vec![];
        let mut dst_labels = vec![];

        for (begin, end, scale) in self.spans.iter() {
            filters.push(format!(
                "[0_{}] atrim={}:{} [0_{}]",
                i + 1,
                begin,
                end,
                i + 2
            ));
            src_labels.push(format!("[0_{}]", i + 1));
            filters.push(format!("[0_{}] atempo={} [0_{}]", i + 2, scale, i + 3));
            dst_labels.push(format!("[0_{}]", i + 3));
            i += 3;
        }
        filters.insert(
            0,
            format!(
                "[0_{}] asplit={} {}",
                index,
                self.spans.len(),
                src_labels.join("")
            ),
        );
        filters.push(format!(
            "{} concat=n={}:v=0:a=1",
            dst_labels.join(""),
            dst_labels.len()
        ));
        filters
    }
}

#[cfg(test)]
mod tests {
    use super::SteppedTempoFilter;

    #[test]
    fn test_pts_filter() {
        let filter = SteppedTempoFilter::new((0.0, 1.0), (20.0, 1.8), 4);
        dbg!(&filter);
        dbg!(filter.to_filters(2));
    }
}
