use std::collections::{HashMap, HashSet};

use graph::{
    page_rank::page_rank,
    prelude::{DirectedCsrGraph, GraphBuilder},
};
use ndarray::{Array2, Zip};
use once_cell::sync::Lazy;
use ordered_float::NotNan;
use regex::Regex;
use rust_stemmers::Stemmer;
use stopwords::{Language, Stopwords, NLTK};
use unicode_segmentation::UnicodeSegmentation;

static EN_STOPWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    NLTK::stopwords(Language::English)
        .unwrap()
        .iter()
        .copied()
        .collect()
});

fn clean(text: &str) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new("\\[.*?\\]|\"|\\n").unwrap());

    RE.replace_all(text, "").to_string()
}

fn sentences(text: &str) -> Vec<&str> {
    // text.unicode_sentences().collect()
    let mut sentences = Vec::with_capacity(64);

    for sentence in cutters::cut(text, cutters::Language::English) {
        println!("{sentence:?}");

        sentences.push(sentence.str);

        for quote in sentence.quotes {
            sentences.extend(quote.sentences);
        }
    }

    sentences
}

pub fn summarize(text: &str) -> String {
    let cleaned = clean(&text);
    let original_sentences = sentences(&cleaned);
    let sentences: Vec<_> = original_sentences
        .iter()
        .copied()
        .map(str::to_lowercase)
        .collect();

    for sentence in &sentences {
        println!("{sentence}");
    }

    let stemmer = Stemmer::create(rust_stemmers::Algorithm::English);
    let mut sentence_stem_freqs: Vec<HashMap<_, usize>> = Vec::with_capacity(sentences.len());

    for sentence in &sentences {
        let mut stem_freqs: HashMap<_, usize> = HashMap::with_capacity(4);

        for word in sentence.unicode_words() {
            if EN_STOPWORDS.contains(word) {
                continue;
            }

            let stem = stemmer.stem(word);
            *stem_freqs.entry(stem).or_default() += 1;
        }

        sentence_stem_freqs.push(stem_freqs);
    }

    let global_stem_freqs =
        sentence_stem_freqs
            .iter()
            .fold(HashMap::new(), |mut freqs, sentence_freqs| {
                freqs.extend(sentence_freqs);
                freqs
            });

    let mut unique_stems: Vec<(_, _)> = global_stem_freqs.iter().collect();
    unique_stems.sort();

    let stem_idfs: HashMap<_, f32> = unique_stems
        .iter()
        .map(|(stem, _)| {
            let document_count = sentences.len() as f32;
            let usage_count = sentence_stem_freqs
                .iter()
                .filter(|freqs| freqs.contains_key(**stem))
                .count() as f32;
            let idf = (document_count / usage_count).log10();

            (stem, idf)
        })
        .collect();

    println!("{stem_idfs:?}");

    let mut sentence_vectors: Array2<f32> = Array2::zeros([sentences.len(), unique_stems.len()]);
    Zip::indexed(&mut sentence_vectors).par_for_each(|(i, j), tfidf| {
        let (stem, _) = &unique_stems[j];

        let idf: f32 = stem_idfs[stem];

        let sentence_freqs = &sentence_stem_freqs[i];
        let freq_in_sentence = sentence_freqs.get(**stem).copied().unwrap_or(0) as f32;
        let total_in_sentence: f32 =
            sentence_freqs.iter().map(|(_, freq)| freq).sum::<usize>() as f32;

        let tf = freq_in_sentence / total_in_sentence;

        *tfidf = tf * idf;

        if freq_in_sentence > 0.0 {
            *tfidf += 1.0;
        }
    });

    let mut edges = Vec::new();
    for (i, vec_a) in sentence_vectors.rows().into_iter().enumerate() {
        for (j, vec_b) in sentence_vectors.rows().into_iter().enumerate() {
            if i == j {
                continue;
            }

            let vec_a_l2_norm = vec_a.dot(&vec_a).sqrt();
            let vec_b_l2_norm = vec_b.dot(&vec_b).sqrt();

            let sim = vec_a.dot(&vec_b) / (vec_a_l2_norm * vec_b_l2_norm);

            if sim > 0.1 {
                edges.push((i, j));
            }
        }
    }

    let graph: DirectedCsrGraph<usize> = GraphBuilder::new().edges(edges).build();
    let (ranks, _iterations, _) = page_rank(&graph, 10, 1E-4);

    let mut ranks: Vec<_> = ranks
        .into_iter()
        .enumerate()
        .map(|(i, rank)| (NotNan::new(rank).unwrap(), i))
        .collect();

    ranks.sort();

    let mut top: Vec<_> = ranks.into_iter().rev().take(3).map(|(_, i)| i).collect();
    top.sort();

    let top: Vec<_> = top.into_iter().map(|i| original_sentences[i]).collect();
    top.join(" ")
}

