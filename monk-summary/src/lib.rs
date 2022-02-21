use std::collections::{HashMap, HashSet};

use ndarray::{Array2, Zip};
use once_cell::sync::Lazy;
use ordered_float::NotNan;
use rayon::{
    iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use regex::Regex;
use rust_stemmers::Stemmer;
use stopwords::{Language, Stopwords, NLTK};
use tracing::info;
use unicode_segmentation::UnicodeSegmentation;

use crate::pagerank::pagerank;

pub mod pagerank;

static EN_STOPWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    NLTK::stopwords(Language::English)
        .unwrap()
        .iter()
        .copied()
        .collect()
});

fn clean(text: &str) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new("\\[.*?\\]|\"|\\n").unwrap());

    info!("cleaning text");

    RE.replace_all(text, "").to_string()
}

fn sentences(text: &str) -> Vec<&str> {
    info!("creating sentences");
    let mut sentences = Vec::with_capacity(64);

    for sentence in cutters::cut(text, cutters::Language::English) {
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
        .par_iter()
        .copied()
        .map(str::to_lowercase)
        .collect();

    info!("creating sentence stem freqs");
    let stemmer = Stemmer::create(rust_stemmers::Algorithm::English);
    let sentence_stem_freqs: Vec<HashMap<_, usize>> = sentences
        .par_iter()
        .map(|sentence| {
            let mut stem_freqs: HashMap<_, usize> = HashMap::with_capacity(4);

            for word in sentence.unicode_words() {
                if EN_STOPWORDS.contains(word) {
                    continue;
                }

                let stem = stemmer.stem(word);
                *stem_freqs.entry(stem).or_default() += 1;
            }

            stem_freqs
        })
        .collect();

    let global_stem_freqs = sentence_stem_freqs.clone().into_par_iter().reduce(
        || HashMap::new(),
        |mut freqs, sentence_freqs| {
            for (stem, count) in sentence_freqs.into_iter() {
                *freqs.entry(stem).or_default() += count;
            }

            freqs
        },
    );

    let mut unique_stems: Vec<(_, _)> = global_stem_freqs.par_iter().collect();
    unique_stems.par_sort();

    let stem_idfs: HashMap<_, f32> = unique_stems
        .par_iter()
        .map(|(stem, _)| {
            let document_count = sentences.len() as f32;
            let usage_count = sentence_stem_freqs
                .par_iter()
                .filter(|freqs| freqs.contains_key(*stem))
                .count() as f32;
            let idf = (document_count / usage_count).log10();

            (stem, idf)
        })
        .collect();

    info!(
        "creating sentence vectors: {} x {}",
        sentences.len(),
        unique_stems.len()
    );
    let mut sentence_vectors: Array2<f32> = Array2::zeros([sentences.len(), unique_stems.len()]);
    Zip::indexed(&mut sentence_vectors).par_for_each(|(i, j), tfidf| {
        let (stem, _) = &unique_stems[j];

        let idf: f32 = stem_idfs[stem];

        let sentence_freqs = &sentence_stem_freqs[i];
        let freq_in_sentence = sentence_freqs.get(*stem).copied().unwrap_or(0) as f32;
        let total_in_sentence: f32 =
            sentence_freqs.iter().map(|(_, freq)| freq).sum::<usize>() as f32;

        let tf = freq_in_sentence / total_in_sentence;

        *tfidf = tf * idf;

        if freq_in_sentence > 0.0 {
            *tfidf += 1.0;
        }
    });

    info!(
        "constructing probabilities: {} x {}",
        sentences.len(),
        sentences.len()
    );
    let mut probabilities = Array2::zeros([sentences.len(), sentences.len()]);
    Zip::indexed(&mut probabilities).par_for_each(|(i, j), p| {
        if i == j {
            return;
        }

        let vec_i = sentence_vectors.row(i);
        let vec_j = sentence_vectors.row(j);

        let vec_i_l2_norm = vec_i.dot(&vec_i).sqrt();
        let vec_j_l2_norm = vec_j.dot(&vec_j).sqrt();

        let cos_sim = vec_i.dot(&vec_j) / (vec_i_l2_norm * vec_j_l2_norm);

        if !cos_sim.is_nan() {
            *p = cos_sim;
        }
    });

    // Row-wise sum should be equal to 1:
    Zip::from(probabilities.rows_mut()).par_for_each(|mut row| {
        if row.sum() != 0.0 {
            row /= row.sum();
        } else {
            row.fill(1.0 / sentences.len() as f32);
        }
    });

    let ranks = pagerank(&probabilities, 0.85, 1E-4);
    let mut ranks: Vec<_> = ranks
        .into_iter()
        .enumerate()
        .map(|(i, rank)| (NotNan::new(rank).unwrap(), i))
        .collect();

    ranks.par_sort();

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
        // Taken from: https://en.wikipedia.org/wiki/Adrian_Monk

        let text = r#"Character development
        Creation
        
        Monk was originally envisioned as a "more goofy and physical" Inspector Clouseau type of character.[3][4][5] However, co-creator David Hoberman came up with the idea of a detective with obsessive–compulsive disorder.[3] This was inspired by his own bout with self-diagnosed obsessive–compulsive disorder; in a Pittsburgh Post-Gazette interview, he stated that, "Like Monk, I couldn't walk on cracks and had to touch poles. I have no idea why—but if I didn't do these things, something terrible would happen."[4]
        
        Other fictional inspirations include Columbo[3][6][7] and Sherlock Holmes, and his obsession with neatness and order may be an homage to Hercule Poirot.[3] Like Holmes, and occasionally Poirot, Monk is accompanied by an earnest assistant with little or no detective ability, similar to Doctor Watson and Captain Hastings, respectively;[8] Monk's two major allies from the police department, Captain Stottlemeyer and Lieutenant Disher (credited as "Deacon" in the pilot episode), are reminiscent of Inspector Lestrade and Chief Inspector Japp, Holmes's and Poirot's well-meaning but ineffectual respective police counterparts. In addition, Monk has a brother whose abilities of deduction are even more amazing than his, yet much more geographically limited due to his own personal problems, somewhat in the style of Mycroft Holmes (who is more adept than Sherlock but also notoriously lazy).[6][9][10]
        
        When trying to think of a possible name for the character, co-creator Andy Breckman decided to look for a "simple monosyllabic last name".
        Casting
        Shalhoub was cast because the producers felt he could "bring the humor and passion of Monk to life".[2]
        
        Co-creator David Hoberman revealed that the casting sessions were "depressing".[11] USA Network's executive vice president Jeff Wachtel stated that looking for the right actor to portray Monk was "casting hell".[12] After two years of developing, the producers still had not found an actor to play the part.[11] Although Michael Richards was considered, distributors of the show ABC and Touchstone worried that the audience would typecast him for more comedic roles after his previous work as Cosmo Kramer on the sitcom series Seinfeld.[2][13] After Richards dropped out of the project, he went on to star in another series about a private detective, The Michael Richards Show, which was cancelled after six episodes.[14]
        Personality
        
            "Monk is a living legend. Quick, brilliant, analytical... [with] an encyclopedic knowledge of a dozen unconventional and assorted subjects, from door locks to horticulture to architecture to human psychology."
        
        Breckman's description of Monk.[9]
        Phobias
        
        In the script for the pilot, "Mr. Monk and the Candidate", Monk is described as being "a modern day Sherlock Holmes", only "nuts".[3] In the introductory scene of the episode, he is examining the scene of Nicole Vasques' murder, and picks up several important clues, but frequently interrupts himself to wonder aloud whether he left his stove on when he left the house that morning. In the season 6 episode "Mr. Monk and the Naked Man", Monk mentions that he has 312 phobias. The strongest of these phobias are: germs, dentists, sharp or pointed objects, vomiting, death and dead things, snakes, crowds, heights, fear, mushrooms, and small spaces, as Monk also mentions in the season 2 episode "Mr. Monk and the Very Very Old Man". In addition, new phobias develop at seemingly random intervals, such as a temporary fear of blankets at the end of the season 5 episode "Mr. Monk Gets a New Shrink". Though it is impossible to determine his strongest phobia, there does appear to be some form of hierarchy between them: in the series finale "Mr. Monk and the End", it is made clear that his fear of vomiting is greater than his fear of death. He has also stated, "Snakes trump heights!".
        
        Due to his overpowering fear of germs, Monk refuses to touch door handles and other common objects with his bare hands, avoids contact with anything dirty, and always uses sanitary wipes after human contact, including basic handshakes.[15] He is also unable to eat food that other people have touched—as shown in the season 7 episode "Mr. Monk Falls in Love" when he and Leyla Zlatavich go out to a Zemenian restaurant—and tends to throw away household items after people touch them, such as ladles and plastic storage containers.
        Assistants
        
        Monk's phobias and anxiety disorders make him depend on personal assistants, who drive him around, do his shopping, and always carry a supply of wipes for his use, as shown in episodes like "Mr. Monk Meets the Playboy", "Mr. Monk Goes to the Carnival", etc.[16] They also take active roles in organizing his consultancy work, and sometimes investigate cases themselves.[17] His first assistant, Sharona Fleming (Bitty Schram), is a single mother and practical nurse by profession, hired by the police department to help Monk recover from the three-year catatonic state he lapsed into after Trudy's death.[15] After several years of loyal service, Sharona leaves the show in season 3 to return to New Jersey and remarry her ex-husband Trevor.[18] After her abrupt departure, Monk has a chance meeting with Natalie Teeger (Traylor Howard), whom he hires as his new assistant starting in "Mr. Monk and the Red Herring".
        Fixations
        
        Monk carries out futile and endless attempts to make the world "balanced".[19][20] Monk is fixated with symmetry,[21] going so far as to always cut his pancakes into squares.[22] He strongly prefers familiarity and rigorous structure in his activities. Monk only drinks Sierra Springs water throughout seasons 1–5 and a fictional brand (Summit Creek) throughout seasons 6–8, to the point that in the season 2 episode "Mr. Monk Goes to Mexico", Monk goes without drinking for several days because he cannot find any Sierra Springs. Monk also has great difficulty in standard social situations, so much so that he must write down common small talk phrases on note cards in an attempt to successfully socialize.[23] While his obsessive attention to minute detail cripples him socially, it makes him a gifted detective and profiler.[9] He has a photographic memory,[17] and can reconstruct entire crime scenes based on little more than scraps of detail that seem unimportant to his colleagues.[15] His trademark method of examining a crime scene, which Sharona used to call his "Zen Sherlock Holmes thing", is to wander seemingly aimlessly around a crime scene, occasionally holding up his hands, as though framing a shot for a photograph.[24] Shalhoub explained in an interview that Monk does this because it "isolates and cuts the crime scene into slices" and causes Monk to look at parts of the crime scene instead of the whole.[24]
        
        Monk's delicate mental condition means that his ability to function can be severely impaired by a variety of factors. One example is shown during the season 5 episode "Mr. Monk and the Garbage Strike", in which the smell of garbage prevents Monk from being able to easily identify the murderer of sanitation union boss Jimmy Cusack, eventually causing him to have a psychotic break. Another example is when entering a chaotic murder scene in the episode "Mr. Monk Meets Dale the Whale", his first impulse is to straighten the lamps, though he is frequently able to hold off his fixations when examining bodies or collecting evidence.[25] Even though Monk's mental state in the series is said to be a result of his wife's death,[15][26] he shows signs of OCD in flashbacks dating back to childhood.[27] To deal with his OCD and phobias, Monk visits a psychiatrist – Dr. Charles Kroger (Stanley Kamel) in the first six seasons and Dr. Neven Bell (Héctor Elizondo) in the last two seasons – weekly, and at several points, daily.[28] Moments of extreme stress can cause Monk to enter a dissociative state, as seen in the Season 1 episode "Mr. Monk and the Earthquake"; he begins speaking in gibberish during these periods, severely hindering his investigative skills.
        
        Over the course of the show (roughly 8 years), Monk overcomes many of his phobias and some aspects of his OCD. Though he has not been cured of many of them, if any at all, he has been able to put them in the back of his mind when involved in case work. One breakthrough is shown in the season 8 episode "Mr. Monk Goes to Group Therapy", when Adrian is locked in a car trunk with his fellow OCD patient and personal rival, Harold Krenshaw. During the terrifying trip, both men overcome their longstanding claustrophobia (fear of small spaces), as well as their own differences, resulting in them becoming friends. Possibly due to this, as well as the many cases Monk has solved over the years, he is reinstated as detective first class by Stottlemeyer in the season 8 episode "Mr. Monk and the Badge". Though he is very excited about his reinstatement initially, Monk realizes that becoming a detective again did not mean that he would be happier. In a session with Dr. Bell, Monk realizes he was always happy as a private detective and consultant to the SFPD as his own boss. After overcoming his fear of heights and singlehandedly capturing a killer window-washer, Monk turns in his badge. In the series finale, he learns that his late wife, Trudy, had given birth to a daughter before they had met. The knowledge and events of the episode lead to him becoming more cheerful. "#;
        // let text = r#"Monk carries out futile and endless attempts to make the world balanced. Monk is fixated with symmetry, going so far as to always cut his pancakes into squares."#;
        println!("{:?}", summarize(text));
    }
}
