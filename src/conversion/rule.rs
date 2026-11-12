//! Rules Engine

use bstr::ByteSlice;

use crate::{conversion::Hypothesis, dictionary::StringTable, model::Candidate};

/// Rule based filter
#[derive(Debug)]
pub struct RuleFilter {
    st: StringTable,
    enabled: bool,
}

#[rustfmt::skip]
const NUMBERS: &[&str] = &[
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "0",
    "１", "２", "３", "４", "５", "６", "７", "８", "９", "０",
    "一", "二", "三", "四", "五", "六", "七", "八", "九", "十",
    "零", "百", "千", "萬", "億"
];

#[rustfmt::skip]
const NUM_X: &[&str] = &[
    "年", "月", "日", "路", "巷", "弄", "鄰", "里", "號", "樓",
    "項", "個", "隻",
];

#[rustfmt::skip]
const ADDR_CTX: &[&str] = &[
    "區", "路", "巷", "弄", "鄰", "里", "號", "樓",
];

impl RuleFilter {
    pub fn new(string_table: StringTable) -> RuleFilter {
        RuleFilter {
            st: string_table,
            enabled: false,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn rerank(&self, hyp: Vec<Hypothesis>) -> Vec<Hypothesis> {
        if !self.enabled {
            return hyp;
        }
        let mut with_score: Vec<(i32, Hypothesis)> = hyp
            .into_iter()
            .map(|hyp| {
                let mut score: i32 = 0;
                let surface = self.to_surface(&hyp);
                for start in 0..surface.len() {
                    score -= self.match_num_x(&surface[start..]);
                    score -= self.match_nth(&surface[start..]);
                }
                score -= self.match_more_more(&surface);
                score -= self.match_address(&surface);
                (score, hyp)
            })
            .collect();
        with_score.sort_by_key(|p| p.0);
        with_score.into_iter().map(|p| p.1).collect()
    }

    fn to_surface(&self, hyp: &Hypothesis) -> Vec<String> {
        hyp.candidates
            .iter()
            .map(|cand| match cand {
                Candidate::None => String::new(),
                Candidate::Word { wid, .. } => self.st.get_text(*wid).unwrap_or(String::new()),
                Candidate::Grapheme { text, .. } => text.clone(),
            })
            .collect::<String>()
            .as_bytes()
            .graphemes()
            .map(|s| s.to_owned())
            .collect()
    }

    fn match_num_x(&self, cand: &[String]) -> i32 {
        if cand.len() < 2 {
            return 0;
        }
        if !NUMBERS.iter().any(|&n| n == cand[0]) {
            return 0;
        }
        if NUM_X.iter().any(|&n| n == cand[1]) {
            return 1;
        }
        0
    }
    fn match_address(&self, cand: &[String]) -> i32 {
        if cand.iter().any(|w| ADDR_CTX.iter().any(|a| a == w)) {
            let score = cand
                .iter()
                .filter(|w| ADDR_CTX.iter().any(|a| a == w))
                .count() as i32;
            if score > 1 {
                return score;
            }
        }
        0
    }
    fn match_nth(&self, cand: &[String]) -> i32 {
        if cand.len() < 2 {
            return 0;
        }
        if cand[0] != "第" {
            return 0;
        }
        if NUM_X.iter().any(|&n| n == cand[1]) {
            return 1;
        }
        0
    }
    fn match_more_more(&self, cand: &[String]) -> i32 {
        if cand.iter().filter(|&n| n == "越").count() >= 2 {
            return 1;
        }
        0
    }
}