#[cfg(test)]
mod tests {
    // use crate::Summarizer;
    use super::summarize;

    #[test]
    fn test_summary() {
        // let text = "This is some Mr. Fisher's example text. It contains two sentences.";
        let text = "He is Walter. He is William. He isn't Peter or September.";
        println!("{:?}", summarize(text));
    }

    #[test]
    fn monk_summary() {
        // let text = "This is some Mr. Fisher's example text. It contains two sentences.";
        let text = r#"Monk carries out futile and endless attempts to make the world "balanced".[19][20] Monk is fixated with symmetry,[21] going so far as to always cut his pancakes into squares.[22] He strongly prefers familiarity and rigorous structure in his activities. Monk only drinks Sierra Springs water throughout seasons 1–5 and a fictional brand (Summit Creek) throughout seasons 6–8, to the point that in the season 2 episode "Mr. Monk Goes to Mexico", Monk goes without drinking for several days because he cannot find any Sierra Springs. Monk also has great difficulty in standard social situations, so much so that he must write down common small talk phrases on note cards in an attempt to successfully socialize.[23] While his obsessive attention to minute detail cripples him socially, it makes him a gifted detective and profiler.[9] He has a photographic memory,[17] and can reconstruct entire crime scenes based on little more than scraps of detail that seem unimportant to his colleagues.[15] His trademark method of examining a crime scene, which Sharona used to call his "Zen Sherlock Holmes thing", is to wander seemingly aimlessly around a crime scene, occasionally holding up his hands, as though framing a shot for a photograph.[24] Shalhoub explained in an interview that Monk does this because it "isolates and cuts the crime scene into slices" and causes Monk to look at parts of the crime scene instead of the whole.[24]

        Monk's delicate mental condition means that his ability to function can be severely impaired by a variety of factors. One example is shown during the season 5 episode "Mr. Monk and the Garbage Strike", in which the smell of garbage prevents Monk from being able to easily identify the murderer of sanitation union boss Jimmy Cusack, eventually causing him to have a psychotic break. Another example is when entering a chaotic murder scene in the episode "Mr. Monk Meets Dale the Whale", his first impulse is to straighten the lamps, though he is frequently able to hold off his fixations when examining bodies or collecting evidence.[25] Even though Monk's mental state in the series is said to be a result of his wife's death,[15][26] he shows signs of OCD in flashbacks dating back to childhood.[27] To deal with his OCD and phobias, Monk visits a psychiatrist – Dr. Charles Kroger (Stanley Kamel) in the first six seasons and Dr. Neven Bell (Héctor Elizondo) in the last two seasons – weekly, and at several points, daily.[28] Moments of extreme stress can cause Monk to enter a dissociative state, as seen in the Season 1 episode "Mr. Monk and the Earthquake"; he begins speaking in gibberish during these periods, severely hindering his investigative skills.
        
        Over the course of the show (roughly 8 years), Monk overcomes many of his phobias and some aspects of his OCD. Though he has not been cured of many of them, if any at all, he has been able to put them in the back of his mind when involved in case work. One breakthrough is shown in the season 8 episode "Mr. Monk Goes to Group Therapy", when Adrian is locked in a car trunk with his fellow OCD patient and personal rival, Harold Krenshaw. During the terrifying trip, both men overcome their longstanding claustrophobia (fear of small spaces), as well as their own differences, resulting in them becoming friends. Possibly due to this, as well as the many cases Monk has solved over the years, he is reinstated as detective first class by Stottlemeyer in the season 8 episode "Mr. Monk and the Badge". Though he is very excited about his reinstatement initially, Monk realizes that becoming a detective again did not mean that he would be happier. In a session with Dr. Bell, Monk realizes he was always happy as a private detective and consultant to the SFPD as his own boss. After overcoming his fear of heights and singlehandedly capturing a killer window-washer, Monk turns in his badge. In the series finale, he learns that his late wife, Trudy, had given birth to a daughter before they had met. The knowledge and events of the episode lead to him becoming more cheerful. "#;
        // let text = r#"Monk carries out futile and endless attempts to make the world balanced. Monk is fixated with symmetry, going so far as to always cut his pancakes into squares."#;
        println!("{:?}", summarize(text));
    }
}
